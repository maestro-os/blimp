//! Utilities.

use anyhow::{anyhow, Result};
use core::str;
use std::ffi::OsStr;
use std::num::NonZeroUsize;
use std::process::Command;
use std::{env, io, thread};

/// Default host triplet in case no one can be retrieved.
const DEFAULT_TRIPLET: &str = "x86_64-linux-gnu";

/// Returns the recommended amount of CPUs to build the package.
pub fn get_jobs_count() -> Result<usize> {
	match env::var("JOBS") {
		Ok(s) => s.parse().map_err(|_| anyhow!("Invalid jobs count: {s}")),
		// Not specified by the user: get the amount of CPU on the system
		Err(_) => Ok(thread::available_parallelism()
			.map(NonZeroUsize::get)
			.unwrap_or(1)),
	}
}

/// Retrieves the host triplet from the compiler.
fn get_host_triplet_from_cc() -> io::Result<Option<String>> {
	let cc = env::var_os("CC");
	let cc = cc.as_deref().unwrap_or(OsStr::new("cc"));
	let output = Command::new(cc).arg("-dumpmachine").output()?;
	let Ok(triplet) = str::from_utf8(&output.stdout) else {
		return Ok(None);
	};
	Ok(Some(triplet.trim().to_owned()))
}

/// Returns the triplet of the host on which the package is to be built.
pub fn get_host_triplet() -> io::Result<String> {
	if let Ok(triplet) = env::var("HOST") {
		return Ok(triplet);
	}
	if let Some(triplet) = get_host_triplet_from_cc()? {
		return Ok(triplet);
	}
	eprintln!("Failed to retrieve host triplet. Defaulting to {DEFAULT_TRIPLET}.");
	Ok(DEFAULT_TRIPLET.to_owned())
}
