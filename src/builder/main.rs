//! The Blimp builder is a tool allowing to build a package.

mod build_desc;

use build_desc::BuildDescriptor;
use common::util;
use std::env;
use std::path::Path;
use std::process::Command;
use std::process::exit;

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
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // The name of the binary file
    let bin = {
        if args.len() == 0 {
            String::from("blimp")
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

	let build_desc_path = format!("{}/package.json", from);
    let build_hook_path = format!("{}/build-hook", from);

    let desc_path = format!("{}/package.json", to);
    let archive_path = format!("{}/archive.tar.gz", to);

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

	// Fetch sources
	build_desc.fetch_all().unwrap_or_else(| e | {
		eprintln!("Failed to fetch sources: {}", e);
		exit(1);
	});

	// The root of the build directory
	let build_dir = "TODO"; // TODO

	// TODO Uncompress the data in /tmp (?)

	// The fake sysroot on which the package will be installed
	let sysroot = "TODO"; // TODO
	// The recommended number of jobs available to build this package
	let jobs = 4; // TODO

	// Executing the build hook
	// TODO cd to the data and run the build hook (set SYSROOT env var with the path to the data dir)
    let status = Command::new(build_hook_path)
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

	// Writing the package descriptor
	util::write_json(&desc_path, &build_desc.get_package()).unwrap_or_else(| e | {
		eprintln!("Failed to write descriptor file: {}", e);
		exit(1);
	});

	// TODO Compress the archive
	todo!();
}
