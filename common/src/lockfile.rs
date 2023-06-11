//! The lock file allows to prevent several instances of the package manager from running at the
//! same time.

use std::fs;
use std::fs::OpenOptions;
use std::path::Path;

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
