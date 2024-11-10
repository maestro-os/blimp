//! Implementation of the package building procedure.

use crate::{desc::BuildDescriptor, WORK_DIR};
use anyhow::Result;
use flate2::{write::GzEncoder, Compression};
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
/// - `package.json`: The file describing the package
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
	/// The path to the fake system root at which the package is "installed".
	sysroot: PathBuf,
}

impl BuildProcess {
	/// Creates a new instance.
	///
	/// `input_path` is the path to the directory containing information to build the package.
	pub fn new(input_path: PathBuf) -> io::Result<Self> {
		let build_desc_path = input_path.join("package.json");
		let build_desc = common::util::read_json::<BuildDescriptor>(&build_desc_path)?;
		Ok(Self {
			input_path,
			build_desc,
			build_dir: common::util::create_tmp_dir(WORK_DIR)?,
			sysroot: common::util::create_tmp_dir(WORK_DIR)?,
		})
	}

	/// Returns the build descriptor of the package to be built.
	pub fn get_build_desc(&self) -> &BuildDescriptor {
		&self.build_desc
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
			.sources
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
			.env("PKG_NAME", self.build_desc.package.get_name())
			.env(
				"PKG_VERSION",
				self.build_desc.package.get_version().to_string(),
			)
			.env("PKG_DESC", self.build_desc.package.get_description())
			.env("JOBS", jobs.to_string())
			.current_dir(&self.build_dir)
			.status()
			.map(|s| s.success())
	}

	/// Creates the archive of the package after being build.
	///
	/// `output_path` is the path at which the package's archive will be created.
	pub fn create_archive(&self, output_path: &Path) -> io::Result<()> {
		let build_desc_path = self.input_path.join("package.json");
		let tar_gz = File::create(output_path)?;
		let enc = GzEncoder::new(tar_gz, Compression::default());
		let mut tar = tar::Builder::new(enc);
		tar.follow_symlinks(false);
		tar.append_path_with_name(build_desc_path, "package.json")?;
		tar.append_dir_all("data", &self.sysroot)?;
		// TODO add install/update/remove hooks
		tar.finish()
	}

	/// Cleans files created by the build process.
	pub fn cleanup(self) -> io::Result<()> {
		fs::remove_dir_all(&self.build_dir)?;
		fs::remove_dir_all(&self.sysroot)?;
		Ok(())
	}
}
