//! This module handles package installation.

use common::package::Package;
use common::repository::Repository;
use common::repository::remote::Remote;
use common::repository;
use common::util;
use common::version::Version;
use crate::confirm;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use tokio::runtime::Runtime;

// TODO Clean
/// Installs the given list of packages.
///
/// Arguments:
/// - `names` is the list of packages to install.
/// - `sysroot` is the path to the root of the system on which the packages will be installed.
/// - `local_repos` is the list of paths to local package repositories.
pub fn install(
	names: &[String],
	sysroot: &Path,
	local_repos: &[PathBuf],
) -> Result<(), Box<dyn Error>> {
	let mut failed = false;

	// The list of repositories
	let repos = Repository::load_all(sysroot, local_repos)?;
	// The list of packages to install with their respective repository
	let mut packages = HashMap::<String, (Package, &Repository)>::new();

	for p in names {
		match repository::get_latest_package(&repos, &p)? {
			Some((repo, package)) => {
				packages.insert(p.to_owned(), (package, repo));
			}

			None => {
				eprintln!("Package `{}` not found!", p);
				failed = true;
			}
		}
	}
	if failed {
		return Ok(());
	}

	println!("Resolving dependencies...");

	// The list of all packages, dependencies included
	let mut total_packages = packages.clone();

	// Resolving dependencies
	for (_, (package, _)) in packages {
		let valid = package.resolve_dependencies(
			sysroot,
			&mut total_packages,
			|name, version| {
				let r = repository::get_package(&repos, name, &version)
					.or_else(|e| {
						eprintln!("error: {}", e);
						Err(())
					})
					.ok()?;

				// TODO If not present, check on remote

				let (remote, package) = r?;
				Some((package, remote))
			},
		)?;

		if !valid {
			failed = true;
		}
	}
	if failed {
		return Ok(());
	}

	// Creating the async runtime
	let rt = Runtime::new().unwrap();
	let mut futures = Vec::new();

	println!("Packages to be installed:");

	for (name, (package, repo)) in &total_packages {
		// TODO get size from local or remote
		// TODO print size for each package

		if package.get_package() {
			println!("\t- {} ({}) - cached", name, package.get_version());
		} else {
			println!("\t- {} ({})", name, package.get_version());
		}
	}

	// The total download size in bytes
	let mut total_size = 0;
	for f in futures {
		total_size += rt.block_on(f)?;
	}

	print!("Download size: ");
	util::print_size(total_size);
	println!();

	if !confirm::prompt() {
		println!("Aborting.");
		return Ok(());
	}

	println!("Downloading packages...");
	let mut futures = Vec::new();

	for (name, (package, repo)) in &total_packages {
		if package.is_in_cache(sysroot) {
			println!("`{}` is in cache.", name);
			continue;
		}

		if let Some(remote) = repo.get_remote() {
			futures.push(Remote::fetch_archive(remote, repo, package));
		}
	}

	// TODO Add progress bar
	for f in futures {
		if !rt.block_on(f) {
			failed = true;
		}
	}

	if failed {
		return Ok(());
	}
	println!();

	println!("Installing packages...");

	// Installing all packages
	for (name, (pack, repo)) in total_packages {
		println!("Installing `{}`...", name);

		let archive_path = repo.get_archive_path(pack.get_name(), pack.get_version());
		if let Err(e) = common::install::install(sysroot, &pack, &archive_path) {
			eprintln!("Failed to install `{}`: {}", name, e);
		}
	}

	println!();
	Ok(())
}
