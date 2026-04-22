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

//! A repository contains packages that can be installed.
//!
//! A repository can be linked to a remote, from which packages can be fetched.

#[cfg(feature = "network")]
pub mod remote;

use crate::{
	package::{self, DependencyType, Package},
	util::current_arch,
	version::{Version, VersionConstraint},
};
use anyhow::{bail, Result};
#[cfg(feature = "network")]
use remote::Remote;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
};

/// Map of packages with their respective repository
pub type PackagesWithRepositoryMap<'r> = HashMap<Package, &'r Repository>;

/// List of packages with their respective repository
pub type PackagesWithRepositoryVec<'r> = Vec<(Package, &'r Repository)>;

/// Packages for an architecture in an index
#[derive(Default, Deserialize, Serialize)]
pub struct IndexArch {
	/// Packages list
	pub package: Vec<Package>,
}

/// A repository's index
#[derive(Default, Deserialize, Serialize)]
pub struct Index {
	/// List of architectures in the index
	pub arch: HashMap<String, IndexArch>,
}

/// A local repository.
pub struct Repository {
	/// The path to the repository.
	path: PathBuf,
	/// The remote associated with the repository.
	#[cfg(feature = "network")]
	remote: Option<Remote>,
}

impl Repository {
	/// Read a local repository.
	pub fn local(path: PathBuf) -> Self {
		Self {
			path,
			#[cfg(feature = "network")]
			remote: None,
		}
	}

	/// Returns the repository's path
	#[inline]
	pub fn get_path(&self) -> &Path {
		&self.path
	}

	/// Returns the remote associated with the repository.
	#[cfg(feature = "network")]
	pub fn get_remote(&self) -> Option<&Remote> {
		self.remote.as_ref()
	}

	/// Returns the path to the repository's index
	pub fn get_index_path(&self) -> PathBuf {
		self.path.join("index")
	}

	/// Reads the repository's index
	pub fn read_index(&self) -> Result<Index> {
		let content = fs::read_to_string(self.get_index_path())?;
		Ok(toml::from_str(&content)?)
	}

	/// Returns the path to a package's metadata
	pub fn get_metadata_path(&self, arch: &str, name: &str, version: &Version) -> PathBuf {
		self.path
			.join("dist")
			.join(arch)
			.join(format!("{name}_{version}.meta"))
	}

	/// Returns the path to a package's archive
	pub fn get_archive_path(&self, arch: &str, name: &str, version: &Version) -> PathBuf {
		self.path
			.join("dist")
			.join(arch)
			.join(format!("{name}_{version}.tar.gz"))
	}

	/// Tells whether the **archive** of a package is present in the repository.
	pub fn is_in_cache(&self, arch: &str, name: &str, version: &Version) -> bool {
		self.get_metadata_path(arch, name, version).exists()
			&& self.get_archive_path(arch, name, version).exists()
	}

	/// Returns a package in the repository
	///
	/// If the package does not exist, the function returns `None`.
	pub fn get_package(
		&self,
		arch: &str,
		name: &str,
		version: &Version,
	) -> Result<Option<Package>> {
		if !package::is_valid_name(name) {
			bail!("invalid package name: {name}");
		}
		let path = self.get_metadata_path(arch, name, version);
		Package::from_file(&path)
	}

	// NOTE: used in maestro-install
	/// Returns the list of packages with each versions in
	/// the repository for the current architecture.
	pub fn list_packages(&self) -> Result<Vec<Package>> {
		let mut index = self.read_index()?;
		let arch = current_arch();
		// Remove to move the object out. We can do this since the index is dropped when the
		// function returns
		let Some(index_arch) = index.arch.remove(arch) else {
			return Ok(vec![]);
		};
		Ok(index_arch.package)
	}

	/// Returns the package with the given name.
	///
	/// Arguments:
	/// - `arch` is the required architecture
	/// - `name` is the name of the package
	/// - `version_constraint` is the version constraint to match. If no constraint is specified,
	///   the latest version is selected
	///
	/// If the package does not exist, the function returns `None`.
	pub fn get_package_with_constraint(
		&self,
		arch: &str,
		name: &str,
		version_constraint: Option<&VersionConstraint>,
	) -> Result<Option<Package>> {
		let mut index = self.read_index()?;
		// Remove to move the object out. We can do this since the index is dropped when the
		// function returns
		let Some(index_arch) = index.arch.remove(arch) else {
			return Ok(None);
		};
		let pkg = index_arch
			.package
			.into_iter()
			.filter(|pkg| {
				if pkg.name != name {
					return false;
				}
				if let Some(c) = version_constraint {
					c.is_valid(&pkg.version)
				} else {
					true
				}
			})
			.max_by(|p0, p1| p0.version.cmp(&p1.version));
		Ok(pkg)
	}
}

// TODO Handle error reporting
/// Returns the package with the given `arch`, `name` and `version` along with its associated
/// repository.
///
/// `repos` is the list of repositories to check on.
///
/// If the package does not exist, the function returns `None`.
pub fn get_package<'a>(
	repos: &'a [Repository],
	arch: &str,
	name: &str,
	version: &Version,
) -> Result<Option<(&'a Repository, Package)>> {
	if !package::is_valid_name(name) {
		bail!("invalid package name: {name}");
	}
	Ok(repos
		.iter()
		.find_map(|repo| match repo.get_package(arch, name, version) {
			Ok(Some(pack)) => Some((repo, pack)),
			_ => None,
		}))
}

// TODO Handle error reporting
/// Returns the package with the given constraints along with its associated repository.
///
/// Arguments:
/// - `arch` is the required architecture
/// - `name` is the name of the package
/// - `version_constraint` is the version constraint to match. If no constraint is specified, the
///   latest version is selected
///
/// If the package does not exist, the function returns `None`.
pub fn get_package_with_constraint<'a>(
	repos: &'a [Repository],
	arch: &str,
	name: &str,
	version_constraint: Option<&VersionConstraint>,
) -> Result<Option<(&'a Repository, Package)>> {
	if !package::is_valid_name(name) {
		bail!("invalid package name: {name}");
	}
	Ok(repos
		.iter()
		.filter_map(|repo| {
			match repo.get_package_with_constraint(arch, name, version_constraint) {
				Ok(Some(pack)) => Some((repo, pack)),
				_ => None,
			}
		})
		.max_by(|(_, p0), (_, p1)| p0.version.cmp(&p1.version)))
}

/// Appends recursive dependencies to given packages.
///
/// Arguments:
/// - `packages` is top level packages to resolve dependencies for.
/// - `repos` is repositories to search packages into.
/// - `dep_type` is the type of dependencies to resolve. `BuildAndRun` resolves everything.
/// - `arch` is the architecture to use.
pub fn get_recursive_dependencies<'r>(
	packages: &PackagesWithRepositoryMap<'r>,
	repos: &'r [Repository],
	dep_type: DependencyType,
	arch: &str,
) -> Result<PackagesWithRepositoryMap<'r>> {
	let mut failed = false;
	// The list of all packages, dependencies included
	let mut total_packages = packages.clone();
	// TODO check dependencies for all packages at once to avoid duplicate errors
	// Resolving dependencies
	for package in packages.keys() {
		let res = package.resolve_dependencies(
			&mut total_packages,
			dep_type.clone(),
			&mut |name, version_constraint| {
				// TODO yet another call for reading whole repo index
				let res = get_package_with_constraint(repos, arch, name, Some(version_constraint));
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
					None => todo!(),
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
		// TODO better exception handling
		bail!("installation failed");
	}
	Ok(total_packages)
}
