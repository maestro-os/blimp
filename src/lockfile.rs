//! The lock file allows to prevent several instances of the package manager from running at the
//! same time.

use std::fs::OpenOptions;
use std::fs;

/// The directory containing cached packages.
const LOCKFILE_PATH: &str = "/usr/lib/blimp/.lock";

/// Creates the lock file if not present. If the file was successfuly created, the function returns
/// `true`. Else, it returns `false`.
pub fn lock() -> bool {
    // Trying to create the file and failing if it already exist, allowing to avoid TOCTOU race
    // conditions
    OpenOptions::new().write(true).create_new(true).open(LOCKFILE_PATH).is_ok()
}

/// Removes the lock file.
pub fn unlock() {
    let _ = fs::remove_file(LOCKFILE_PATH);
}
