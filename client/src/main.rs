//! Blimp is a simple package manager for Unix systems.

mod confirm;
mod install;
mod update;

use common::Environment;
use common::repository::remote::Remote;
use install::install;
use std::env;
use std::error::Error;
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

/// Returns an environment for the given sysroot.
///
/// If the environment's lockfile cannot be acquired, the function returns an error.
fn get_env(sysroot: PathBuf) -> Result<Environment, Box<dyn Error>> {
	Environment::with_root(sysroot).ok_or("failed to acquire lockfile".into())
}

/// Lists remotes.
fn remote_list(env: &Environment) -> Result<(), Box<dyn Error>> {
	let remotes = Remote::load_list(env)
		.map_err(|e| -> Box<dyn Error> {
			format!("IO error: {}", e).into()
		})?;

	println!("Remotes list:");

	for r in remotes {
		let host = r.get_host();

		match r.get_motd() {
			Ok(m) => println!("- {} (status: UP): {}", host, m),
			Err(_) => println!("- {} (status: DOWN)", host),
		}
	}

	Ok(())
}

/// Adds one or several remotes.
///
/// Arguments:
/// - `env` is the environment.
/// - `remotes` is the list of remotes to add.
fn remote_add(env: &mut Environment, remotes: &[String]) -> Result<(), Box<dyn Error>> {
	let mut list = Remote::load_list(env)?;
	list.sort();

	for r in remotes {
		match list.binary_search_by(|r1| r1.get_host().cmp(r)) {
			Ok(_) => eprintln!("Remote `{}` already exists", r),
			Err(_) => list.push(Remote::new(r.clone())),
		}
	}

	Remote::save_list(env, &list)?;
	Ok(())
}

/// Removes one or several remotes.
///
/// Arguments:
/// - `env` is the environment.
/// - `remotes` is the list of remotes to remove.
fn remote_remove(env: &mut Environment, remotes: &[String]) -> Result<(), Box<dyn Error>> {
	let mut list = Remote::load_list(env)?;
	list.sort();

	for r in remotes {
		match list.binary_search_by(|r1| r1.get_host().cmp(r)) {
			Ok(i) => {
				let _ = list.remove(i);
			}
			Err(_) => eprintln!("Remote `{}` not found", r),
		}
	}

	Remote::save_list(env, &list)?;
	Ok(())
}

fn main_(sysroot: PathBuf, local_repos: &[PathBuf]) -> Result<bool, Box<dyn Error>> {
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

		"update" => {
			let mut env = get_env(sysroot)?;
			update(&mut env)?;

			Ok(true)
		}

		"install" => {
			let names = &args[2..];
			if names.is_empty() {
				eprintln!("Please specify one or several packages");
				return Ok(false);
			}

			let mut env = get_env(sysroot)?;
			install(names, &mut env, local_repos)?;

			Ok(true)
		}

		"upgrade" => {
			let names = &args[2..];
			if names.is_empty() {
				eprintln!("Please specify one or several packages");
				return Ok(false);
			}

			let _env = get_env(sysroot)?;
			// TODO
			todo!();
		}

		"remove" => {
			let names = &args[2..];
			if names.is_empty() {
				eprintln!("Please specify one or several packages");
				return Ok(false);
			}

			let _env = get_env(sysroot)?;
			// TODO
			todo!();
		}

		"clean" => {
			let _env = get_env(sysroot)?;
			// TODO
			todo!();
		}

		"remote-list" => {
			let env = get_env(sysroot)?;
			remote_list(&env)?;

			Ok(true)
		}

		"remote-add" => {
			let names = &args[2..];
			if names.is_empty() {
				eprintln!("Please specify one or several remotes to add");
				return Ok(false);
			}

			let mut env = get_env(sysroot)?;
			remote_add(&mut env, names)?;

			Ok(true)
		}

		"remote-remove" => {
			let names = &args[2..];
			if names.is_empty() {
				eprintln!("Please specify one or several remotes to remove");
				return Ok(false);
			}

			let mut env = get_env(sysroot)?;
			remote_remove(&mut env, names)?;

			Ok(true)
		}

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
	let sysroot = env::var("SYSROOT")
		.map(PathBuf::from)
		.unwrap_or(PathBuf::from("/"));
	let local_repos = env::var("LOCAL_REPOSITORIES")
		.map(|s| s.split(":").map(|s| PathBuf::from(s)).collect())
		.unwrap_or(vec![]);

	match main_(sysroot, &local_repos) {
		Ok(false) => exit(1),

		Err(e) => {
			eprintln!();
			eprintln!("error: {}", e);
			exit(1);
		}

		_ => {},
	}
}
