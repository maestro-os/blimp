/*
 * Copyright 2026 Luc Lenôtre
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

use std::{error::Error, fs, io::ErrorKind, os::unix, path::Path};

/// Creates the FHS folder hierarchy on the disk.
///
/// - `sysroot` is the path to the FHS system root.
/// - `log` is whether to print on creation or not.
pub fn create_dirs(sysroot: &Path, log: bool) -> Result<(), Box<dyn Error>> {
	let dirs = &[
		"boot",
		"dev",
		"etc",
		"home",
		"media",
		"mnt",
		"opt",
		"proc",
		"root",
		"run",
		"srv",
		"sys",
		"tmp",
		"usr",
		"var",
		"etc/opt",
		"etc/sysconfig",
		"run/lock",
		"run/log",
		"usr/bin",
		"usr/include",
		"usr/lib",
		"usr/lib/firmware",
		"usr/local",
		"usr/sbin",
		"usr/share",
		"usr/src",
		"usr/share/doc",
		"usr/share/info",
		"usr/share/locale",
		"usr/share/man",
		"usr/share/misc",
		"usr/local/bin",
		"usr/local/include",
		"usr/local/lib",
		"usr/local/sbin",
		"usr/local/share",
		"usr/local/src",
		"usr/local/share/doc",
		"usr/local/share/info",
		"usr/local/share/locale",
		"usr/local/share/man",
		"usr/local/share/misc",
		"var/cache",
		"var/lib",
		"var/local",
		"var/log",
		"var/mail",
		"var/opt",
		"var/spool",
		"var/lib/misc",
	];
	for path in dirs {
		if log {
			println!("Create directory `{path}`");
		}
		let path = sysroot.join(path);
		match fs::create_dir(path) {
			Ok(_) => {}
			Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
			Err(e) => return Err(e.into()),
		}
	}
	let links: &[(&str, &str)] = &[
		("usr/bin", "bin"),
		("usr/sbin", "sbin"),
		("usr/lib", "lib"),
		("usr/lib", "lib64"),
		("lib", "usr/lib64"),
	];
	for (target, link) in links {
		if log {
			println!("Create symlink `{link}` -> `{target}`");
		}
		let path = sysroot.join(link);
		match unix::fs::symlink(target, &path) {
			Ok(_) => {}
			Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
			Err(e) => return Err(e.into()),
		}
	}
	Ok(())
}
