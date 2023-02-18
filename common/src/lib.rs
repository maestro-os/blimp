//! This library contains common code between the client and the server.

#![feature(io_error_more)]

pub mod build;
pub mod lockfile;
pub mod package;
pub mod repository;
pub mod util;
pub mod version;

#[cfg(feature = "network")]
pub mod download;

use package::InstalledPackage;
use package::Package;
use repository::Repository;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::io;
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
	/// The function tries to lock the environment so that no other instance can access it at the same time.
	/// If already locked, the function returns `None`.
	pub fn with_root(sysroot: PathBuf) -> Option<Self> {
		let path = util::concat_paths(&sysroot, Path::new(LOCKFILE_PATH));

		if lockfile::lock(&path) {
			Some(Self {
				sysroot,
			})
		} else {
			None
		}
	}

	/// Returns the sysroot of the current environement.
	pub fn get_sysroot(&self) -> &Path {
		&self.sysroot
	}

	/// Loads and returns the list of all repositories.
	///
	/// Arguments:
	/// - `local_repos` is the list of paths of local repositories.
	pub fn list_repositories(&self, local_repos: &[PathBuf]) -> io::Result<Vec<Repository>> {
		let mut repos = vec![];

		// TODO List from blimp directory using sysroot

		let mut local_repos = local_repos.iter()
			.map(|path| Repository::load(path.clone()))
			.collect::<Result<Vec<_>, _>>()?;
		repos.append(&mut local_repos);

		Ok(repos)
	}

	/// Returns the list of installed packages.
	///
	/// The key is the name of the package and the value is the installed package.
	pub fn get_installed_list(&self) -> io::Result<HashMap<String, InstalledPackage>> {
		let path = util::concat_paths(&self.sysroot, Path::new(INSTALLED_FILE));

		match util::read_json::<HashMap<String, InstalledPackage>>(&path) {
			Ok(pkgs) => Ok(pkgs),

			Err(e) if e.kind() == ErrorKind::NotFound => Ok(HashMap::new()),

			Err(e) => Err(e),
		}
	}

	/// Updates the list of installed packages to the disk.
	pub fn update_installed_list(
		&self, list: &HashMap<String, InstalledPackage>
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
	/// If the package is already installed, the function does nothing.
	///
	/// The function does not resolve dependencies. It is the caller's responsibility to install
	/// them beforehand.
	pub fn install(&self, pkg: &Package, archive_path: &Path) -> Result<(), Box<dyn Error>> {
		let mut installed = self.get_installed_list()?;

		// If the package is installed, do nothing
		if installed.contains_key(pkg.get_name()) {
			return Ok(());
		}

		// Uncompressing the package
		let files = util::uncompress_wrap(archive_path, |tmp_dir| {
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

			let files = todo!(); // TODO

			let mut post_install_hook_path: PathBuf = tmp_dir.to_path_buf();
			post_install_hook_path.push("post-install-hook");
			if !util::run_hook(&post_install_hook_path, &self.sysroot)? {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					"Post-install hook failed!",
				));
			}

			Ok(files)
		})??;

		installed.insert(pkg.get_name().to_owned(), InstalledPackage {
			desc: pkg.clone(),

			files,
		});
		self.update_installed_list(&installed)?;

		Ok(())
	}

	/// Installs a new verion of the package, removing the previous.
	///
	/// Arguments:
	/// - `pkg` is the package to be updated.
	/// - `archive_path` is the path to the archive of the new version of the package.
	///
	/// If the package is not installed, the function does nothing.
	pub fn update(&self, pkg: &Package, archive_path: &Path) -> Result<(), Box<dyn Error>> {
		let mut installed = self.get_installed_list()?;

		// If the package is not installed, do nothing
		if !installed.contains_key(pkg.get_name()) {
			return Ok(());
		}

		// Uncompressing the package
		let files = util::uncompress_wrap(archive_path, |tmp_dir| {
			let mut pre_upgrade_hook_path: PathBuf = tmp_dir.to_path_buf();
			pre_upgrade_hook_path.push("pre-upgrade-hook");
			if !util::run_hook(&pre_upgrade_hook_path, &self.sysroot)? {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					"Pre-upgrade hook failed!",
				));
			}

			// TODO Patch files corresponding to the ones in inner data archive

			let files = todo!(); // TODO

			let mut post_upgrade_hook_path: PathBuf = tmp_dir.to_path_buf();
			post_upgrade_hook_path.push("post-upgrade-hook");
			if !util::run_hook(&post_upgrade_hook_path, &self.sysroot)? {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					"Post-upgrade hook failed!",
				));
			}

			Ok(files)
		})??;

		installed.insert(pkg.get_name().to_owned(), InstalledPackage {
			desc: pkg.clone(),

			files,
		});
		self.update_installed_list(&installed)?;

		Ok(())
	}

	/// Removes the given package.
	///
	/// This function does not check dependency breakage. It is the caller's responsibility to
	/// ensure no other package depend on the package to be removed.
	///
	/// If the package is not installed, the function does nothing.
	pub fn remove(&self, pkg: &InstalledPackage) -> Result<(), Box<dyn Error>> {
		let mut installed = self.get_installed_list()?;

		// If the package is not installed, do nothing
		if !installed.contains_key(pkg.desc.get_name()) {
			return Ok(());
		}

		// TODO must keep a copy at installation
		/*let mut pre_remove_hook_path: PathBuf = tmp_dir.to_path_buf();
		pre_remove_hook_path.push("pre-remove-hook");
		if !util::run_hook(&pre_remove_hook_path, &self.sysroot)? {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"Pre-remove hook failed!",
			));
		}*/

		// Remove the package's files
		// Removing is made in reverse order to ensure inner files are removed first
		let mut files = pkg.files.clone();
		files.sort_unstable_by(|a, b| a.cmp(b).reverse());
		for sys_path in &pkg.files {
			let path = util::concat_paths(&self.sysroot, &sys_path);

			let file_type = fs::metadata(&path)?.file_type();
			if file_type.is_dir() {
				match fs::remove_dir(&path) {
					Ok(_) => {},

					// If the directory is not empty, ignore error
					Err(e) if e.kind() == ErrorKind::DirectoryNotEmpty => {},

					Err(e) => return Err(e.into()),
				}
			} else {
				fs::remove_file(&path)?;
			}
		}

		// TODO must keep a copy at installation
		/*let mut post_remove_hook_path: PathBuf = tmp_dir.to_path_buf();
		post_remove_hook_path.push("post-remove-hook");
		if !util::run_hook(&post_remove_hook_path, &self.sysroot)? {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"Post-remove hook failed!",
			));
		}*/

		installed.remove(pkg.desc.get_name());
		self.update_installed_list(&installed)?;

		Ok(())
	}

	/// Unlocks the environment.
	pub fn unlock(self) {}
}

impl Drop for Environment {
	fn drop(&mut self) {
		let path = util::concat_paths(&self.sysroot, Path::new(LOCKFILE_PATH));
		lockfile::unlock(&path);
	}
}
