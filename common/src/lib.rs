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

//! The blimp library is the core of the Blimp package manager.

pub use anyhow;
pub use flate2;
pub use tar;
pub use tokio;
pub use tokio_util;
pub use utils as maestro_utils;

#[cfg(feature = "network")]
pub mod download;
pub mod lock;
pub mod package;
pub mod repository;
pub mod util;
pub mod version;

use crate::{
	repository::{remote::Remote, PackagesWithRepositoryVec, Repository},
	version::Version,
};
use anyhow::{bail, Result};
use package::{InstalledPackage, Package};
use std::{
	env,
	error::Error,
	fs,
	io::{self, ErrorKind},
	path::{Path, PathBuf},
};

/// Instance lock file
const LOCK_PATH: &str = "var/lib/blimp/.lock";
/// Directory storing information about installed packages
const INSTALLED_DB: &str = "var/lib/blimp/installed";
/// The file which contains the list of remotes
const REMOTES_LIST: &str = "var/lib/blimp/remotes-list";
/// Directory containing remote repositories
const REMOTES: &str = "var/lib/blimp/remotes";

/// The user agent for HTTP requests.
pub const USER_AGENT: &str = concat!("blimp/", env!("CARGO_PKG_VERSION"));

/// An environment is a system managed by the package manager.
///
/// On creation, the environment creates a lockfile to ensure no other instance can access it at
/// the same time.
///
/// The lockfile is destroyed when the environment is dropped.
pub struct Environment {
	/// The path to the sysroot of the environment
	sysroot: PathBuf,
	/// Local repositories, if any
	local_repos: Vec<PathBuf>,
	/// The architecture to install for
	arch: String,
}

impl Environment {
	/// Tries to lock the environment at `sysroot` so that no other instance can access it at the
	/// same time.
	///
	/// Note: the function gets the list of local repositories from the `LOCAL_REPO` environment
	/// variable, if set. The variable should contain a colon-separated list of paths to local
	/// repositories.
	///
	/// Arguments:
	/// - `sysroot` is the root directory of the system to lock
	/// - `arch` is the architecture to use
	///
	/// If the environment is already locked, the function returns `None`.
	pub fn acquire(sysroot: &Path, arch: &str) -> io::Result<Option<Self>> {
		let sysroot = sysroot.canonicalize()?;
		let path = sysroot.join(LOCK_PATH);
		let acquired = lock::lock(&path)?;
		let local_repos = env::var("LOCAL_REPO") // TODO var_os
			.map(|s| s.split(':').map(PathBuf::from).collect())
			.unwrap_or_default();
		Ok(acquired.then(|| Self {
			sysroot,
			local_repos,
			arch: arch.to_owned(),
		}))
	}

	/// Returns the sysroot of the current environment.
	#[inline]
	pub fn sysroot(&self) -> &Path {
		&self.sysroot
	}

	// TODO check if used in an other repo
	/// Returns the local repositories list
	#[inline]
	pub fn local_repos(&self) -> &[PathBuf] {
		&self.local_repos
	}

	/// Returns the repository architecture to use
	#[inline]
	pub fn arch(&self) -> &str {
		&self.arch
	}

	/// List local & remote repositories
	pub fn list_repositories(&self) -> Result<Vec<Repository>> {
		let repos = self
			.local_repos()
			.iter()
			.cloned()
			.map(Repository::local)
			// Add remote repositories
			.chain(
				Remote::load_list(self)?
					.iter()
					.map(|r| r.load_repository(self).unwrap()),
			) // TODO handle error
			.collect::<Vec<_>>();
		Ok(repos)
	}

	/// If installed, returns the version of the package with the given `name`
	pub fn get_installed_version(&self, name: &str) -> Result<Option<Version>> {
		// Ensure the parent directory exists
		let path = self.sysroot.join(INSTALLED_DB);
		fs::create_dir_all(&path)?;
		// Read file and get version
		let path = path.join(name);
		let res = fs::read_to_string(path);
		let installed = match res {
			Ok(i) => i,
			Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
			Err(e) => return Err(e.into()),
		};
		let installed: InstalledPackage = toml::from_str(&installed)?;
		Ok(Some(installed.desc.version))
	}

