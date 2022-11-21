//! This module implements the build descriptor structure.

use crate::package::Package;
use crate::util;
use crate::version::Version;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::fs;
use std::io::BufReader;
use std::io;
use std::path::Path;
use std::process::Command;

#[cfg(feature = "network")]
use crate::download::DownloadTask;

/// The directory storing packages' sources to build them on serverside.
pub const SERVER_PACKAGES_SRC_DIR: &str = "build_src";

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

		/// If true, unwrapping the tarball.
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

		/// If true, unwrapping the tarball.
		unwrap: bool,
	},
}

impl Source {
	/// Fetches files from the source and uncompresses them if necessary.
	/// Files are placed into the build directory `build_dir` according to the location.
	pub async fn fetch(&self, build_dir: &Path) -> Result<(), Box<dyn Error>> {
		#[cfg(not(feature = "network"))]
		match self {
			Self::Local {
				location,

				path,

				unwrap,
			} => {
				// TODO
			}

			_ => {
				panic!("Feature `network` is not enabled! Please recompile blimp common with \
this feature enabled");
			},
		}

		#[cfg(feature = "network")]
		match self {
			Self::Url {
				location,

				url,

				unwrap,
			} => {
				let (path, _) = util::create_tmp_file()?;

				// Downloading
				let mut download_task = DownloadTask::new(url, &path).await?;
				while download_task.next().await? {}

				let dest_path = build_dir.join(location);

				// Uncompressing the archive
				util::uncompress(&path, &dest_path, *unwrap)?;
			}

			Self::Git {
				location,

				git_url,
			} => {
				let dest_path = build_dir.join(location);

				let status = Command::new("git")
					.args([
						OsString::from("clone"),
						OsString::from(git_url),
						dest_path.into()
					])
					.status()?;

				if !status.success() {
					return Err(format!("Cloning `{}` failed", git_url).into());
				}
			}

			_ => {},
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
	/// Lists build descriptors on serverside.
	/// The function returns a vector of package paths and their associated respective descriptors.
	pub fn server_list() -> io::Result<Vec<(String, Self)>> {
		let mut descs = Vec::new();

		let files = fs::read_dir(SERVER_PACKAGES_SRC_DIR)?;
		for p in files {
			let path = p?.path().into_os_string().into_string().unwrap();
			let desc_path = format!("{}/package.json", path);

			match File::open(desc_path.clone()) {
				Ok(file) => {
					let reader = BufReader::new(file);
					descs.push((path, serde_json::from_reader(reader)?));
				}

				Err(err) => {
					eprintln!("Warning: cannot open `{}`: {}", desc_path, err);
				}
			}
		}

		Ok(descs)
	}

	/// TODO doc
	pub fn server_get(name: &str, version: &Version) -> io::Result<Option<(String, Self)>> {
		// TODO Optimize
		Ok(Self::server_list()?
			.into_iter()
			.filter(|(_, desc)| {
				desc.package.get_name() == name && desc.package.get_version() == version
			})
			.next())
	}

	/// Returns a reference to the list of sources.
	pub fn get_sources(&self) -> &Vec<Source> {
		&self.sources
	}

	/// Returns a reference to the package descriptor.
	pub fn get_package(&self) -> &Package {
		&self.package
	}
}
