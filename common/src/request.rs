//! This module contains structures that represent JSON object used as response to HTTP requests.

use crate::package::Package;
use serde::Deserialize;
use serde::Serialize;

/// Structure representing the response to the request of all packages present on the server.
#[derive(Deserialize, Serialize)]
pub struct PackageListResponse {
	/// The list of packages on the remote.
	pub packages: Vec<Package>,
}

/// Structure representing the response to the request of the size of a package.
#[derive(Deserialize, Serialize)]
pub struct PackageSizeResponse {
	/// The size of the package in bytes.
	pub size: u64,
}