	/// Writes installed package information
	fn write_installed_version(&self, pkg: &InstalledPackage) -> Result<()> {
		// Ensure the parent directory exists
		let path = self.sysroot.join(INSTALLED_DB);
		fs::create_dir_all(&path)?;
		// Write
		let path = path.join(&pkg.desc.name);
		let content = toml::to_string(pkg)?;
		fs::write(path, content)?;
		Ok(())
	}

	/// Installs the given package.
	///
	/// Arguments:
	/// - `pkg` is the package to be installed
	/// - `archive_path` is the path to the archive of the package
	///
	/// The function does not resolve dependencies. It is the caller's responsibility to install
	/// them beforehand.
	pub fn install(&mut self, pkg: &Package, archive_path: &Path) -> Result<(), Box<dyn Error>> {
		// Read archive
		let mut archive = util::read_package_archive(archive_path)?;
		// TODO Get hooks (pre-install-hook and post-install-hook)
		// TODO Execute pre-install-hook
		// The list of installed files
		let mut files = vec![];
		// Copy files
		for e in archive.entries()? {
			let mut e = e?;
			let path = e.path()?;
			// Exclude files outside the `data` directory
			let Ok(path) = path.strip_prefix("data/") else {
				continue;
			};
			let dst = self.sysroot.join(path);
			// Create parent directories
			if let Some(parent) = dst.parent() {
				fs::create_dir_all(parent)?;
			}
			let path = path.to_path_buf();
			e.unpack(dst)?;
			files.push(path);
		}
		// TODO Execute post-install-hook
		self.write_installed_version(&InstalledPackage {
			desc: pkg.clone(),
			files,
		})?;
		Ok(())
	}

	/// Install all packages into environment.
	///
	/// Arguments:
	/// - `total_packages` is the whole list of packages to install
	pub fn install_packages<'r>(
		&mut self,
		total_packages: &PackagesWithRepositoryVec<'r>,
	) -> Result<()> {
		let mut failed = false;
		for (pkg, repo) in total_packages {
			println!("Installing `{}`...", pkg.name);
			let archive_path = repo.get_archive_path(self.arch(), &pkg.name, &pkg.version);
			if let Err(e) = self.install(pkg, &archive_path) {
				eprintln!("Failed to install `{}`: {e}", &pkg.name);
				failed = true;
			}
		}
		if failed {
			bail!("installation failed");
		}
		Ok(())
	}

	/// Installs a new version of the package, removing the previous.
	///
	/// Arguments:
	/// - `pkg` is the package to be updated
	/// - `archive_path` is the path to the archive of the new version of the package
	pub fn upgrade(&mut self, _pkg: &Package, archive_path: &Path) -> Result<()> {
		// Read archive
		let _archive = util::read_package_archive(archive_path)?;
		// TODO Get hooks (pre-update-hook and post-update-hook)
		// TODO Execute pre-update-hook
		// TODO Patch files corresponding to the ones in inner data archive
		// TODO Execute post-update-hook
		// TODO update package in installed db
		Ok(())
	}

	/// Removes the given package.
	///
	/// This function does not check dependency breakage. It is the caller's responsibility to
	/// ensure no other package depend on the package to be removed.
	pub fn remove(&mut self, pkg: &InstalledPackage) -> Result<()> {
		// TODO Get hooks (pre-remove-hook and post-remove-hook. Copy at installation?)
		// TODO Execute pre-remove-hook
		// Remove the package's files
		// Removing is made in reverse order to ensure inner files are removed first
		let mut files = pkg.files.clone();
		files.sort_unstable_by(|a, b| a.cmp(b).reverse());
		for sys_path in &files {
			let path = util::concat_paths(&self.sysroot, sys_path);
			let dir = fs::metadata(&path)
				.map(|m| m.file_type().is_dir())
				.unwrap_or(false);
			let result = if dir {
				fs::remove_dir(&path)
			} else {
				fs::remove_file(&path)
			};
			match result {
				Ok(_) => {}
				Err(e)
					if matches!(e.kind(), ErrorKind::DirectoryNotEmpty | ErrorKind::NotFound) => {}
				Err(e) => return Err(e.into()),
			}
		}
		// TODO Execute post-remove-hook
		// TODO remove package from installed db
		Ok(())
	}
}

impl Drop for Environment {
	fn drop(&mut self) {
		let path = self.sysroot.join(LOCK_PATH);
		lock::unlock(&path).unwrap_or_else(|e| eprintln!("blimp: could not remove lockfile: {e}"));
	}
}
