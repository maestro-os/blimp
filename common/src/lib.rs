//! This library contains common code between the client and the server.

pub mod build;
pub mod lockfile;
pub mod package;
pub mod repository;
pub mod util;
pub mod version;

#[cfg(feature = "network")]
pub mod download;

use package::Package;
use std::error::Error;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// The path to the file storing the list of installed packages.
const INSTALLED_FILE: &str = "/usr/lib/blimp/installed";

/// An instance of a Blimp environment.
pub struct Environment {
	/// The path to the sysroot of the environment.
	sysroot: PathBuf,
}

impl Environment {
	/// Returns an instance for the environment with the given sysroot.
	pub fn with_root(sysroot: PathBuf) -> Self {
		Self {
			sysroot,
		}
	}

	/// Inserts the current package in the list of installed packages.
	fn insert_installed(&self, package: &Package) -> io::Result<()> {
		let path = util::concat_paths(&self.sysroot, Path::new(INSTALLED_FILE));

		// Reading the file
		let mut packages: Vec<Package> = match util::read_json(&path) {
			Ok(packages) => packages,
			Err(_) => vec![],
		};

		packages.push(package.clone());

		// Writing the file
		util::write_json(&path, &packages)
	}

	/// Removes the current package from the list of installed packages.
	fn remove_installed(&self, name: &str) -> io::Result<()> {
		let path = util::concat_paths(&self.sysroot, Path::new(INSTALLED_FILE));

		// Reading the file
		let Ok(mut packages) = util::read_json::<Vec<Package>>(&path) else {
			return Ok(());
		};

		// Removing the entry
		let mut i = 0;
		while i < packages.len() {
			if packages[i].get_name() == name {
				packages.remove(i);
			}

			i += 1;
		}

		// Writing the file
		util::write_json(&path, &packages)
	}

	/// Returns the installed package with name `name`.
	///
	/// If the package isn't installed, the function returns None.
	pub fn get_installed(&self, name: &str) -> io::Result<Option<Package>> {
		let path = util::concat_paths(&self.sysroot, Path::new(INSTALLED_FILE));

		// Reading the file
		let Ok(packages) = util::read_json::<Vec<Package>>(&path) else {
			return Ok(None);
		};

		for p in packages {
			if p.get_name() == name {
				return Ok(Some(p));
			}
		}

		Ok(None)
	}

	/// Installs the given package.
	///
	/// Arguments:
	/// - `package` is the package to be installed.
	/// - `archive_path` is the path to the archive of the package.
	///
	/// If the package is already installed, the function does nothing.
	///
	/// TODO
	pub fn install(&self, package: &Package, archive_path: &Path) -> Result<(), Box<dyn Error>> {
		if self.get_installed(package.get_name())?.is_some() {
			return Ok(());
		}

		// Uncompressing the package
		util::uncompress_wrap(archive_path, |tmp_dir| {
			let mut pre_install_hook_path: PathBuf = tmp_dir.to_path_buf();
			pre_install_hook_path.push("pre-install-hook");
			if !util::run_hook(&pre_install_hook_path, &self.sysroot)? {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					"Pre-install hook failed!",
				));
			}

			// Installing the package's files
			let mut data_path = tmp_dir.to_path_buf();
			data_path.push("data");
			util::recursive_copy(&data_path, &self.sysroot)?;

			let mut post_install_hook_path: PathBuf = tmp_dir.to_path_buf();
			post_install_hook_path.push("post-install-hook");
			if !util::run_hook(&post_install_hook_path, &self.sysroot)? {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					"Post-install hook failed!",
				));
			}

			Ok(())
		})??;

		self.insert_installed(package)?;

		Ok(())
	}

	/// Installs a new verion of the package, removing the previous.
	///
	/// Arguments:
	/// - `package` is the package to be updated.
	/// - `archive_path` is the path to the archive of the new version of the package.
	pub fn update(&self, package: &Package, archive_path: &Path) -> Result<(), Box<dyn Error>> {
		// Uncompressing the package
		util::uncompress_wrap(archive_path, |tmp_dir| {
			let mut pre_upgrade_hook_path: PathBuf = tmp_dir.to_path_buf();
			pre_upgrade_hook_path.push("pre-upgrade-hook");
			if !util::run_hook(&pre_upgrade_hook_path, &self.sysroot)? {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					"Pre-upgrade hook failed!",
				));
			}

			// TODO Patch files corresponding to the ones in inner data archive

			let mut post_upgrade_hook_path: PathBuf = tmp_dir.to_path_buf();
			post_upgrade_hook_path.push("post-upgrade-hook");
			if !util::run_hook(&post_upgrade_hook_path, &self.sysroot)? {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					"Post-upgrade hook failed!",
				));
			}

			Ok(())
		})??;

		self.remove_installed(package.get_name())?;
		self.insert_installed(package)?;

		Ok(())
	}

	/// Removes the package with the given name.
	///
	/// Arguments:
	/// - `name` is the name of the package.
	/// - `archive_path` is the path to the archive of the new version of the package.
	pub fn remove(&self, name: &str, archive_path: &Path) -> Result<(), Box<dyn Error>> {
		// Uncompressing the package
		util::uncompress_wrap(archive_path, |tmp_dir| {
			let mut pre_remove_hook_path: PathBuf = tmp_dir.to_path_buf();
			pre_remove_hook_path.push("pre-remove-hook");
			if !util::run_hook(&pre_remove_hook_path, &self.sysroot)? {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					"Pre-remove hook failed!",
				));
			}

			// TODO Remove files corresponding to the ones in inner data archive

			let mut post_remove_hook_path: PathBuf = tmp_dir.to_path_buf();
			post_remove_hook_path.push("post-remove-hook");
			if !util::run_hook(&post_remove_hook_path, &self.sysroot)? {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					"Post-remove hook failed!",
				));
			}

			Ok(())
		})?;

		self.remove_installed(name)?;

		Ok(())
	}
}
