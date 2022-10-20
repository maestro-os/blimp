//! A repository contains packages that can be installed. It can either be remote or local.

pub mod local;
pub mod remote;

use crate::package::Package;
use local::LocalRepository;
use remote::Remote;
use std::io;

/// Trait representing a repository.
pub enum Repository {
	/// A local repository.
	Local(LocalRepository),
	/// A remote repository.
	Remote(Remote),
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
			.map(|path| Self::Local(LocalRepository::new(path.to_string())));
		repos.extend(iter);

		let iter = Remote::load_list(sysroot)?
			.into_iter()
			.map(|r| Self::Remote(r));
		repos.extend(iter);

		Ok(repos)
	}
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
