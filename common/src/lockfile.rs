//! The lock file allows to prevent several instances of the package manager from running at the
//! same time.

use crate::util;
use std::error::Error;
use std::fs::OpenOptions;
use std::fs;
use std::path::Path;

/// The directory containing cached packages.
const LOCKFILE_PATH: &str = "/usr/lib/blimp/.lock";

/// Creates the lock file if not present.
///
/// `path` is the path to the lockfile.
///
/// If the file was successfuly created, the function returns `true`. Else, it returns `false`.
pub fn lock(path: &Path) -> bool {
	// Trying to create the file and failing if it already exist, preventing TOCTOU race conditions
	OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(path)
		.is_ok()
}

/// Removes the lock file.
///
/// `path` is the path to the lockfile.
pub fn unlock(path: &Path) {
	let _ = fs::remove_file(path);
}

/// Executes the given closure `f` while locking.
///
/// `sysroot` is the system's root.
///
/// If the lock cannot be aquired, the function returns an error.
pub fn lock_wrap<T, F: FnOnce() -> T>(f: F, sysroot: &Path) -> Result<T, Box<dyn Error>> {
	let path = util::concat_paths(sysroot, Path::new(LOCKFILE_PATH));

	if !lock(&path) {
		return Err(format!("failed to acquire lockfile {}", path.display()).into());
	}

	let result = f();

	unlock(&path);

	Ok(result)
}
