//! The blimp library is the core of the Blimp package manager.

#![feature(io_error_more)]

pub use anyhow;
pub use serde_json;
pub use tokio;
pub use tokio_util;
pub use utils as maestro_utils;

#[cfg(feature = "network")]
pub mod download;
pub mod lockfile;
pub mod package;
pub mod repository;
pub mod util;
pub mod version;

use crate::version::Version;
use anyhow::Result;
use package::{InstalledPackage, Package};
use std::{
	error::Error,
	fs, io,
	io::ErrorKind,
	path::{Path, PathBuf},
};

/// The directory containing cached packages.
const LOCKFILE_PATH: &str = "var/lib/blimp/.lock";
/// The path to directory storing information about installed packages.
const INSTALLED_DB: &str = "var/lib/blimp/installed";

/// An environment is a system managed by the package manager.
///
/// On creation, the environment creates a lockfile to ensure no other instance can access it at
/// the same time.
///
/// The lockfile is destroyed when the environment is dropped.
pub struct Environment {
	/// The path to the sysroot of the environment.
	sysroot: PathBuf,
}

impl Environment {
	/// Returns an instance for the environment with the given sysroot.
	///
	/// The function tries to lock the environment so that no other instance can access it at the
	/// same time. If already locked, the function returns `None`.
	pub fn with_root(sysroot: &Path) -> io::Result<Option<Self>> {
		let sysroot = sysroot.canonicalize()?;
		let path = sysroot.join(LOCKFILE_PATH);
		let acquired = lockfile::lock(&path)?;
		Ok(acquired.then_some(Self {
			sysroot,
		}))
	}

	/// Returns the sysroot of the current environment.
	pub fn sysroot(&self) -> &Path {
		&self.sysroot
	}

	/// Returns the installed version for the package with the given `name`.
	pub fn get_installed_version(&self, _name: &str) -> Option<Version> {
		todo!()
	}

	/// Installs the given package.
	///
	/// Arguments:
	/// - `pkg` is the package to be installed.
	/// - `archive_path` is the path to the archive of the package.
	///
	/// The function does not resolve dependencies. It is the caller's responsibility to install
	/// them beforehand.
	pub fn install(&self, _pkg: &Package, archive_path: &Path) -> Result<(), Box<dyn Error>> {
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
		// TODO add package to installed db
		Ok(())
	}

	/// Installs a new version of the package, removing the previous.
	///
	/// Arguments:
	/// - `pkg` is the package to be updated.
	/// - `archive_path` is the path to the archive of the new version of the package.
	pub fn update(&self, _pkg: &Package, archive_path: &Path) -> Result<()> {
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
	pub fn remove(&self, pkg: &InstalledPackage) -> Result<()> {
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
		let path = self.sysroot.join(LOCKFILE_PATH);
		lockfile::unlock(&path).unwrap_or_else(|e| eprintln!("blimp: could not remove lockfile: {e}"));
	}
}
