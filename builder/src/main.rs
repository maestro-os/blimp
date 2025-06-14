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
use common::{
	anyhow::{anyhow, bail, Result},
	clap::Parser,
	repository::Repository,
	serde_json,
	tokio::runtime::Runtime,
};
use std::{env, fs, io, path::PathBuf, process::exit, str};

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

All environment variable are optional")]
#[command(version, about, long_about = None)]
struct Args {
	/// Path to the directory containing the package to build.
	#[arg(long)]
	from: PathBuf,
	/// Output directory path.
	#[arg(long)]
	to: PathBuf,
	/// If set, the package is packed into an archive, written to this directory.
	/// Else, the package is directly *installed* in this directory (which acts as the system
	/// root).
	#[arg(long)]
	package: bool,
}

/// Prepares the repository's directory for the package.
///
/// On success, the function returns the output archive path.
fn prepare(build_process: &BuildProcess, to: PathBuf) -> io::Result<PathBuf> {
	// Create directory
	let build_desc = build_process.get_build_desc();
	let name = &build_desc.package.name;
	let version = &build_desc.package.version;
	let package_path = to.join(name).join(version.to_string());
	fs::create_dir_all(&package_path)?;
	// Create descriptor
	let desc_path = package_path.join("desc");
	let desc = serde_json::to_string(&build_desc.package)?;
	fs::write(desc_path, desc)?;
	// Get archive path
	let repo = Repository::load(to)?;
	Ok(repo.get_archive_path(name, version))
}

fn main_impl(args: Args) -> Result<()> {
	// Read environment
	let jobs = get_jobs_count()?;
	let build = get_build_triplet()?;
	let host = env::var("HOST");
	let host = host.as_deref().unwrap_or(build.as_str());
	let target = env::var("TARGET");
	let target = target.as_deref().unwrap_or(host);
	let debug = env::var("BLIMP_DEBUG")
		.map(|s| s == "true")
		.unwrap_or(false);
	println!("[INFO] Jobs: {jobs}; Build: {build}; Host: {host}; Target: {target}");
	let build_process = BuildProcess::new(args.from, (!args.package).then(|| args.to.clone()))?;
	let rt = Runtime::new()?;
	rt.block_on(build_process.fetch_sources())
		.map_err(|e| anyhow!("Cannot fetch sources: {e}"))?;
	println!("[INFO] Compilation...");
	let success = build_process
		.build(jobs, &build, host, target)
		.map_err(|e| anyhow!("Cannot build package: {e}"))?;
	if !success {
		bail!("Package build failed!");
	}
	if args.package {
		println!("[INFO] Prepare repository at `{}`...", args.to.display());
		let archive_path = prepare(&build_process, args.to)
			.map_err(|e| anyhow!("Failed to prepare directory for package: {e}"))?;
		println!("[INFO] Create archive...");
		build_process
			.create_archive(&archive_path)
			.map_err(|e| anyhow!("Cannot create archive: {e}"))?;
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
