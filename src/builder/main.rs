//! The Blimp builder is a tool allowing to build a package.

mod build_desc;

use build_desc::BuildDescriptor;
use common::util;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::env;
use std::fs::File;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::exit;
use std::str;

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
	eprintln!("The software builds the package according to the package's build files, then \
writes the package's description `package.json` and archive `package.tar.gz` into the given \
destination directory.");
	eprintln!();
	eprintln!("When creating a package, the building process can be debugged using the \
BLIMP_DEBUG, allowing to keep the files after building to investigate problems");
	eprintln!();
	eprintln!("ENVIRONMENT VARIABLES:");
	eprintln!("\tJOBS: Specifies the recommended number of jobs to build the package");
	eprintln!("\tTARGET: The target for which the package is built");
	eprintln!("\tBLIMP_DEBUG: If set to one, the builder is set to debug mode");
}

/// Runs the build hook.
/// `hook_path` is the path to the hook file.
/// `build_dir` is the path to the build directory.
/// `sysroot` is the fake sysroot on which the package is installed before being compressed.
/// `jobs` is the recommended number of jobs to build this package.
/// `host` is the host triplet.
/// `target` is the target triplet.
fn run_build_hook(hook_path: &str, build_dir: &str, sysroot: &str, jobs: u32, host: &str,
	target: &str) {
	// TODO Pipe stdout and stderr into log files

	// TODO Clean
	let absolute_hook_path = fs::canonicalize(&PathBuf::from(hook_path)).unwrap().into_os_string()
		.into_string().unwrap();

	// Executing the build hook
	let status = Command::new(absolute_hook_path)
		.env("HOST", host)
		.env("TARGET", target)
		.env("SYSROOT", sysroot)
		.env("JOBS", format!("{}", jobs))
		.current_dir(build_dir)
		.status().unwrap_or_else(| e | {
			eprintln!("Failed to execute build hook: {}", e);
			exit(1);
		});
	
	// Tells whether the process succeeded
	let success = {
		if let Some(code) = status.code() {
			code == 0
		} else {
			false
		}
	};

	// On fail, exit
	if !success {
		eprintln!("Build hook failed!");
		exit(1);
	}
}

/// Returns the recommended amount of CPUs to build the package.
fn get_jobs_count() -> u32 {
	match env::var("JOBS") {
		Ok(s) => s.parse::<u32>().unwrap_or_else(| _ | {
			eprintln!("Invalid jobs count: {}", s);
			exit(1);
		}),

		Err(_) => 4, // TODO Get the number from the number of CPUs?
	}
}

/// Creates the archive.
fn create_archive(archive_path: &str, desc_path: &str, sysroot_path: &str) -> io::Result<()> {
	let tar_gz = File::create(archive_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.follow_symlinks(false);
    tar.append_path_with_name(desc_path, "package.json")?;
    tar.append_dir_all("data", sysroot_path)?;

	tar.finish()
}

/// Returns the triplet of the host on which the package is to be built.
fn get_host_triplet() -> String {
	env::var("HOST").unwrap_or_else(| _ | {
		let output = Command::new("cc")
			.arg("-dumpmachine")
			.output();

		if let Ok(out) = output {
			if let Ok(triplet) = str::from_utf8(&out.stdout) {
				return triplet.trim().to_owned();
			}
		}

		let default = "x86_64-linux-gnu".to_owned();
		eprintln!("Failed to retrieve host triplet. Defaulting to {}.", default);
		default
	})
}

// TODO Clean up on error
/// Builds the package.
/// `from` and `to` correspond to the command line arguments.
fn build(from: &str, to: &str) {
	let build_desc_path = format!("{}/package.json", from);
	let build_hook_path = format!("{}/build-hook", from);

	let desc_path = format!("{}/package.json", to);
	let archive_path = format!("{}/package.tar.gz", to);

	// If destination files already exist, fail
	if Path::new(&desc_path).exists() {
		eprintln!("{}: File exists", desc_path);
		exit(1);
	}
	if Path::new(&archive_path).exists() {
		eprintln!("{}: File exists", archive_path);
		exit(1);
	}

	// Reading the build descriptor
	let build_desc = util::read_json::<BuildDescriptor>(&build_desc_path).unwrap_or_else(| e | {
		eprintln!("Failed to read the build descriptor: {}", e);
		exit(1);
	});

	// The package
	let package = build_desc.get_package();

	println!("Building the package `{}` version `{}`...",
		package.get_name(), package.get_version());

	// The root of the build directory
	let build_dir = util::create_tmp_dir().unwrap_or_else(| e | {
		eprintln!("Failed to create the build directory: {}", e);
		exit(1);
	});
	// The fake sysroot on which the package will be installed to be compressed
	let sysroot = util::create_tmp_dir().unwrap_or_else(| e | {
		eprintln!("Failed to create the fake sysroot directory: {}", e);
		exit(1);
	});

	println!("Fetching sources...");

	for s in build_desc.get_sources() {
		println!("Fetching {}", s.get_url());

		s.fetch(&build_dir).unwrap_or_else(| e | {
			eprintln!("Failed to fetch sources: {}", e);
			exit(1);
		});
	}

	// Retrieving parameters from environment variables
	let jobs = get_jobs_count();
	let host = get_host_triplet();
	let target = env::var("TARGET").unwrap_or(host.clone());

	println!("Jobs: {}; Host: {}; Target: {}", jobs, host, target);
	println!();

	run_build_hook(&build_hook_path, &build_dir, &sysroot, jobs, &host, &target);

	println!("Writing built package...");

	// Writing the package descriptor
	util::write_json(&desc_path, package).unwrap_or_else(| e | {
		eprintln!("Failed to write descriptor file: {}", e);
		exit(1);
	});

	// Creating the archive
	create_archive(&archive_path, &desc_path, &sysroot).unwrap_or_else(| e | {
		eprintln!("Failed to create archive: {}", e);
		exit(1);
	});

	if env::var("BLIMP_DEBUG").unwrap_or("0".to_owned()) == "1" {
		println!("[DEBUG] The build directory is located at: {}", build_dir);
		println!("[DEBUG] The fake sysroot directory is located at: {}", sysroot);
	} else {
		println!("Cleaning up...");

		// Removing temporary directory
		let _ = fs::remove_dir_all(&sysroot);
		let _ = fs::remove_dir_all(&build_dir);
	}

	println!("Done! :)");
}

fn main() {
	let args: Vec<String> = env::args().collect();
	// The name of the binary file
	let bin = {
		if args.len() == 0 {
			String::from("blimp-builder")
		} else {
			args[0].clone()
		}
	};

	// If the argument count is incorrect, print usage
	if args.len() <= 1 || args.len() > 3 {
		print_usage(&bin);
		exit(1);
	}

	let from = args[1].clone();
	let to = {
		if args.len() < 3 {
			".".to_owned()
		} else {
			args[2].clone()
		}
	};

	build(&from, &to);
}
