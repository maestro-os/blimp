//! This module implements the build descriptor structure.

use common::package::Package;
use serde::Deserialize;
use serde::Serialize;
use std::io;

/// Structure representing the location of sources and where to unpack them.
#[derive(Deserialize, Serialize)]
pub struct Source {
	/// The location relative to the build directory where the archive will be unpacked.
	location: String,

	/// The URL of the sources.
	url: String,
}

/// Structure describing how to build a package.
#[derive(Deserialize, Serialize)]
pub struct BuildDescriptor {
	/// The list of sources for the package.
	sources: Vec<Source>,

	/// The package's descriptor.
	package: Package,
}

impl BuildDescriptor {
	/// Fetches all the sources.
	pub fn fetch_all(&self) -> io::Result<()> {
		// TODO
		todo!();
	}

	/// Returns a reference to the package descriptor.
	pub fn get_package(&self) -> &Package {
		&self.package
	}
}
