/*
 * Copyright 2025 Luc Lenôtre
 *
 * This file is part of Maestro.
 *
 * Maestro is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Maestro is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Maestro. If not, see <https://www.gnu.org/licenses/>.
 */

//! Utility allowing to build packages.

mod build;
#[allow(unused)]
mod cache;
mod desc;
mod util;

use crate::{
	build::BuildProcess,
	util::{get_build_triplet, get_jobs_count},
};
use clap::{Args, Parser, Subcommand};
use common::{
	anyhow::{self, anyhow, bail, Result},
	repository::{Index, IndexArch, Repository},
	tokio::runtime::Runtime,
};
use s3::{creds::Credentials, Region};
use std::{fs, path::PathBuf, process::exit, str, str::FromStr};

/// Build, store and index packages
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
	/// Build a package
	Build(BuildArgs),
	/// Build the index of a s3 bucket repository
	Index(IndexArgs),
	/// Upload packages from a local repository to a s3 bucket
	Upload(UploadArgs),
}

/// Build a package according to its descriptor
#[derive(Args, Debug)]
struct BuildArgs {
	/// Path to the directory containing the package to build
	#[arg(long)]
	from: PathBuf,
	/// Output directory path
	#[arg(long)]
	to: PathBuf,
	/// If set, the package is packed into an archive, written to this directory.
	/// Else, the package is directly *installed* in this directory (which acts as the system
	/// root)
	#[arg(long)]
	package: bool,

	/// Specifies the recommended number of jobs to build the package
	#[arg(short, long)]
	jobs: Option<usize>,
	/// Target triplet of the machine on which the package is built
	#[arg(long)]
	build: Option<String>,
	/// Target triplet of the machine for which the package is built
	#[arg(long)]
	host: Option<String>,
	/// Target triplet for which the package builds (this is useful when cross-compiling
	/// compilers)
	#[arg(long)]
	target: Option<String>,
	/// Build in a chroot environment
	#[arg(long)]
	chroot: bool,

	/// If set, build files are kept for troubleshooting purpose
	#[arg(long)]
	debug: bool,

	/// Path to the work directory, containing build directories
	#[arg(long, default_value = "work/")]
	work_dir: PathBuf,
}

/// Upload a package to a s3 bucket repository
#[derive(Args, Debug)]
struct UploadArgs {
	/// Path to the package file (`.tar.gz` or `.meta`); both files are uploaded
	#[arg(long)]
	from: PathBuf,
	/// Bucket name
	#[arg(long)]
	bucket: String,
	/// Bucket region
	#[arg(long)]
	region: String,
	/// Bucket endpoint
	#[arg(long)]
	endpoint: Option<String>,
}

/// Index a s3 bucket repository
#[derive(Args, Debug)]
struct IndexArgs {
	/// Bucket name
	#[arg(long)]
	bucket: String,
	/// Bucket region
	#[arg(long)]
	region: String,
	/// Bucket endpoint
	#[arg(long)]
	endpoint: Option<String>,
}

/// Returns the architecture directory name for the given `host`
fn get_arch(host: &str) -> &str {
	let arch = host.split_once('-').map(|(a, _)| a);
	match arch {
		Some("i386" | "i486" | "i586" | "i686") => "x86",
		Some(a) => a,
		None => host,
	}
}

fn build(args: BuildArgs) -> Result<()> {
	// Read environment
	let jobs = get_jobs_count(&args);
	let build = get_build_triplet(&args)?;
	let host = args.host.as_deref().unwrap_or(build.as_str());
	let arch = get_arch(host);
	let target = args.target.as_deref().unwrap_or(host);
	fs::create_dir_all(&args.to)
		.map_err(|e| anyhow!("failed to create destination directory: {e}"))?;
	println!("[INFO] Jobs: {jobs}; Build: {build}; Host: {host}; Target: {target}");
	let pkg_path = (!args.package).then(|| args.to.clone());
	let rt = Runtime::new()?;
	let build_process = rt
		.block_on(async {
			let build_process =
				BuildProcess::new(args.from, pkg_path, &args.work_dir, args.chroot).await?;
			build_process.fetch_sources().await?;
			Ok::<_, anyhow::Error>(build_process)
		})
		.map_err(|e| anyhow!("cannot fetch sources: {e}"))?;
	println!("[INFO] Compilation...");
	let success = build_process
		.build(jobs, &build, host, target)
		.map_err(|e| anyhow!("cannot build package: {e}"))?;
	if !success {
		bail!("package build failed");
	}
	if args.package {
		println!("[INFO] Prepare repository at `{}`...", args.to.display());
		let repo = Repository::local(args.to.clone());
		build_process
			.write_metadata(&repo, arch)
			.map_err(|e| anyhow!("failed to write package metadata: {e}"))?;
		println!("[INFO] Create archive...");
		build_process
			.create_archive(&repo, arch)
			.map_err(|e| anyhow!("failed to create package archive: {e}"))?;
	}
	if args.debug {
		eprintln!(
			"[DEBUG] Build directory path: {}; Install path: {}",
			build_process.get_build_dir().display(),
			build_process.get_install_path().display()
		);
	} else {
		println!("[INFO] Cleaning up...");
		build_process.cleanup(args.package)?;
	}
	Ok(())
}

