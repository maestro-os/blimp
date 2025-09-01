//! Implementation of the package building procedure.

use crate::{desc::BuildDescriptor, WORK_DIR};
use common::{anyhow::Result, repository::Repository, tar, tokio, zstd};
use std::{
	fs,
	fs::File,
	io,
	path::{Path, PathBuf},
	process::Command,
	str,
	sync::Arc,
};

/// A build process is the operation of converting source code into an installable package.
///
/// To build a package, the following files are required:
/// - `build.toml`: Information to prepare for building the package
/// - `build-hook`: The script to build the package
///
/// The package is build and then installed to a fake system root, which is then compressed.
pub struct BuildProcess {
	/// The path to the directory containing information to build the package.
	input_path: PathBuf,
	/// The build descriptor.
	build_desc: BuildDescriptor,

	/// The path to the build directory.
	build_dir: PathBuf,
	/// The path to the system root in which the package is installed.
	sysroot: PathBuf,
}

impl BuildProcess {
	/// Creates a new instance.
	///
	/// Arguments:
	/// - `input_path` is the path to the directory containing information to build the package.
	/// - `sysroot` is the path to the system root. If `None`, a directory is created.
	pub fn new(input_path: PathBuf, sysroot: Option<PathBuf>) -> Result<Self> {
		let build_desc_path = input_path.join("metadata.toml");
		let build_desc = fs::read_to_string(build_desc_path)?;
		let build_desc = toml::from_str::<BuildDescriptor>(&build_desc)?;
		build_desc.package.validate()?;
		Ok(Self {
			input_path,
			build_desc,

			build_dir: common::util::create_tmp_dir(WORK_DIR)?,
			sysroot: sysroot
				.map(fs::canonicalize)
				.unwrap_or_else(|| common::util::create_tmp_dir(WORK_DIR))?,
		})
	}

	/// Returns the path to the build directory.
	pub fn get_build_dir(&self) -> &Path {
		&self.build_dir
	}

	/// Returns the path to the fake system root at which the package is "installed".
	pub fn get_sysroot(&self) -> &Path {
		&self.sysroot
	}

	/// Fetches resources required to build the package.
	pub async fn fetch_sources(&self) -> Result<()> {
		let build_dir = Arc::new(self.build_dir.clone());
		let futures = self
			.build_desc
			.source
			.iter()
			.cloned()
			.map(move |s| {
				let build_dir = build_dir.clone();
				tokio::spawn(async move { s.fetch(&build_dir).await })
			})
			.collect::<Vec<_>>();
		for f in futures {
			f.await??;
		}
		Ok(())
	}

	/// Builds the package.
	///
	/// Arguments:
	/// - `jobs` is the number of concurrent jobs.
	/// - `host` is the triplet of the host machine.
	/// - `target` is the triplet of the target machine.
	///
	/// On success, the function returns `true`.
	pub fn build(&self, jobs: usize, build: &str, host: &str, target: &str) -> io::Result<bool> {
		let absolute_input = fs::canonicalize(&self.input_path)?;
		let hook_path = absolute_input.join("build-hook");
		Command::new(hook_path)
			.env("DESC_PATH", absolute_input)
			.env("BUILD", build)
			.env("HOST", host)
			.env("TARGET", target)
			.env("SYSROOT", &self.sysroot)
			.env("PKG_NAME", &self.build_desc.package.name)
			.env("PKG_VERSION", self.build_desc.package.version.to_string())
			.env("PKG_DESC", &self.build_desc.package.description)
			.env("JOBS", jobs.to_string())
			.current_dir(&self.build_dir)
			.status()
			.map(|s| s.success())
	}

	/// Writes the package's metadata to the repository
	pub fn write_metadata(&self, repo: &Repository, arch: &str) -> Result<()> {
		// Make sure the arch directory exists
		fs::create_dir_all(repo.get_path().join(arch))?;
		// Create metadata
		let metadata = toml::to_string(&self.build_desc.package)?;
		fs::write(
			repo.get_metadata_path(
				arch,
				&self.build_desc.package.name,
				&self.build_desc.package.version,
			),
			metadata,
		)?;
		Ok(())
	}

	/// Creates the archive of the package after being build.
	pub fn create_archive(&self, repo: &Repository, arch: &str) -> io::Result<()> {
		let output_path = repo.get_archive_path(
			arch,
			&self.build_desc.package.name,
			&self.build_desc.package.version,
		);
		let build_desc_path = self.input_path.join("metadata.toml");
		let archive = File::create(output_path)?;
		let enc = zstd::stream::Encoder::new(archive, 0)?;
		let mut tar = tar::Builder::new(enc);
		tar.follow_symlinks(false);
		tar.append_path_with_name(build_desc_path, "metadata.toml")?;
		tar.append_dir_all("data", &self.sysroot)?;
		// TODO add install/update/remove hooks
		tar.finish()
	}

	/// Cleans files created by the build process.
	///
	/// `sysroot`: if `true`, the sysroot is also deleted.
	pub fn cleanup(self, sysroot: bool) -> io::Result<()> {
		fs::remove_dir_all(self.build_dir)?;
		if sysroot {
			fs::remove_dir_all(self.sysroot)?;
		}
		Ok(())
	}
}
