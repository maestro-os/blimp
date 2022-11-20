//! Blimp is a simple package manager for Unix systems.

mod confirm;
mod install;
mod update;

use common::lockfile;
use common::repository::remote::Remote;
use install::install;
use std::env;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use update::update;

/// The software's current version.
const VERSION: &str = "0.1";

/// Prints command line usage.
fn print_usage(bin: &str) {
	eprintln!("blimp package manager version {}", VERSION);
	eprintln!();
	eprintln!("USAGE:");
	eprintln!("\t{} <COMMAND> [OPTIONS]", bin);
	eprintln!();
	eprintln!("COMMAND:");
	eprintln!("\tinfo <package...>: Prints informations about the given package(s)");
	eprintln!("\tinstall <package...>: Installs the given package(s)");
	eprintln!("\tupdate: Synchronizes packets informations from remote");
	eprintln!(
		"\tupgrade [package...]: Upgrades the given package(s). If no package is specified, \
the package manager updates every packages that are not up to date"
	);
	eprintln!("\tremove <package...>: Removes the given package(s)");
	eprintln!("\tclean: Clean the cache");
	eprintln!("\tremote-list: Lists remote servers");
	eprintln!("\tremote-add <remote>: Adds a remote server");
	eprintln!("\tremote-remove <remote>: Removes a remote server");
	eprintln!();
	eprintln!("OPTIONS:");
	eprintln!("\t--verbose: Enables verbose mode");
	eprintln!(
		"\t--version <version>: When installing or upgrading, this option allows to\
specify a version"
	);
	eprintln!();
	eprintln!("ENVIRONMENT VARIABLES:");
	eprintln!("\tSYSROOT: Specifies the path to the system's root");
	eprintln!(
		"\tLOCAL_REPOSITORIES: Specifies paths separated by `:` at which packages are \
stored locally (the SYSROOT variable doesn't apply to these paths)"
	);
}

/// Lists remotes.
/// `sysroot` is the path to the root of the system.
fn remote_list(sysroot: &str) -> bool {
	match Remote::load_list(sysroot) {
		Ok(remotes) => {
			println!("Remotes list:");

			for r in remotes {
				let host = r.get_host();

				match r.get_motd() {
					Ok(m) => println!("- {} (status: UP): {}", host, m),
					Err(_) => println!("- {} (status: DOWN)", host),
				}
			}

			true
		}

		Err(e) => {
			eprintln!("IO error: {}", e);
			false
		}
	}
}

/// Adds one or several remotes.
/// `sysroot` is the path to the root of the system.
/// `remotes` is the list of remotes to add.
fn remote_add(sysroot: &str, remotes: &[String]) -> bool {
	let mut list = match Remote::load_list(sysroot) {
		Ok(l) => l,
		Err(e) => {
			eprintln!("Cannot read remotes list: {}", e);
			return false;
		}
	};
	list.sort();

	for r in remotes {
		match list.binary_search_by(|r1| r1.get_host().cmp(r)) {
			Ok(_) => eprintln!("Remote `{}` already exists", r),
			Err(_) => list.push(Remote::new(r.clone())),
		}
	}

	match Remote::save_list(sysroot, &list) {
		Ok(_) => true,
		Err(e) => {
			eprintln!("Cannot write remotes list: {}", e);
			false
		}
	}
}

/// Removes one or several remotes.
/// `sysroot` is the path to the root of the system.
/// `remotes` is the list of remotes to remove.
fn remote_remove(sysroot: &str, remotes: &[String]) -> bool {
	let mut list = match Remote::load_list(sysroot) {
		Ok(l) => l,
		Err(e) => {
			eprintln!("Cannot read remotes list: {}", e);
			return false;
		}
	};
	list.sort();

	for r in remotes {
		match list.binary_search_by(|r1| r1.get_host().cmp(r)) {
			Ok(i) => {
				let _ = list.remove(i);
			}
			Err(_) => eprintln!("Remote `{}` not found", r),
		}
	}

	match Remote::save_list(sysroot, &list) {
		Ok(_) => true,
		Err(e) => {
			eprintln!("Cannot write remotes list: {}", e);
			false
		}
	}
}

fn main_(sysroot: &Path, local_repos: &[PathBuf]) -> Result<bool, Box<dyn Error>> {
	let args: Vec<String> = env::args().collect();
	// Name of the current binary file
	let bin = args.first().map(|s| s.as_str()).unwrap_or("blimp");

	// If no argument is specified, print usage
	if args.len() <= 1 {
		print_usage(&bin);
		return Ok(false);
	}

	// Matching command
	match args[1].as_str() {
		"info" => {
			let packages = &args[2..];
			if packages.is_empty() {
				eprintln!("Please specify one or several packages");
				return Ok(false);
			}

			// TODO
			todo!();
		}

		"update" => lockfile::lock_wrap(|| update(sysroot), sysroot),

		"install" => lockfile::lock_wrap(|| {
				let names = &args[2..];
				if names.len() == 0 {
					eprintln!("Please specify one or several packages");
					return Ok(false);
				}

				install(names, &sysroot, local_repos)?;
				println!("Done! :)");
				Ok(true)
			},
			sysroot,
		)?,

		"upgrade" => lockfile::lock_wrap(|| {
				let _packages = &args[2..];

				// TODO
				todo!();
			},
			sysroot,
		),

		"remove" => lockfile::lock_wrap(|| {
				let packages = &args[2..];
				if packages.len() == 0 {
					eprintln!("Please specify one or several packages");
					return false;
				}

				// TODO
				todo!();
			},
			sysroot,
		),

		"clean" => lockfile::lock_wrap(|| {
				// TODO
				todo!();
			},
			sysroot,
		),

		"remote-list" => lockfile::lock_wrap(|| remote_list(sysroot), sysroot),

		"remote-add" => lockfile::lock_wrap(|| {
				if args.len() <= 2 {
					eprintln!("Please specify a remote(s) to add");
					return false;
				}

				remote_add(sysroot, &args[2..])
			},
			sysroot,
		),

		"remote-remove" => lockfile::lock_wrap(|| {
				if args.len() <= 2 {
					eprintln!("Please specify a remote(s) to remove");
					return false;
				}

				remote_remove(sysroot, &args[2..])
			},
			sysroot,
		),

		_ => {
			eprintln!("Command `{}` doesn't exist", args[1]);
			eprintln!();
			print_usage(&bin);

			Ok(false)
		}
	}
}

fn main() {
	// Getting the sysroot
	let sysroot = env::var("SYSROOT").unwrap_or("/".to_string());
	let local_repos = env::var("LOCAL_REPOSITORIES")
		.map(|s| s.split(":").map(|s| PathBuf::from(s)).collect())
		.unwrap_or(vec![]);

	match main_(&sysroot, &local_repos) {
		Ok(false) => exit(1),

		Err(e) => {
			eprintln!("{}", e);
			exit(1);
		}
	}
}
