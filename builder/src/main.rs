//! The Blimp builder is a tool allowing to build a package.

use anyhow::anyhow;
use anyhow::Result;
use common::build::BuildProcess;
use common::repository::Repository;
use common::util;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::exit;
use std::str;
use std::thread;
use tokio::runtime::Runtime;

/// Prints command line usage.
fn print_usage(bin: &str) {
	eprintln!(
		"blimp package builder version {}",
		env!("CARGO_PKG_VERSION")
	);
	eprintln!();
	eprintln!("USAGE:");
	eprintln!("\t{} <FROM> <TO>", bin);
	eprintln!();
	eprintln!("FROM is the path to the package's build files");
	eprintln!("TO is the repository in which the output package will be placed");
	eprintln!();
	eprintln!(
		"The software builds the package according to the package's build files, then writes the \
package into the repository at the given path."
	);
	eprintln!();
	eprintln!(
		"The building process can be debugged using the BLIMP_DEBUG environment variable, \
allowing to keep build files to troubleshoot problems"
	);
	eprintln!();
	eprintln!("ENVIRONMENT VARIABLES:");
	eprintln!("\tJOBS: Specifies the recommended number of jobs to build the package");
	eprintln!("\tTARGET: The target for which the package is built");
	eprintln!("\tBLIMP_DEBUG: If set to `true`, the builder is set to debug mode");
}

/// Returns the recommended amount of CPUs to build the package.
fn get_jobs_count() -> u32 {
	match env::var("JOBS") {
		Ok(s) => s.parse::<u32>().unwrap_or_else(|_| {
			eprintln!("Invalid jobs count: {}", s);
			exit(1);
		}),

		Err(_) => thread::available_parallelism()
			.map(|n| n.get() as u32)
			.unwrap_or(1),
	}
}

/// Returns the triplet of the host on which the package is to be built.
fn get_host_triplet() -> io::Result<String> {
	if let Ok(triplet) = env::var("HOST") {
		return Ok(triplet);
	}

	if let Some(triplet) = common::build::get_host_triplet()? {
		return Ok(triplet);
	}

	let default = "x86_64-linux-gnu".to_owned();
	eprintln!(
		"Failed to retrieve host triplet. Defaulting to {}.",
		default
	);
	Ok(default)
}

/// Builds the package.
///
/// `from` and `to` correspond to the command line arguments.
fn build(from: PathBuf, to: PathBuf) -> Result<()> {
	let debug = env::var("BLIMP_DEBUG")
		.map(|s| s == "true")
		.unwrap_or(false);

	let jobs = get_jobs_count();
	let host = get_host_triplet()?;
	let target = env::var("TARGET").unwrap_or_else(|_| host.clone());
	println!("[INFO] Jobs: {}; Host: {}; Target: {}", jobs, host, target);

	let build_process = BuildProcess::new(from)?;

	println!("[INFO] Fetching sources...");
	// TODO Progress bars

	let rt = Runtime::new()?;
	rt.block_on(build_process.fetch_sources())
		.map_err(|e| anyhow!("Cannot fetch sources: {e}"))?;

	println!("[INFO] Compilation...");

	let success = build_process
		.build(jobs, &host, &target)
		.map_err(|e| anyhow!("Cannot build package: {e}"))?;
	if !success {
		eprintln!("Package build failed!");
		exit(1);
	}

	println!("[INFO] Preparing repository at `{}`...", to.display());

	// TODO Move to separate function
	let archive_path = {
		let build_desc = build_process.get_build_desc();
		let name = build_desc.package.get_name();
		let version = build_desc.package.get_version();

		let package_path = to.join(name).join(version.to_string());
		fs::create_dir_all(&package_path)?;

		let desc_path = package_path.join("desc");
		util::write_json(&desc_path, &build_desc.package)?;

		let repo = Repository::load(to)?;
		repo.get_archive_path(name, version)
	};

	println!("[INFO] Creating archive...");

	build_process
		.create_archive(&archive_path)
		.unwrap_or_else(|e| {
			eprintln!("Cannot create archive: {}", e);
			exit(1);
		});

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
	let bin = args.first().map(|s| s.as_str()).unwrap_or("blimp-builder");

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
