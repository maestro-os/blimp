//! A repository contains packages that can be installed.
//!
//! A repository can be linked to a remote, from which packages can be fetched.

#[cfg(feature = "network")]
pub mod remote;

use crate::{
	package,
	package::Package,
	version::{Version, VersionConstraint},
};
use anyhow::{bail, Result};
#[cfg(feature = "network")]
use remote::Remote;
use std::{
	fs,
	path::{Path, PathBuf},
};

/// A local repository.
pub struct Repository {
	/// The path to the repository.
	path: PathBuf,

	/// The remote associated with the repository.
	#[cfg(feature = "network")]
	remote: Option<Remote>,
}

impl Repository {
	/// Loads the repository at the given path.
	///
	/// If the repository is invalid, the function returns an error.
	pub fn load(path: PathBuf) -> Self {
		Self {
			path,

			#[cfg(feature = "network")]
			remote: None, // TODO read from file
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
		self.get_archive_path(arch, name, version).exists()
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
		Package::load(&path)
	}

	/// Returns the list of packages with each versions in the repository.
	pub fn list_packages(&self) -> Result<Vec<Package>> {
		fs::read_dir(&self.path)?
			.filter_map(|ent| {
				let ent = ent.ok()?;
				if !ent.file_type().ok()?.is_dir() {
					return None;
				}

				let name = ent.file_name().to_str()?.to_owned();
				let ent_path = self.path.join(name);

				let iter = fs::read_dir(&ent_path).ok()?.filter_map(move |ent| {
					let ent = ent.ok()?;
					if !ent.file_type().ok()?.is_dir() {
						return None;
					}

					let ent_name = ent.file_name().to_str()?.to_owned();
					let version = Version::try_from(ent_name.as_ref()).ok()?;

					let ent_path = ent_path.join(version.to_string());
					Package::load(&ent_path).transpose()
				});
				Some(iter)
			})
			.flatten()
			.collect()
	}

	/// Returns the package with the given name.
	///
	/// Arguments:
	/// - `arch` is the required architecture
	/// - `name` is the name of the package
	/// - `version_constraint` is the version constraint to match. If no constraint is specified,
	///   the latest version is selected
	///
	/// If the package doesn't exist, the function returns `None`.
	pub fn get_package_with_constraint(
		&self,
		arch: &str,
		name: &str,
		version_constraint: Option<&VersionConstraint>,
	) -> Result<Option<Package>> {
		let base_path = self.path.join("dist").join(arch);
		fs::read_dir(base_path)?
			.filter_map(|ent| {
				let ent = ent.ok()?;
				if !ent.file_type().ok()?.is_file() {
					return None;
				}
				let n = ent.file_name();
				let n = n.to_str()?;
				// Retrieve package name and version
				let name_version = n.strip_suffix(".meta")?;
				let (n, version) = name_version.split_once('_')?;
				if n != name {
					return None;
				}
				Version::try_from(version).ok()
			})
			.filter(|version| {
				if let Some(c) = version_constraint {
					c.is_valid(version)
				} else {
					true
				}
			})
			.max()
			.and_then(|version| self.get_package(arch, name, &version).transpose())
			.transpose()
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
		.filter_map(|repo| match repo.get_package(arch, name, version) {
			Ok(Some(pack)) => Some((repo, pack)),
			_ => None,
		})
		.next())
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
