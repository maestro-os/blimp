//! The blimp library is the core of the Blimp package manager.

#![feature(io_error_more)]

pub mod lockfile;
pub mod package;
pub mod repository;
pub mod util;
pub mod version;

#[cfg(feature = "network")]
pub mod download;

use anyhow::Result;
use package::InstalledPackage;
use package::Package;
use repository::Repository;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

/// The directory containing cached packages.
const LOCKFILE_PATH: &str = "/usr/lib/blimp/.lock";
/// The path to the file storing the list of installed packages.
const INSTALLED_FILE: &str = "/usr/lib/blimp/installed";

/// An instance of a Blimp environment.
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
	pub fn with_root(sysroot: PathBuf) -> io::Result<Option<Self>> {
		let path = util::concat_paths(&sysroot, Path::new(LOCKFILE_PATH));
		let acquired = lockfile::lock(&path)?;
		Ok(acquired.then_some(Self {
			sysroot,
		}))
	}

	/// Returns the sysroot of the current environement.
	pub fn get_sysroot(&self) -> &Path {
		&self.sysroot
	}

	/// Loads and returns the list of all repositories.
	///
	/// `local_repos` is the list of paths to local repositories.
	pub fn list_repositories(&self, local_repos: &[PathBuf]) -> io::Result<Vec<Repository>> {
		// TODO Add blimp's inner repositories (local representations of remotes)
		local_repos
			.iter()
			.map(|path| Repository::load(path.clone()))
			.collect::<Result<Vec<_>, _>>()
	}

	/// Loads the list of installed packages.
	///
	/// The key is the name of the package and the value is the installed package.
	pub fn load_installed_list(&self) -> io::Result<HashMap<String, InstalledPackage>> {
		let path = util::concat_paths(&self.sysroot, Path::new(INSTALLED_FILE));

		match util::read_json::<HashMap<String, InstalledPackage>>(&path) {
			Ok(pkgs) => Ok(pkgs),
			Err(e) if e.kind() == ErrorKind::NotFound => Ok(HashMap::new()),
			Err(e) => Err(e),
		}
	}

	/// Updates the list of installed packages to the disk.
	pub fn update_installed_list(
		&self,
		list: &HashMap<String, InstalledPackage>,
	) -> io::Result<()> {
		let path = util::concat_paths(&self.sysroot, Path::new(INSTALLED_FILE));
		util::write_json(&path, list)
	}

	/// Installs the given package.
	///
	/// Arguments:
	/// - `pkg` is the package to be installed.
	/// - `archive_path` is the path to the archive of the package.
	///
	/// The function does not resolve dependencies. It is the caller's responsibility to install
	/// them beforehand.
	pub fn install(&self, pkg: &Package, archive_path: &Path) -> Result<(), Box<dyn Error>> {
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
			let mut path_iter = path.iter();

			// Exclude files that are not in the `data` directory
			if path_iter.next() != Some(OsStr::new("data")) {
				continue;
			}

			let sys_path = path_iter.filter(|c| !c.is_empty()).collect::<PathBuf>();
			if sys_path.components().count() == 0 {
				continue;
			}

			let dst = self.sysroot.join(&sys_path);

			// Create parent directories
			if let Some(parent) = dst.parent() {
				fs::create_dir_all(parent)?;
			}

			e.unpack(dst)?;
			files.push(sys_path);
		}

		// TODO Execute post-install-hook

		// Update installed list
		let mut installed = self.load_installed_list()?;
		installed.insert(
			pkg.get_name().to_owned(),
			InstalledPackage {
				desc: pkg.clone(),
				files,
			},
		);
		self.update_installed_list(&installed)?;

		Ok(())
	}

	/// Installs a new verion of the package, removing the previous.
	///
	/// Arguments:
	/// - `pkg` is the package to be updated.
	/// - `archive_path` is the path to the archive of the new version of the package.
	pub fn update(&self, pkg: &Package, archive_path: &Path) -> Result<()> {
		// Read archive
		let _archive = util::read_package_archive(archive_path)?;

		// TODO Get hooks (pre-update-hook and post-update-hook)

		// TODO Execute pre-update-hook

		// The list of installed files
		let files = vec![];

		// TODO Patch files corresponding to the ones in inner data archive

		// TODO Execute post-update-hook

		// Update installed list
		let mut installed = self.load_installed_list()?;
		installed.insert(
			pkg.get_name().to_owned(),
			InstalledPackage {
				desc: pkg.clone(),
				files,
			},
		);
		self.update_installed_list(&installed)?;

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

		// Update installed list
		let mut installed = self.load_installed_list()?;
		installed.remove(pkg.desc.get_name());
		self.update_installed_list(&installed)?;

		Ok(())
	}
}

impl Drop for Environment {
	fn drop(&mut self) {
		let path = util::concat_paths(&self.sysroot, Path::new(LOCKFILE_PATH));
		let _ = lockfile::unlock(&path);
	}
}
