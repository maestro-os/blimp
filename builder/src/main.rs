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
use clap::Parser;
use common::{
	anyhow::{anyhow, bail, Result},
	repository::Repository,
	tokio::runtime::Runtime,
};
use std::{env, fs, path::PathBuf, process::exit, str};

/// The path to the work directory.
const WORK_DIR: &str = "work/";

/// Builds packages according to their descriptors.
#[derive(Parser, Debug)]
#[clap(after_long_help = "Environment variables:
\tJOBS: Specifies the recommended number of jobs to build the package
\tBUILD: Target triplet of the machine on which the package is built
\tHOST: Target triplet for which the package is built
\tTARGET: Target triplet for which the package builds (this is useful when cross-compiling compilers)
\tBLIMP_DEBUG: If set to `true`, build files are kept for troubleshooting purpose

All environment variables are optional")]
#[command(version, about, long_about = None)]
struct Args {
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

fn main_impl(args: Args) -> Result<()> {
	// Read environment
	let jobs = get_jobs_count()?;
	let build = get_build_triplet()?;
	let host = env::var("HOST");
	let host = host.as_deref().unwrap_or(build.as_str());
	let arch = get_arch(host);
	let target = env::var("TARGET");
	let target = target.as_deref().unwrap_or(host);
	let debug = env::var("BLIMP_DEBUG")
		.map(|s| s == "true")
		.unwrap_or(false);
	fs::create_dir_all(&args.to)
		.map_err(|e| anyhow!("failed to create destination directory: {e}"))?;
	println!("[INFO] Jobs: {jobs}; Build: {build}; Host: {host}; Target: {target}");
	let sysroot = (!args.package).then(|| args.to.clone());
	let build_process = BuildProcess::new(args.from, sysroot)?;
	let rt = Runtime::new()?;
	rt.block_on(build_process.fetch_sources())
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
		let repo = Repository::load(args.to.clone());
		build_process
			.write_metadata(&repo, arch)
			.map_err(|e| anyhow!("failed to write package metadata: {e}"))?;
		println!("[INFO] Create archive...");
		build_process
			.create_archive(&repo, arch)
			.map_err(|e| anyhow!("failed to create package archive: {e}"))?;
	}
	if debug {
		eprintln!(
			"[DEBUG] Build directory path: {}; Fake sysroot path: {}",
			build_process.get_build_dir().display(),
			build_process.get_sysroot().display()
		);
	} else {
		println!("[INFO] Cleaning up...");
		build_process.cleanup(args.package)?;
	}
	Ok(())
}

fn main() {
	let args = Args::parse();
	if let Err(e) = main_impl(args) {
		eprintln!("blimp-builder: error: {e}");
		exit(1);
	}
}
