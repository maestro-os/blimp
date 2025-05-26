//! A build descriptor defines how to build a package.
//!
//! A build descriptor contains general information about the package, but also sources for files
//! used for building the package.
//!
//! Source files may come from different sources. See [`SourceRemote`].
//!
//! A tarball is a compressed file containing sources for a package.
//! Tarballs may contain a single directory in which all files are present. "Unwrapping" is the
//! action of moving all the files out of this directory while decompressing the archive.

use common::{anyhow::Result, package::Package};
use serde::{Deserialize, Serialize};
use std::{
	fs,
	path::{Path, PathBuf},
};

/// Remote location of a source.
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SourceRemote {
	/// Download a tarball from a URL.
	Url {
		/// The URL of the sources.
		url: String,
		// TODO add optional hash to check tarball
		// TODO add option to fetch hash from URL?
	},
	/// Clone the given repository.
	Git {
		/// The URL to the Git repository.
		git_url: String,
		/// The branch to clone from. If not specified, the default branch is used.
		branch: Option<String>,
	},
	/// Copy from a local path to a tarball or directory.
	Local {
		/// The local path.
		path: PathBuf,
	},
}

/// Description of sources files, where to find them and where to place them for building.
#[derive(Clone, Deserialize, Serialize)]
pub struct Source {
	/// Source-type specific fields.
	#[serde(flatten)]
	inner: SourceRemote,
	/// The location relative to the build directory where the source files will be placed.
	location: PathBuf,
}

impl Source {
	/// Fetches files from the source and decompress them if necessary.
	///
	/// Files are placed into the build directory `build_dir` according to the specified location.
	pub async fn fetch(&self, build_dir: &Path) -> Result<()> {
		let dest_path = common::util::concat_paths(build_dir, &self.location);
		match &self.inner {
			SourceRemote::Local { path } => {
				let metadata = fs::metadata(path)?;
				if metadata.is_dir() {
					common::util::recursive_copy(path, &dest_path)?;
				} else {
					// TODO decompress only if it is an actual archive
					common::util::decompress(path, &dest_path)?;
				}
			}
			#[cfg(not(feature = "network"))]
			_ => panic!("Feature `network` is not enabled! Please recompile blimp with this feature enabled"),
			#[cfg(feature = "network")]
			_ => {}
		}
		#[cfg(feature = "network")]
		match &self.inner {
			SourceRemote::Url {
				url,
			} => {
				use crate::WORK_DIR;
				use common::download::DownloadTask;

				// Download
				let (path, _) = common::util::create_tmp_file(WORK_DIR)?;
				let mut download_task = DownloadTask::new(url, &path).await?;
				// TODO progress bar
				while download_task.next().await? > 0 {}
				// TODO check integrity with hash if specified
				common::util::decompress(&path, &dest_path)?;
			}
			SourceRemote::Git {
				git_url,
				branch,
			} => {
				use common::anyhow::bail;
				use std::process::Command;
				let mut cmd = Command::new("git");
				cmd.arg("clone")
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
					bail!("Cloning from `{git_url}` failed");
				}
			}
			_ => {}
		}
		Ok(())
	}
}

/// Description of how to build a package.
#[derive(Deserialize, Serialize)]
pub struct BuildDescriptor {
	/// The list of sources for the package.
	pub sources: Vec<Source>,
	/// The package's descriptor.
	pub package: Package,
}
