//! The lock file allows to prevent several instances of the package manager from running at the
//! same time.

use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;

/// Creates the lock file if not present.
///
/// `path` is the path to the lockfile.
///
/// If the file was successfully created, the function returns `true`. Else, it returns `false`.
pub fn lock(path: &Path) -> io::Result<bool> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}
	// Trying to create the file and failing if it already exists, preventing TOCTOU race conditions
	let acquired = OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(path)
		.is_ok();
	Ok(acquired)
}

/// Removes the lock file.
///
/// `path` is the path to the lockfile.
pub fn unlock(path: &Path) -> io::Result<()> {
	fs::remove_file(path)
}
