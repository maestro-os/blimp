//! Blimp is a simple package manager for Unix systems.

mod confirm;
mod install;
mod remove;

#[cfg(feature = "network")]
mod update;

use anyhow::anyhow;
use anyhow::Result;
use common::Environment;
use install::install;
use remove::remove;
use std::env;
use std::path::PathBuf;
use std::process::exit;
use tokio::runtime::Runtime;

#[cfg(feature = "network")]
use common::repository::remote::Remote;

/// Prints command line usage.
fn print_usage(bin: &str) {
	eprintln!(
		"blimp package manager version {}",
		env!("CARGO_PKG_VERSION")
	);
	eprintln!();
	eprintln!("USAGE:");
	eprintln!("\t{} <COMMAND> [OPTIONS]", bin);
	eprintln!();
	eprintln!("COMMAND:");
	eprintln!("\tinfo <package...>: Prints information about the given package(s)");
	eprintln!("\tinstall <package...>: Installs the given package(s)");
	eprintln!("\tupdate: Synchronizes packets information from remote");
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
		"\tLOCAL_REPO: Specifies paths separated by `:` at which packages are \
stored locally (the SYSROOT variable doesn't apply to these paths)"
	);
}

/// Prints the message to tell that the required feature is not available.
#[allow(unused)]
fn network_not_enabled() {
	eprintln!("This feature is not enabled. To use it, recompile with the feature `network`");
}

/// Returns an environment for the given sysroot.
///
/// If the environment's lockfile cannot be acquired, the function returns an error.
fn get_env(sysroot: PathBuf) -> Result<Environment> {
	Environment::with_root(sysroot).ok_or(anyhow!("failed to acquire lockfile"))
}

/// Lists remotes.
#[cfg(feature = "network")]
fn remote_list(env: &Environment) -> std::io::Result<()> {
	let remotes = Remote::load_list(env)?;

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
#[cfg(feature = "network")]
fn remote_add(env: &mut Environment, remotes: &[String]) -> std::io::Result<()> {
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
#[cfg(feature = "network")]
fn remote_remove(env: &mut Environment, remotes: &[String]) -> std::io::Result<()> {
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

fn main_(sysroot: PathBuf, local_repos: &[PathBuf]) -> Result<bool> {
	let args: Vec<String> = env::args().collect();
	// Name of the current binary file
	let bin = args.first().map(|s| s.as_str()).unwrap_or("blimp");

	// If no argument is specified, print usage
	if args.len() <= 1 {
		print_usage(bin);
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

		#[cfg(feature = "network")]
		"update" => {
			let mut env = get_env(sysroot)?;
			let rt = Runtime::new()?;
			rt.block_on(update::update(&mut env))?;

			Ok(true)
		}

		"install" => {
			let names = &args[2..];
			if names.is_empty() {
				eprintln!("Please specify one or several packages");
				return Ok(false);
			}

			let mut env = get_env(sysroot)?;
			let rt = Runtime::new()?;
			rt.block_on(install(names, &mut env, local_repos))?;

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

			let mut env = get_env(sysroot)?;
			remove(names, &mut env)?;

			Ok(true)
		}

		"clean" => {
			let _env = get_env(sysroot)?;
			// TODO
			todo!();
		}

		#[cfg(feature = "network")]
		"remote-list" => {
			let env = get_env(sysroot)?;
			remote_list(&env)?;

			Ok(true)
		}

		#[cfg(feature = "network")]
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

		#[cfg(feature = "network")]
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

		#[cfg(not(feature = "network"))]
		"update" | "remote-list" | "remote-add" | "remote-remove" => {
			network_not_enabled();
			Ok(false)
		}

		_ => {
			eprintln!("Command `{}` doesn't exist", args[1]);
			eprintln!();
			print_usage(bin);

			Ok(false)
		}
	}
}

fn main() {
	let sysroot = env::var_os("SYSROOT")
		.map(PathBuf::from)
		.unwrap_or(PathBuf::from("/"));
	let local_repos: Vec<PathBuf> = env::var("LOCAL_REPO") // TODO var_os
		.map(|s| s.split(':').map(PathBuf::from).collect())
		.unwrap_or_default();

	match main_(sysroot, &local_repos) {
		Ok(false) => exit(1),

		Err(e) => {
			eprintln!("error: {e}");
			exit(1);
		}

		_ => {}
	}
}
