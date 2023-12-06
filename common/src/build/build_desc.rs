//! This module implements the build descriptor structure.

use crate::package::Package;
use crate::util;
use anyhow::bail;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[cfg(feature = "network")]
use crate::download::DownloadTask;

/// Structure representing the location of sources and where to unpack them.
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Source {
	/// Downloading a tarball from an URL.
	Url {
		/// The location relative to the build directory where the archive will be unpacked.
		location: PathBuf,

		/// The URL of the sources.
		url: String,

		/// If true, unwrapping the tarball.
		unwrap: bool,
	},

	/// Cloning the given repository.
	Git {
		/// The location relative to the build directory where the archive will be unpacked.
		location: PathBuf,

		/// The URL to the Git repository.
		git_url: String,
		/// The branch to clone from. If not specified, the default branch is used.
		branch: Option<String>,
	},

	/// Copying from a local path.
	Local {
		/// The location relative to the build directory where the archive will be unpacked.
		location: PathBuf,

		/// The path to the local tarball or directory.
		path: String,

		/// If true, unwrapping the tarball.
		unwrap: bool,
	},
}

impl Source {
	/// Fetches files from the source and uncompresses them if necessary.
	/// Files are placed into the build directory `build_dir` according to the location.
	pub async fn fetch(&self, build_dir: &Path) -> Result<()> {
		#[cfg(not(feature = "network"))]
		match self {
			Self::Local {
				location,

				path,

				unwrap,
			} => {
				let _dest_path = util::concat_paths(build_dir, location);

				// TODO
			}

			_ => {
				panic!(
					"Feature `network` is not enabled! Please recompile blimp common with \
this feature enabled"
				);
			}
		}

		#[cfg(feature = "network")]
		match self {
			Self::Url {
				location,

				url,

				unwrap,
			} => {
				let dest_path = util::concat_paths(build_dir, location);

				// Downloading
				let (path, _) = util::create_tmp_file()?;
				let mut download_task = DownloadTask::new(url, &path).await?;
				while download_task.next().await? {}

				// Uncompressing the archive
				util::uncompress(&path, &dest_path, *unwrap)?;
			}

			Self::Git {
				location,

				git_url,
				branch,
			} => {
				let dest_path = util::concat_paths(build_dir, location);

				let mut cmd = Command::new("git")
					.arg("clone")
					// Only keep the last commit
					.arg("--depth")
					.arg("1")
					.arg("--single-branch");
				if let Some(branch) = branch {
					cmd.arg("-b");
					cmd.arg(branch);
				}
				let status = cmd.arg(git_url).arg(dest_path).status()?;
				if !status.success() {
					bail!("Cloning `{git_url}` failed");
				}
			}

			_ => {}
		}

		// TODO Remove the archive?

		Ok(())
	}
}

/// Structure describing how to build a package.
#[derive(Deserialize, Serialize)]
pub struct BuildDescriptor {
	/// The list of sources for the package.
	pub sources: Vec<Source>,

	/// The package's descriptor.
	pub package: Package,
}
