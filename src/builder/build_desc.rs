//! This module implements the build descriptor structure.

use common::package::Package;
use common::util;
use serde::Deserialize;
use serde::Serialize;
use std::io::Write;

/// Structure representing the location of sources and where to unpack them.
#[derive(Deserialize, Serialize)]
pub struct Source {
	/// The location relative to the build directory where the archive will be unpacked.
	location: String,

	/// The URL of the sources.
	url: String,
}

impl Source {
	/// Returns the source's URL.
	pub fn get_url(&self) -> &String {
		&self.url
	}

	/// Fetches the file from the URL and uncompresses it into the build directory `build_dir`.
	pub fn fetch(&self, build_dir: &str) -> Result<(), String> {
		// TODO Do not keep the whole file in RAM before writing
		/*let client = reqwest::blocking::Client::builder()
			.connect_timeout(None)
			.pool_idle_timeout(None)
			.timeout(None)
			.build()
			.or_else(| e | Err(format!("HTTP request failed: {}", e)))?;
		let response = client.get(&self.url).send()
			.or_else(| e | Err(format!("HTTP request failed: {}", e)))?;
		let content = response.bytes()
			.or_else(| e | Err(format!("HTTP request failed: {}", e)))?;

		let (path, mut file) = util::create_tmp_file().or_else(| e | {
			Err(format!("Failed to create file: {}", e))
		})?;
		file.write(&content).or_else(| e | Err(format!("IO error: {}", e)))?;*/

 	 	// TODO Find a cleaner solution
		let (path, _) = util::create_tmp_file().or_else(| e | {
			Err(format!("Failed to create file: {}", e))
		})?;
		let _ = std::process::Command::new("wget")
			.args(["-O", &path, &self.url])
			.status();

		// Uncompressing the archive
		util::uncompress(&path, &build_dir)
			.or_else(| e | Err(format!("Failed to uncompress archive: {}", e)))?;

		// TODO Root directory unwrapping

    	// TODO Remove the archive?

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
	/// Returns a reference to the list of sources.
	pub fn get_sources(&self) -> &Vec<Source> {
		&self.sources
	}

	/// Returns a reference to the package descriptor.
	pub fn get_package(&self) -> &Package {
		&self.package
	}
}
