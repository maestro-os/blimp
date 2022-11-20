//! TODO doc

use crate::package::Package;
use crate::util;
use std::error::Error;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// The path to the file storing the list of installed packages.
const INSTALLED_FILE: &str = "/usr/lib/blimp/installed";

/// Installs the package. If the package is already installed, the function does nothing.
///
/// Arguments:
/// - `sysroot` is the path to the system's root.
/// - `package` is the package to be installed.
/// - `archive_path` is the path to the package's archive.
///
/// The function assumes the running dependencies of the package are already installed.
pub fn install(
	sysroot: &Path,
	package: &Package,
	archive_path: &Path,
) -> Result<(), Box<dyn Error>> {
	if is_installed(sysroot, package.get_name()) {
		return Ok(());
	}

	// Uncompressing the package
	util::uncompress_wrap(archive_path, |tmp_dir| {
		let mut pre_install_hook_path: PathBuf = tmp_dir.to_path_buf();
		pre_install_hook_path.push("pre-install-hook");
		if !util::run_hook(&pre_install_hook_path, &sysroot)? {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"Pre-install hook failed!",
			));
		}

		// Installing the package's files
		let mut data_path = tmp_dir.to_path_buf();
		data_path.push("data");
		util::recursive_copy(&data_path, &sysroot)?;

		let mut post_install_hook_path: PathBuf = tmp_dir.to_path_buf();
		post_install_hook_path.push("post-install-hook");
		if !util::run_hook(&post_install_hook_path, &sysroot)? {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"Post-install hook failed!",
			));
		}

		Ok(())
	})??;

	insert_installed(sysroot, package)?;

	Ok(())
}

/// Upgrades the package.
///
/// Arguments:
/// - `sysroot` is the path to the system's root.
/// - `package` is the package to be installed.
/// - `archive_path` is the path to the package's archive.
pub fn upgrade(
	sysroot: &Path,
	package: &Package,
	archive_path: &Path,
) -> Result<(), Box<dyn Error>> {
	// Uncompressing the package
	util::uncompress_wrap(archive_path, |tmp_dir| {
		let mut pre_upgrade_hook_path: PathBuf = tmp_dir.to_path_buf();
		pre_upgrade_hook_path.push("pre-upgrade-hook");
		if !util::run_hook(&pre_upgrade_hook_path, &sysroot)? {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"Pre-upgrade hook failed!",
			));
		}

		// TODO Patch files corresponding to the ones in inner data archive

		let mut post_upgrade_hook_path: PathBuf = tmp_dir.to_path_buf();
		post_upgrade_hook_path.push("post-upgrade-hook");
		if !util::run_hook(&post_upgrade_hook_path, &sysroot)? {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"Post-upgrade hook failed!",
			));
		}

		Ok(())
	})??;

	remove_installed(sysroot, package.get_name())?;
	insert_installed(sysroot, package)?;

	Ok(())
}

/// Removes the package.
///
/// Arguments:
/// - `sysroot` is the path to the system's root.
/// - `package` is the package to be installed.
/// - `archive_path` is the path to the package's archive.
pub fn remove(
	sysroot: &Path,
	package: &Package,
	archive_path: &Path,
) -> Result<(), Box<dyn Error>> {
	// Uncompressing the package
	util::uncompress_wrap(archive_path, |tmp_dir| {
		let mut pre_remove_hook_path: PathBuf = tmp_dir.to_path_buf();
		pre_remove_hook_path.push("pre-remove-hook");
		if !util::run_hook(&pre_remove_hook_path, &sysroot)? {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"Pre-remove hook failed!",
			));
		}

		// TODO Remove files corresponding to the ones in inner data archive

		let mut post_remove_hook_path: PathBuf = tmp_dir.to_path_buf();
		post_remove_hook_path.push("post-remove-hook");
		if !util::run_hook(&post_remove_hook_path, &sysroot)? {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"Post-remove hook failed!",
			));
		}

		Ok(())
	})??;

	remove_installed(sysroot, package.get_name())?;

	Ok(())
}

/// Returns the installed package with name `name`.
///
/// `sysroot` is the path to the system's root.
///
/// If the package isn't installed, the function returns None.
pub fn get_installed(sysroot: &Path, name: &str) -> io::Result<Option<Package>> {
	let mut path = sysroot.to_path_buf();
	path.push(INSTALLED_FILE);

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

/// Tells whether the package with name `name` is installed on the system.
///
/// `sysroot` is the path to the system's root.
///
/// This function doesn't check if the version of the package is the same.
pub fn is_installed(sysroot: &Path, name: &str) -> bool {
	get_installed(sysroot, name)
		.unwrap_or(None)
		.is_some()
}

/// Inserts the current package in the list of installed packages.
pub fn insert_installed(sysroot: &Path, package: &Package) -> io::Result<()> {
	let mut path = sysroot.to_path_buf();
	path.push(INSTALLED_FILE);

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
pub fn remove_installed(sysroot: &Path, name: &str) -> io::Result<()> {
	let mut path = sysroot.to_path_buf();
	path.push(INSTALLED_FILE);

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
