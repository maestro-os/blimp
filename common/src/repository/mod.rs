//! A repository contains packages that can be installed.
//!
//! A repository can be linked to a remote, from which packages can be fetched.

#[cfg(feature = "network")]
pub mod remote;

use crate::package::Package;
use crate::version::Version;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "network")]
use remote::Remote;

/// Structure representing a local repository.
pub struct Repository {
	/// The path to the repository.
	path: PathBuf,

	/// The remote associated with the repository.
	#[cfg(feature = "network")]
	remote: Option<Remote>,
}

impl Repository {
	/// Loads and returns the list of all repositories.
	///
	/// Arguments:
	/// - `sysroot` is the path to the system's root.
	/// - `local_repos` is the list of paths of local repositories.
	pub fn load_all(_sysroot: &Path, local_repos: &[PathBuf]) -> io::Result<Vec<Self>> {
		let mut repos = vec![];

		// TODO List from blimp directory using sysroot

		let mut local_repos = local_repos.iter()
			.map(|path| Self::load(path.clone()))
			.collect::<Result<Vec<_>, _>>()?;
		repos.append(&mut local_repos);

		Ok(repos)
	}

	/// Loads the repository at the given path.
	///
	/// If the repository is invalid, the function returns an error.
	pub fn load(path: PathBuf) -> io::Result<Self> {
		Ok(Self {
			path,

			#[cfg(feature = "network")]
			remote: None, // TODO read from file
		})
	}

	/// Returns the remote associated with the repository.
	#[cfg(feature = "network")]
	pub fn get_remote(&self) -> Option<&Remote> {
		self.remote.as_ref()
	}

	/// Returns the path to the descriptor of the package with the given name `name` and version
	/// `version`.
	pub fn get_desc_path(&self, name: &str, version: &Version) -> PathBuf {
		let mut path = self.path.clone();
		path.push(&self.path);
		path.push(name);
		path.push(version.to_string());
		path.push("desc");

		path
	}

	/// Returns the path to the archive of the package with the given name `name` and version
	/// `version`.
	pub fn get_archive_path(&self, name: &str, version: &Version) -> PathBuf {
		let mut path = self.path.clone();
		path.push(&self.path);
		path.push(name);
		path.push(version.to_string());
		path.push("archive");

		path
	}

	/// Returns the package with name `name` and version `version`.
	///
	/// If the package doesn't exist, the function returns None.
	pub fn get_package(&self, name: &str, version: &Version) -> io::Result<Option<Package>> {
		let mut path = self.path.clone();
		path.push(name);
		path.push(version.to_string());

		Package::load(path)
	}

	/// Returns the list of packages with each versions in the repository.
	pub fn list_packages(&self) -> io::Result<Vec<Package>> {
		fs::read_dir(&self.path)?
			.filter_map(|ent| {
				let ent = ent.ok()?;
				if !ent.file_type().ok()?.is_dir() {
					return None;
				}

				let name = ent.file_name().to_str()?.to_owned();
				let ent_path = self.path.join(name);

				let iter = fs::read_dir(&ent_path)
					.ok()?
					.into_iter()
					.filter_map(|ent| {
						let ent = ent.ok()?;
						if !ent.file_type().ok()?.is_dir() {
							return None;
						}

						let ent_name = ent.file_name().to_str()?.to_owned();
						let version = Version::try_from(ent_name.as_ref()).ok()?;

						let ent_path = ent_path.join(version.to_string());
						Package::load(ent_path).transpose()
					})
					.collect::<Vec<_>>()
					.into_iter();

				Some(iter)
			})
			.flatten()
			.collect()
	}

	/// Returns the latest version of the package with name `name`.
	///
	/// If the package doesn't exist, the function returns None.
	pub fn get_latest_package(&self, name: &str) -> io::Result<Option<Package>> {
		let latest_version = fs::read_dir(self.path.join(name))?
			.into_iter()
			.filter_map(|ent| {
				let ent = ent.ok()?;

				if ent.file_type().ok()?.is_dir() {
					let name = ent.file_name();
					Version::try_from(name.to_str()?).ok()
				} else {
					None
				}
			})
			.max();

		match latest_version {
			Some(latest_version) => self.get_package(name, &latest_version),
			None => Ok(None),
		}
	}
}

// TODO Handle error reporting
/// Returns the package with name `name` and version `version` along with its associated
/// repository.
///
/// `repos` is the list of repositories to check on.
///
/// If the package doesn't exist, the function returns None.
pub fn get_package<'a>(
	repos: &'a [Repository],
	name: &str,
	version: &Version,
) -> io::Result<Option<(&'a Repository, Package)>> {
	Ok(repos.iter()
		.filter_map(|repo| {
			match repo.get_package(name, version) {
				Ok(Some(pack)) => Some((repo, pack)),
				_ => None,
			}
		})
		.next())
}

// TODO Handle error reporting
/// Returns the latest version of the package with name `name` along with its associated
/// repository.
///
/// `repos` is the list of repositories to check on.
///
/// If the package doesn't exist, the function returns None.
pub fn get_latest_package<'a>(
	repos: &'a [Repository],
	name: &str,
) -> io::Result<Option<(&'a Repository, Package)>> {
	Ok(repos.iter()
		.filter_map(|repo| {
			match repo.get_latest_package(name) {
				Ok(Some(pack)) => Some((repo, pack)),
				_ => None,
			}
		})
		.max_by(|(_, p0), (_, p1)| p0.get_version().cmp(p1.get_version())))
}
