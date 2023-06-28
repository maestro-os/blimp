//! This module handles package installation.

use anyhow::bail;
use crate::confirm;
use anyhow::Result;
use common::package::Package;
use common::repository;
use common::repository::Repository;
use common::util;
use common::Environment;
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "network")]
use common::repository::remote::Remote;

// TODO Clean
/// Installs the given list of packages.
///
/// Arguments:
/// - `names` is the list of packages to install.
/// - `env` is the blimp environment.
/// - `local_repos` is the list of paths to local package repositories.
pub async fn install(
	names: &[String],
	env: &mut Environment,
	local_repos: &[PathBuf],
) -> Result<()> {
	let installed = env.load_installed_list()?;

	let mut failed = false;

	// The list of repositories
	let repos = env.list_repositories(local_repos)?;
	// The list of packages to install with their respective repository
	let mut packages = HashMap::<Package, &Repository>::new();

	for name in names {
		let pkg = repository::get_package_with_constraints(&repos, name, &[])?;
		let Some((repo, pkg)) = pkg else {
            eprintln!("Package `{}` not found!", name);
            failed = true;
            continue;
        };
		packages.insert(pkg, repo);

		if let Some(installed) = installed.get(name) {
			println!(
				"Package `{}` version `{}` is already installed. Reinstalling",
				name,
				installed.desc.get_version()
			);
		}
	}
	if failed {
		bail!("installation failed");
	}

	println!("Resolving dependencies...");

	// The list of all packages, dependencies included
	let mut total_packages = packages.clone();

	// TODO check dependencies for all packages at once to avoid duplicate errors
	// Resolving dependencies
	for (package, _) in packages {
		let res = package.resolve_dependencies(
			&mut total_packages,
			&mut |name, version_constraints| {
				let res = repository::get_package_with_constraints(&repos, name, version_constraints);
				let pkg = match res {
					Ok(p) => p,
					Err(e) => {
						eprintln!("error: {e}");
						return None;
					}
				};

				match pkg {
					Some((repo, pkg)) => Some((pkg, repo)),

					// If not present, check on remote
					None => {
						// TODO
						todo!();
					}
				}
			}
		)?;
		if let Err(errs) = res {
			for e in errs {
				eprintln!("{e}");
			}

			failed = true;
		}
	}
	if failed {
		bail!("installation failed");
	}

	println!("Packages to be installed:");

	// List packages to be installed
	#[cfg(feature = "network")]
	{
		let mut total_size = 0;
		for (pkg, repo) in &total_packages {
			let name = pkg.get_name();
			let version = pkg.get_version();

			match repo.get_package(name, version)? {
				Some(_) => println!("\t- {} ({}) - cached", name, version),

				None => {
					let remote = repo.get_remote().unwrap();

					// Get package size from remote
					let size = remote.get_size(pkg).await?;
					total_size += size;

					println!("\t- {} ({}) - download size: {}", name, version, size);
				}
			}
		}

		print!("Total download size: ");
		util::print_size(total_size);
		println!();
	}
	#[cfg(not(feature = "network"))]
	{
		for pkg in total_packages.keys() {
			println!("\t- {} ({}) - cached", pkg.get_name(), pkg.get_version());
		}
	}

	if !confirm::prompt() {
		println!("Aborting.");
		return Ok(());
	}

	#[cfg(feature = "network")]
	{
		println!("Downloading packages...");
		let mut futures = Vec::new();

		// TODO download biggest packets first (sort_unstable by decreasing size)
		for (pkg, repo) in &total_packages {
			if repo.is_in_cache(pkg.get_name(), pkg.get_version()) {
				println!("`{}` is in cache.", pkg.get_name());
				continue;
			}

			if let Some(remote) = repo.get_remote() {
				// TODO limit the number of packages downloaded concurrently
				futures.push((
					pkg.get_name(),
					pkg.get_version(),
					// TODO spawn task
					async {
						let mut task = Remote::fetch_archive(remote, repo, pkg).await?;
						while task.next().await? {
							// TODO update progress bar
						}

						Ok::<(), anyhow::Error>(())
					},
				));
			}
		}

		// TODO Add progress bar
		for (name, version, f) in futures {
			match f.await {
				Ok(()) => continue,
				Err(e) => eprintln!("Failed to download `{}` version `{}`: {}", name, version, e),
			}
		}
		if failed {
			bail!("installation failed");
		}
	}

	println!();
	println!("Installing packages...");

	// Installing all packages
	for (pkg, repo) in total_packages {
		println!("Installing `{}`...", pkg.get_name());

		let archive_path = repo.get_archive_path(pkg.get_name(), pkg.get_version());
		if let Err(e) = env.install(&pkg, &archive_path) {
			eprintln!("Failed to install `{}`: {e}", pkg.get_name());
			failed = true;
		}
	}
	if failed {
		bail!("installation failed");
	}

	Ok(())
}
