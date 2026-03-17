/*
 * Copyright 2025 Luc Lenôtre
 *
 * This file is part of Maestro.
 *
 * Maestro is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Maestro is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Maestro. If not, see <https://www.gnu.org/licenses/>.
 */

//! The instance lock file allows to prevent several instances of the package manager from running
//! at the same time.

use std::{fs, fs::OpenOptions, io, path::Path};

/// Creates the instance lock file if not present.
///
/// `path` is the path to the file.
///
/// If the file was successfully created, the function returns `true`. Else, it returns `false`.
pub fn lock(path: &Path) -> io::Result<bool> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}
	// Try to create the file and failing if it already exists, preventing TOCTOU race
	// conditions
	let acquired = OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(path)
		.is_ok();
	Ok(acquired)
}

/// Removes the instance lock file.
///
/// `path` is the path to the file.
pub fn unlock(path: &Path) -> io::Result<()> {
	fs::remove_file(path)
}
