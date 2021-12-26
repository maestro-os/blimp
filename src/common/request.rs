//! This module contains structures that represent JSON object used as response to HTTP requests.

use serde::Deserialize;
use serde::Serialize;

/// Structure representing the response to the request of the size of a package.
#[derive(Deserialize, Serialize)]
pub struct PackageSizeResponse {
    /// The size of the package in bytes.
    pub size: u64,
}
