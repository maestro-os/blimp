//! This module implements the build descriptor structure.

use common::package::Package;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use tokio::runtime::Runtime;

/// Structure representing the location of sources and where to unpack them.
#[derive(Deserialize, Serialize)]
pub struct Source {
	/// The location relative to the build directory where the archive will be unpacked.
	location: String,

	/// The URL of the sources.
	url: String,
}

impl Source {
	// TODO Do not keep the whole file in RAM before writing
	/// Fetches the file from the URL.
	pub async fn fetch(&self) -> Result<(), String> {
		let response = reqwest::get(&self.url).await
			.or_else(| e | Err(format!("HTTP request failed: {}", e)))?;
		let content = response.bytes().await
			.or_else(| e | Err(format!("HTTP request failed: {}", e)))?;

		let filename = "TODO"; // TODO
		let mut file = File::create(filename).or_else(| e | {
			Err(format!("Failed to create file: {}", e))
		})?;
		file.write(&content).or_else(| e | Err(format!("IO error: {}", e)))?;

		Ok(())
	}
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
	pub fn fetch_all(&self) -> Result<(), String> {
		// Creating the async runtime
		let rt = Runtime::new().unwrap();
		let mut futures = Vec::new();

		for s in &self.sources {
			futures.push(s.fetch());
		}
		for f in futures {
			rt.block_on(f)?;
		}

		Ok(())
	}

	/// Returns a reference to the package descriptor.
	pub fn get_package(&self) -> &Package {
		&self.package
	}
}
