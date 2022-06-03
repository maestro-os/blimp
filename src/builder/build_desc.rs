//! This module implements the build descriptor structure.

use common::download;
use common::package::Package;
use common::util;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::process::Command;

/// Structure representing the location of sources and where to unpack them.
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Source {
	/// Downloading a tarball from an URL.
	Url {
		/// The location relative to the build directory where the archive will be unpacked.
		location: String,

		/// The URL of the sources.
		url: String,

		/// If true, the builder unwraps the package, meaning that if the tarball contains a single
		/// directory, its content is taken instead of the directory itself.
		unwrap: bool,
	},

	/// Cloning the given repository.
	Git {
		/// The location relative to the build directory where the archive will be unpacked.
		location: String,

		/// The URL to the Git repository.
		git_url: String,
	},

	/// Copying from a local path.
	Local {
		/// The location relative to the build directory where the archive will be unpacked.
		location: String,

		/// The path to the local tarball or directory.
		path: String,

		/// If true, the builder unwraps the package, meaning that if the tarball contains a single
		/// directory, its content is taken instead of the directory itself.
		unwrap: bool,
	},
}

impl Source {
	/// Fetches files from the source and uncompresses them if necessary.
	/// Files are placed into the build directory `build_dir` according to the location.
	pub async fn fetch(&self, build_dir: &str) -> Result<(), Box<dyn Error>> {
		match self {
			Self::Url {
				location,

				url,

				unwrap,
			} => {
				println!("Fetching `{}`...", url);

				let (path, _) = util::create_tmp_file()?;
				download::download_file(url, &path).await?;

				let dest_path = format!("{}/{}", build_dir, location);
				// Uncompressing the archive
				util::uncompress(&path, &dest_path, *unwrap)?;
			},

			Self::Git {
				location,

				git_url,
			} => {
				println!("Cloning `{}`...", git_url);

				let dest_path = format!("{}/{}", build_dir, location);
				let status = Command::new("git")
					.args(["clone", git_url, &dest_path])
					.status()?;

				if !status.success() {
					return Err(format!("Cloning `{}` failed", git_url).into());
				}
			},

			Self::Local {
				location,

				path,

				unwrap,
			} => {
				println!("Copying `{}`...", path);

				// TODO
				todo!();
			},
		}

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
