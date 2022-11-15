//! A repository contains packages that can be installed.
//!
//! Repositories can either be local, or linked to remotes, from which packages can be fetched.

#[cfg(feature = "network")]
pub mod remote;

use crate::package::Package;
use crate::version::Version;
use std::io;
use std::path::PathBuf;

#[cfg(feature = "network")]
use remote::Remote;

/// Structure representing a repository.
pub struct Repository {
	/// The path to the repository.
	path: String,
}

impl Repository {
	/// Loads and returns the list of all repositories.
	///
	/// Arguments:
    /// - `sysroot` is the path to the system's root.
	/// - `local_repos` is the list of paths of local repositories.
	pub fn load_all(
		sysroot: &str,
		local_repos: &[String],
	) -> io::Result<Vec<Self>> {
		let mut repos = vec![];

		let iter = local_repos.iter()
			.map(|path| Self::new(path.to_string()));
		repos.extend(iter);

		// TODO Load repos from remotes

		Ok(repos)
	}

	/// Creates a new instance from the given path.
	pub fn new(path: String) -> Self {
		Self {
			path,
		}
	}

	/// Returns the remote associated with the repository.
	#[cfg(feature = "network")]
	pub fn get_remote(&self) -> Option<Remote> {
		// TODO
		todo!();
	}

	/// Returns the path to the descriptor associated with the given package `pack`.
	pub fn get_cache_desc_path(&self, pack: &Package) -> PathBuf {
		format!("{}/{}/{}/desc", self.path, pack.get_name(), pack.get_version()).into()
	}

	/// Returns the path to the archive associated with the given package `pack`.
	pub fn get_cache_archive_path(&self, pack: &Package) -> PathBuf {
		format!("{}/{}/{}/archive", self.path, pack.get_name(), pack.get_version()).into()
	}

	/// Returns the latest version of the package with name `name` along with its associated
	/// repository.
	/// If the package doesn't exist, the function returns None.
	///
	/// Arguments:
	/// - `sysroot` is the path to the system's root.
	pub fn get_latest_package(
		&self,
		sysroot: &str,
		name: &str
	) -> io::Result<Option<Package>> {
		// TODO
		todo!();
	}
}

/// Returns the package with name `name` and version `version` along with its associated
/// repository.
/// If the package doesn't exist, the function returns None.
///
/// Arguments:
/// - `repos` is the list of repositories to check on.
/// - `sysroot` is the path to the system's root.
pub fn get_package<'a>(
	_repos: &'a [Repository],
	_sysroot: &str,
	_name: &str,
	_version: &Version,
) -> io::Result<Option<(&'a Repository, Package)>> {
	// TODO
	todo!();
}

/// Returns the latest version of the package with name `name` along with its associated
/// repository.
/// If the package doesn't exist, the function returns None.
///
/// Arguments:
/// - `repos` is the list of repositories to check on.
/// - `sysroot` is the path to the system's root.
pub fn get_latest_package<'a>(
	repos: &'a [Repository],
	sysroot: &str,
	name: &str
) -> io::Result<Option<(&'a Repository, Package)>> {
	for repo in repos {
		match repo.get_latest_package(sysroot, name)? {
			Some(pack) => return Ok(Some((repo, pack))),
			None => {},
		}
	}

	Ok(None)
}
