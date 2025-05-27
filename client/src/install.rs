//! This module handles package installation.

use crate::confirm;
use common::{
	anyhow::{bail, Result},
	package::Package,
	repository,
	repository::Repository,
	Environment,
};
use std::{collections::HashMap, io, path::PathBuf};

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
	local_repos: Vec<PathBuf>,
) -> Result<()> {
	// The list of repositories
	let repos = local_repos
		.into_iter()
		.map(Repository::load)
		.collect::<io::Result<Vec<_>>>()?;
	// Tells whether the operation failed
	let mut failed = false;
	// The list of packages to install with their respective repository
	let mut packages = HashMap::<Package, &Repository>::new();
	for name in names {
		let pkg = repository::get_package_with_constraint(&repos, name, None)?;
		let Some((repo, pkg)) = pkg else {
			eprintln!("Package `{name}` not found!");
			failed = true;
			continue;
		};
		packages.insert(pkg, repo);
		// If already installed, print message
		if let Some(version) = env.get_installed_version(name) {
			println!("Package `{name}` version `{version}` is already installed. Reinstalling",);
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
			&mut |name, version_constraint| {
				let res = repository::get_package_with_constraint(
					&repos,
					name,
					Some(version_constraint),
				);
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
						todo!()
					}
				}
			},
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
			let name = &pkg.name;
			let version = &pkg.version;
			match repo.get_package(name, version)? {
				Some(_) => println!("\t- {name} ({version}) - cached"),
				None => {
					// Get package size from remote
					let remote = repo.get_remote().unwrap();
					let size = remote.get_size(pkg).await?;
					total_size += size;
					println!("\t- {name} ({version}) - download size: {size}");
				}
			}
		}
		println!(
			"Total download size: {}",
			common::maestro_utils::util::ByteSize(total_size)
		);
	}
	#[cfg(not(feature = "network"))]
	{
		for pkg in total_packages.keys() {
			println!("\t- {} ({}) - cached", pkg.name, pkg.version);
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
		// TODO download biggest packages first (sort_unstable by decreasing size)
		for (pkg, repo) in &total_packages {
			if repo.is_in_cache(&pkg.name, &pkg.version) {
				println!("`{}` is in cache.", &pkg.name);
				continue;
			}
			if let Some(remote) = repo.get_remote() {
				// TODO limit the number of packages downloaded concurrently
				futures.push((
					&pkg.name,
					&pkg.version,
					// TODO spawn task
					async {
						use common::download::DownloadTask;
						use std::fs::OpenOptions;

						let path = repo.get_archive_path(&pkg.name, &pkg.version);
						let file = OpenOptions::new()
							.create(true)
							.write(true)
							.truncate(true)
							.open(path)?;
						let url = remote.download_url(pkg);
						let mut task = DownloadTask::new(&url, &file).await?;
						while task.next().await? > 0 {}
						Ok::<(), common::anyhow::Error>(())
					},
				));
			}
		}
		for (name, version, f) in futures {
			match f.await {
				Ok(()) => continue,
				Err(error) => {
					eprintln!("Failed to download `{name}` version `{version}`: {error}")
				}
			}
		}
		if failed {
			bail!("installation failed");
		}
	}
	println!();
	println!("Installing packages...");
	// Install all packages
	for (pkg, repo) in total_packages {
		println!("Installing `{}`...", pkg.name);
		let archive_path = repo.get_archive_path(&pkg.name, &pkg.version);
		if let Err(e) = env.install(&pkg, &archive_path) {
			eprintln!("Failed to install `{}`: {e}", &pkg.name);
			failed = true;
		}
	}
	if failed {
		bail!("installation failed");
	}
	Ok(())
}
