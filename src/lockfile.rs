//! The lock file allows to prevent several instances of the package manager from running at the
//! same time.

use std::fs::OpenOptions;
use std::fs;

/// Blimp's path.
const BLIMP_PATH: &str = "/usr/lib/blimp";

/// The directory containing cached packages.
const LOCKFILE_PATH: &str = "/usr/lib/blimp/.lock";

/// Creates the lock file if not present. If the file was successfuly created, the function returns
/// `true`. Else, it returns `false`.
pub fn lock(sysroot: &str) -> bool {
	// Creating directories
    let blimp_dir = format!("{}/{}", sysroot, BLIMP_PATH);
    let _ = fs::create_dir_all(blimp_dir);

    let path = format!("{}/{}", sysroot, LOCKFILE_PATH);
    // Trying to create the file and failing if it already exist, allowing to avoid TOCTOU race
    // conditions
    OpenOptions::new().write(true).create_new(true).open(path).is_ok()
}

/// Removes the lock file.
pub fn unlock(sysroot: &str) {
    let path = format!("{}/{}", sysroot, LOCKFILE_PATH);
    let _ = fs::remove_file(path);
}
