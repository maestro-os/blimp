//! Blimp is a simple package manager for Unix systems.

mod confirm;
mod install;
#[cfg(feature = "network")]
mod remote;
mod remove;
#[cfg(feature = "network")]
mod update;

use common::{
	anyhow::{anyhow, Result},
	tokio, Environment,
};
use install::install;
use remove::remove;
use std::{
	env,
	path::{Path, PathBuf},
	process::exit,
};

/// Prints command line usage.
fn print_usage() {
	eprintln!(
		"blimp package manager version {}",
		env!("CARGO_PKG_VERSION")
	);
	eprintln!();
	eprintln!("USAGE:");
	eprintln!("\tblimp <COMMAND> [OPTIONS]");
	eprintln!();
	eprintln!("COMMAND:");
	eprintln!("\tinfo <package...>: Prints information about the given package(s)");
	eprintln!("\tinstall <package...>: Installs the given package(s)");
	#[cfg(feature = "network")]
	{
		eprintln!("\tupdate: Synchronizes packages information from remote");
	}
	eprintln!(
		"\tupgrade [package...]: Upgrades the given package(s). If no package is specified, \
the package manager updates every package that is not up to date"
	);
	eprintln!("\tremove <package...>: Removes the given package(s)");
	eprintln!("\tclean: Clean the cache");
	#[cfg(feature = "network")]
	{
		eprintln!("\tremote-list: Lists remote servers");
		eprintln!("\tremote-add <remote>: Adds a remote server");
		eprintln!("\tremote-remove <remote>: Removes a remote server");
	}
	eprintln!();
	eprintln!("OPTIONS:");
	eprintln!("\t--verbose: Enables verbose mode");
	eprintln!(
		"\t--version <version>: When installing or upgrading, this option allows to \
specify a version"
	);
	eprintln!();
	eprintln!("ENVIRONMENT VARIABLES:");
	eprintln!("\tSYSROOT: Specifies the path to the system's root");
	eprintln!(
		"\tLOCAL_REPO: Specifies paths separated by `:` at which packages are \
stored locally (the SYSROOT variable does not apply to these paths)"
	);
}

/// Returns an environment for the given sysroot.
///
/// If the environment's lockfile cannot be acquired, the function returns an error.
fn get_env(sysroot: &Path) -> Result<Environment> {
	Environment::with_root(sysroot)?.ok_or(anyhow!("failed to acquire lockfile"))
}

async fn main_impl(sysroot: &Path, local_repos: Vec<PathBuf>) -> Result<bool> {
	// If no argument is specified, print usage
	let args: Vec<String> = env::args().collect();
	if args.len() <= 1 {
		print_usage();
		return Ok(false);
	}
	// Match command
	match args[1].as_str() {
		"info" => {
			let packages = &args[2..];
			if packages.is_empty() {
				eprintln!("Please specify one or several packages");
				return Ok(false);
			}
			todo!()
		}
		#[cfg(feature = "network")]
		"update" => {
			let mut env = get_env(sysroot)?;
			update::update(&mut env).await?;
			Ok(true)
		}
		"install" => {
			let names = &args[2..];
			if names.is_empty() {
				eprintln!("Please specify one or several packages");
				return Ok(false);
			}
			let mut env = get_env(sysroot)?;
			install(names, &mut env, local_repos).await?;
			Ok(true)
		}
		"upgrade" => {
			let names = &args[2..];
			if names.is_empty() {
				eprintln!("Please specify one or several packages");
				return Ok(false);
			}
			let _env = get_env(sysroot)?;
			todo!()
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
			todo!()
		}
		#[cfg(feature = "network")]
		"remote-list" => {
			let env = get_env(sysroot)?;
			remote::list(&env).await?;
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
			remote::add(&mut env, names)?;
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
			remote::remove(&mut env, names)?;
			Ok(true)
		}
		#[cfg(not(feature = "network"))]
		"update" | "remote-list" | "remote-add" | "remote-remove" => {
			eprintln!(
				"This feature is not enabled. To use it, recompile the package manager with the feature `network`"
			);
			Ok(false)
		}
		cmd => {
			eprintln!("Command `{cmd}` does not exist");
			eprintln!();
			print_usage();
			Ok(false)
		}
	}
}

#[tokio::main]
async fn main() {
	let sysroot = env::var_os("SYSROOT")
		.map(PathBuf::from)
		.unwrap_or(PathBuf::from("/"));
	let local_repos = env::var("LOCAL_REPO") // TODO var_os
		.map(|s| s.split(':').map(PathBuf::from).collect())
		.unwrap_or_default();
	let res = main_impl(&sysroot, local_repos).await;
	match res {
		Ok(false) => exit(1),
		Err(e) => {
			eprintln!("error: {e}");
			exit(1);
		}
		_ => {}
	}
}
