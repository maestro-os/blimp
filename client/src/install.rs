//! This module handles package installation.

use common::package::Package;
use common::repository::Repository;
use common::repository::remote::Remote;
use common::repository;
use common::util;
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
	let mut packages = HashMap::<Package, &Repository>::new();

	for name in names {
		if let Some(pkg) = get_installed(sysroot, name)? {
			println!(
				"Package `{}` version `{}` is already installed. Skipping...",
				name, pkg.get_version()
			);

			continue;
		}

		match repository::get_package_with_constraints(&repos, &name, &[])? {
			Some((repo, package)) => {
				packages.insert(package, repo);
			}

			None => {
				eprintln!("Package `{}` not found!", name);
				failed = true;
			}
		}
	}
	if failed {
		return Err("installation failed".into());
	}

	println!("Resolving dependencies...");

	// The list of all packages, dependencies included
	let mut total_packages = packages.clone();

	// Resolving dependencies
	for (package, _) in packages {
		let res = package.resolve_dependencies(
			sysroot,
			&mut total_packages,
			&mut |name, version_constraints| {
				let res = repository::get_package_with_constraints(
					&repos,
					name,
					version_constraints
				)
					.or_else(|e| {
						eprintln!("error: {}", e);
						Err(())
					})
					.ok()?;

				// If not present, check on remote
				if res.is_none() {
					// TODO
					todo!();
				}

				let (repo, pkg) = res?;
				Some((pkg, repo))
			},
		)?;
		match res {
			Err(errs) => {
				for e in errs {
					eprintln!("{}", e);
				}

				failed = true;
			},

			_ => {},
		}
	}
	if failed {
		return Err("installation failed".into());
	}

	// Creating the async runtime
	let rt = Runtime::new().unwrap();

	println!("Packages to be installed:");

	// The total download size in bytes
	let mut total_size = 0;

	for (pkg, repo) in &total_packages {
		let name = pkg.get_name();
		let version = pkg.get_version();

		match repo.get_package(name, version)? {
			Some(_) => println!("\t- {} ({}) - cached", name, version),

			None => {
				let remote = repo.get_remote().unwrap();

				// Get package size from remote
				let size = rt.block_on(async {
					remote.get_size(pkg).await
				})?;
				total_size += size;

				println!("\t- {} ({}) - download size: {}", name, version, size);
			},
		}
	}

	print!("Total download size: ");
	util::print_size(total_size);
	println!();

	if !confirm::prompt() {
		println!("Aborting.");
		return Ok(());
	}

	println!("Downloading packages...");
	let mut futures = Vec::new();

	for (pkg, repo) in &total_packages {
		if repo.is_in_cache(pkg.get_name(), pkg.get_version()) {
			println!("`{}` is in cache.", pkg.get_name());
			continue;
		}

		if let Some(remote) = repo.get_remote() {
			futures.push((
				pkg.get_name(),
				pkg.get_version(),
				Remote::fetch_archive(remote, repo, pkg)
			));
		}
	}

	// TODO Add progress bar
	for (name, version, f) in futures {
		match rt.block_on(f) {
			Ok(_task) => {
				// TODO
			},

			Err(e) => eprintln!("Failed to download `{}` version `{}`: {}", name, version, e),
		}
	}
	if failed {
		return Err("installation failed".into());
	}

	println!();
	println!("Installing packages...");

	// Installing all packages
	for (pkg, repo) in total_packages {
		println!("Installing `{}`...", pkg.get_name());

		let archive_path = repo.get_archive_path(pkg.get_name(), pkg.get_version());
		if let Err(e) = common::install::install(sysroot, &pkg, &archive_path) {
			eprintln!("Failed to install `{}`: {}", pkg.get_name(), e);
		}
	}

	println!();
	Ok(())
}
