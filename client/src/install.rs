/*
 * Copyright 2025 Luc Lenôtre
 *
 * This file is part of Maestro.
 *
 * Maestro is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Maestro is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Maestro. If not, see <https://www.gnu.org/licenses/>.
 */

//! This module handles package installation.

use crate::confirm;
use common::{
	anyhow::{bail, Result},
	maestro_utils::util::ByteSize,
	package::{DependencyType, Package},
	repository::{
		self, get_recursive_dependencies, PackagesWithRepositoryMap, PackagesWithRepositoryVec,
		Repository,
	},
	Environment,
};
use std::collections::HashMap;

/// Get the list of packages to install.
///
/// Print if package is missing or already installed.
///
/// If at least one package is missing, error out.
///
/// Arguments:
/// - `names` is packages names to search
/// - `repos` is repositories to search packages into
/// - `env` is the environment to install on
fn packages_to_install<'r>(
	names: &[String],
	repos: &'r [Repository],
	env: &Environment,
) -> Result<PackagesWithRepositoryMap<'r>> {
	let mut failed = false;
	let mut packages = HashMap::<Package, &Repository>::new();

	// TODO list all packages for all repo, instead of
	// reading index for each repo for each package
	for name in names {
		let pkg = repository::get_package_with_constraint(repos, env.arch(), name, None)?;
		let Some((repo, pkg)) = pkg else {
			eprintln!("Package `{name}` not found!");
			failed = true;
			continue;
		};
		packages.insert(pkg, repo);
		// If already installed, print message
		if let Some(version) = env.get_installed_version(name)? {
			println!("Package `{name}` version `{version}` is already installed. Reinstalling");
		}
	}
	if failed {
		bail!("installation failed");
	}
	Ok(packages)
}

/// Print download size for each dependency
/// and the total download size.
///
/// Arguments:
/// - `total_packages` is the whole list of packages to install
/// - `env` is the environment to install on
#[cfg(feature = "network")]
async fn print_download_size<'r>(
	total_packages: &PackagesWithRepositoryVec<'r>,
	env: &Environment,
) -> Result<()> {
	let mut total_size = 0;
	for (pkg, repo) in total_packages {
		let name = &pkg.name;
		let version = &pkg.version;
		match repo.get_package(env.arch(), name, version)? {
			Some(_) => println!("\t- {name} {version} - cached"),
			None => {
				// Get package size from remote
				let remote = repo.get_remote().unwrap();
				let size = remote.get_size(env, pkg).await?;
				total_size += size;
				println!("\t- {name} {version} (download size: {})", ByteSize(size));
			}
		}
	}
	println!();
	println!("Total download size: {}", ByteSize(total_size));
	println!();
	Ok(())
}

/// Installs the given list of packages.
///
/// Arguments:
/// - `names` is the list of packages to install.
/// - `env` is the blimp environment.
pub async fn install(names: &[String], env: &mut Environment) -> Result<()> {
	if names.is_empty() {
		bail!("must specify at least one package");
	}
	let repos = env.list_repositories()?;
	let packages = packages_to_install(names, &repos, env)?;

	println!("Resolving dependencies...");
	let total_packages =
		get_recursive_dependencies(&packages, &repos, DependencyType::Run, env.arch())?;
	let mut total_packages: Vec<_> = total_packages.into_iter().collect();
	total_packages.sort_unstable_by(|(p0, _), (p1, _)| p0.name.cmp(&p1.name));

	println!("Packages to be installed:");
	#[cfg(feature = "network")]
	print_download_size(&total_packages, env).await?;
	#[cfg(not(feature = "network"))]
	{
		for pkg in total_packages.keys() {
			println!("\t- {} {} - cached", pkg.name, pkg.version);
		}
	}
	if !confirm::prompt() {
		println!("Aborting.");
		return Ok(());
	}
	#[cfg(feature = "network")]
	{
		use common::repository::remote::download_packages;

		println!("Downloading packages...");
		download_packages(&total_packages, env).await?;
	}
	println!();
	println!("Installing packages...");
	env.install_packages(&total_packages)
}
