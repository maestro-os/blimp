//! Utility allowing to build packages.

mod build;
mod desc;
mod util;

use crate::build::BuildProcess;
use crate::util::{get_build_triplet, get_jobs_count};
use anyhow::Result;
use anyhow::{anyhow, bail};
use common::repository::Repository;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::str;
use tokio::runtime::Runtime;

/// The path to the work directory.
const WORK_DIR: &str = "work/";

/// Prints command line usage.
fn print_usage(bin: &str) {
	eprintln!(
		"blimp package builder version {}",
		env!("CARGO_PKG_VERSION")
	);
	eprintln!();
	eprintln!("USAGE:");
	eprintln!("\t{bin} <FROM> <TO>");
	eprintln!();
	eprintln!("FROM is the path to the package's build files");
	eprintln!("TO is the repository in which the output package will be placed");
	eprintln!();
	eprintln!("Builds packages according to their descriptor, then writes them into the repository at the given path.");
	eprintln!();
	eprintln!("ENVIRONMENT VARIABLES:");
	eprintln!("\tJOBS: Specifies the recommended number of jobs to build the package");
	eprintln!("\tBUILD: Target triplet of the machine on which the package is built");
	eprintln!("\tHOST: Target triplet for which the package is built");
	eprintln!("\tTARGET: Target triplet for which the package builds (this is useful when cross-compiling compilers)");
	eprintln!("\tBLIMP_DEBUG: If set to `true`, build files are kept for troubleshooting purpose");
	eprintln!();
	eprintln!("All environment variable are optional");
}

/// Builds the package.
///
/// `from` and `to` correspond to the command line arguments.
fn build(from: PathBuf, to: PathBuf) -> Result<()> {
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
	let build_process = BuildProcess::new(from)?;
	println!("[INFO] Fetch sources...");
	// TODO Progress bars
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
	println!("[INFO] Prepare repository at `{}`...", to.display());
	// TODO Move to separate function
	let archive_path = {
		let build_desc = build_process.get_build_desc();
		let name = build_desc.package.get_name();
		let version = build_desc.package.get_version();
		let package_path = to.join(name).join(version.to_string());
		fs::create_dir_all(&package_path)?;
		let desc_path = package_path.join("desc");
		common::util::write_json(&desc_path, &build_desc.package)?;
		let repo = Repository::load(to)?;
		repo.get_archive_path(name, version)
	};
	println!("[INFO] Create archive...");
	build_process
		.create_archive(&archive_path)
		.map_err(|e| anyhow!("Cannot create archive: {e}"))?;
	if debug {
		eprintln!(
			"[DEBUG] Build directory path: {}; Fake sysroot path: {}",
			build_process.get_build_dir().display(),
			build_process.get_sysroot().display()
		);
	} else {
		println!("[INFO] Cleaning up...");
		build_process.cleanup()?;
	}
	Ok(())
}

fn main() {
	let args: Vec<String> = env::args().collect();
	// The name of the binary file
	let bin = args.first().map(String::as_ref).unwrap_or("blimp-builder");
	// If the argument count is incorrect, print usage
	if args.len() != 3 {
		print_usage(bin);
		exit(1);
	}
	let from = PathBuf::from(&args[1]);
	let to = PathBuf::from(&args[2]);
	if let Err(e) = build(from, to) {
		eprintln!("{bin}: error: {e}");
		exit(1);
	}
}
