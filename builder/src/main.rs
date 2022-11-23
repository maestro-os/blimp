//! The Blimp builder is a tool allowing to build a package.

use common::build::BuildProcess;
use std::env;
use std::path::PathBuf;
use std::process::exit;
use std::str;
use std::thread;

/// The software's current version.
const VERSION: &str = "0.1";

/// Prints command line usage.
fn print_usage(bin: &str) {
	eprintln!("blimp package builder version {}", VERSION);
	eprintln!();
	eprintln!("USAGE:");
	eprintln!("\t{} <FROM> [TO]", bin);
	eprintln!();
	eprintln!("FROM is the path to the package's build files");
	eprintln!("TO is the path to the directory where the files will be written");
	eprintln!();
	eprintln!(
		"The software builds the package according to the package's build files, then \
writes the package's description `package.json` and archive `package.tar.gz` into the given \
destination directory."
	);
	eprintln!();
	eprintln!(
		"When creating a package, the building process can be debugged using the \
BLIMP_DEBUG, allowing to keep the files after building to investigate problems"
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
fn get_host_triplet() -> String {
	env::var("HOST").unwrap_or_else(|_| {
		common::build::get_host_triplet()
			.unwrap_or_else(|| {
				let default = "x86_64-linux-gnu".to_owned();
				eprintln!(
					"Failed to retrieve host triplet. Defaulting to {}.",
					default
				);

				default
			})
	})
}

/// Builds the package.
///
/// `from` and `to` correspond to the command line arguments.
fn build(from: PathBuf, to: PathBuf) {
	let debug = env::var("BLIMP_DEBUG")
		.map(|s| s == "true")
		.unwrap_or(false);

	let jobs = get_jobs_count();
	let host = get_host_triplet();
	let target = env::var("TARGET")
		.unwrap_or_else(|_| host.clone());

	let mut build_process = BuildProcess::new(from);
	build_process.set_clean_on_drop(!debug);

	build_process.prepare().unwrap_or_else(|e| {
		eprintln!("Cannot prepare building process: {}", e);
		exit(1);
	});

	println!("[INFO] Fetching sources...");
	// TODO Progress bars

	build_process.fetch_sources().unwrap_or_else(|e| {
		eprintln!("Cannot fetch sources: {}", e);
		exit(1);
	});

	println!("[INFO] Compilation...");

	let success = build_process.build(jobs, &host, &target).unwrap_or_else(|e| {
		eprintln!("Cannot build package: {}", e);
		exit(1);
	});
	if !success {
		eprintln!("Package build failed!");
		exit(1);
	}

	println!("[INFO] Creating archive...");

	build_process.create_archive(&to).unwrap_or_else(|e| {
		eprintln!("Cannot create archive: {}", e);
		exit(1);
	});

	if debug {
		eprintln!(
			"[DEBUG] Build directory path: {}; Fake sysroot path: {}",
			build_process.get_build_dir().unwrap().display(),
			build_process.get_sysroot().unwrap().display()
		);
	}
}

fn main() {
	let args: Vec<String> = env::args().collect();
	// The name of the binary file
	let bin = args.first()
		.map(|s| s.as_str())
		.unwrap_or("blimp-builder");

	// If the argument count is incorrect, print usage
	if args.len() <= 1 || args.len() > 3 {
		print_usage(&bin);
		exit(1);
	}

	let from = PathBuf::from(&args[1]);
	let to = {
		if args.len() < 3 {
			PathBuf::from(".")
		} else {
			PathBuf::from(&args[2])
		}
	};

	build(from, to);
}