async fn upload(args: UploadArgs) -> Result<()> {
	let region = match args.endpoint {
		Some(endpoint) => Region::Custom {
			region: args.region,
			endpoint,
		},
		None => Region::from_str(&args.region)?,
	};
	let credentials = Credentials::default()?;
	let bucket = s3::Bucket::new(&args.bucket, region, credentials)?;
	// Derive the stem (strip .tar.gz or .meta extension)
	let path = args.from.canonicalize()?;
	let dir = path
		.parent()
		.ok_or_else(|| anyhow!("path has no parent directory"))?;
	let arch = dir
		.file_name()
		.and_then(|n| n.to_str())
		.ok_or_else(|| anyhow!("cannot determine architecture from parent directory name"))?;
	let filename = path
		.file_name()
		.and_then(|n| n.to_str())
		.ok_or_else(|| anyhow!("invalid filename"))?;
	let stem = filename
		.strip_suffix(".tar.gz")
		.or_else(|| filename.strip_suffix(".meta"))
		.ok_or_else(|| anyhow!("file must end with `.tar.gz` or `.meta`"))?;
	for ext in [".tar.gz", ".meta"] {
		let file_path = dir.join(format!("{stem}{ext}"));
		let key = format!("dist/{arch}/{stem}{ext}");
		println!("Upload `{key}`...");
		let data = fs::read(&file_path)
			.map_err(|e| anyhow!("failed to read `{}`: {e}", file_path.display()))?;
		bucket.put_object(&key, &data).await?;
	}
	println!("Done!");
	Ok(())
}

async fn index(args: IndexArgs) -> Result<()> {
	let region = match args.endpoint {
		Some(endpoint) => Region::Custom {
			region: args.region,
			endpoint,
		},
		None => Region::from_str(&args.region)?,
	};
	let credentials = Credentials::default()?;
	let bucket = s3::Bucket::new(&args.bucket, region, credentials)?;
	let entries = bucket.list("dist/".to_owned(), None).await?;
	let iter = entries.into_iter().flat_map(|n| n.contents).flat_map(|e| {
		let key = e.key.strip_prefix("dist/")?;
		let separator_off = key.find('/')?;
		let (arch, _) = key.split_at(separator_off);
		if e.key.ends_with(".meta") {
			Some((arch.to_owned(), e.key))
		} else {
			None
		}
	});
	let mut index = Index::default();
	for (arch, key) in iter {
		println!("Download `{key}`...");
		let data = bucket.get_object(&key).await?.to_vec();
		let Ok(data) = str::from_utf8(&data) else {
			eprintln!("warning: `{key}` has invalid UTF8, ignored");
			continue;
		};
		let pkg = match toml::from_str(data) {
			Ok(p) => p,
			Err(e) => {
				eprintln!("warning: `{key}` is invalid, ignored: {e}");
				continue;
			}
		};
		let ent = index.arch.entry(arch).or_insert(IndexArch::default());
		ent.package.push(pkg);
	}
	if index.arch.is_empty() {
		eprintln!("warning: no package found");
	}
	println!("Upload index...");
	let index = toml::to_string(&index).unwrap();
	bucket.put_object("/index", index.as_bytes()).await?;
	println!("Done!");
	Ok(())
}

fn main_impl(cmd: Command) -> Result<()> {
	match cmd {
		Command::Build(a) => build(a),
		Command::Index(a) => {
			let rt = Runtime::new()?;
			rt.block_on(index(a))
		}
		Command::Upload(a) => {
			let rt = Runtime::new()?;
			rt.block_on(upload(a))
		}
	}
}

fn main() {
	let cli = Cli::parse();
	if let Err(e) = main_impl(cli.command) {
		eprintln!("blimp-builder: error: {e}");
		exit(1);
	}
}
