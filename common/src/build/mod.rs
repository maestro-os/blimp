//! TODO doc

pub mod build_desc;

use crate::build::build_desc::BuildDescriptor;
use crate::util;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;
use tokio::runtime::Runtime;

/// A build process referes to the operation of converting source code into an installable package.
///
/// To build a package, the following files are needed:
/// - `package.json`: The file describing the package
/// - `build-hook`: The script to build the package
pub struct BuildProcess {
	/// The path to the directory containing informations to build the package.
	input_path: PathBuf,

	/// The build descriptor.
	build_desc: Option<BuildDescriptor>,

	/// The path to the build directory.
	build_dir: Option<PathBuf>,
	/// The path to the fake sysroot at which the package is "installed".
	sysroot: Option<PathBuf>,

	/// Whether temporary files must be cleaned up on drop.
	clean_on_drop: bool,
}

impl BuildProcess {
	/// Creates a new instance.
	///
	/// `input_path` is the path to the directory containing informations to build the package.
	pub fn new(input_path: PathBuf) -> Self {
		Self {
			input_path,

			build_desc: None,

			build_dir: None,
			sysroot: None,

			clean_on_drop: true,
		}
	}

	/// TODO doc
	pub fn get_build_desc(&self) -> Option<&BuildDescriptor> {
		self.build_desc.as_ref()
	}

	/// TODO doc
	pub fn get_build_dir(&self) -> Option<&PathBuf> {
		self.build_dir.as_ref()
	}

	/// TODO doc
	pub fn get_sysroot(&self) -> Option<&PathBuf> {
		self.sysroot.as_ref()
	}

	/// Prepares for building.
	pub fn prepare(&mut self) -> io::Result<()> {
		let build_desc_path = self.input_path.join("package.json");
		self.build_desc = Some(util::read_json::<BuildDescriptor>(&build_desc_path)?);

		self.build_dir = Some(util::create_tmp_dir()?);
		self.sysroot = Some(util::create_tmp_dir()?);

		Ok(())
	}

	/// Fetches resources required to build the package.
	pub fn fetch_sources(&self) -> Result<(), Box<dyn Error>> {
		let (
			Some(build_dir),
			Some(build_desc)
		) = (
			&self.build_dir,
			&self.build_desc
		) else {
			return Ok(());
		};

		let runtime = Runtime::new()?;
		let futures = build_desc
			.sources
			.iter()
			.map(|s| s.fetch(&build_dir))
			.collect::<Vec<_>>();

		for f in futures {
			runtime.block_on(f)?;
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
	pub fn build(&self, jobs: u32, host: &str, target: &str) -> io::Result<bool> {
		let (
			Some(build_dir),
			Some(sysroot),
		) = (
			&self.build_dir,
			&self.sysroot,
		) else {
			return Ok(false);
		};

		let absolute_input = fs::canonicalize(&self.input_path)?;
		let hook_path = absolute_input.join("build-hook");

		Command::new(hook_path)
			.env("DESC_PATH", absolute_input)
			.env("HOST", host)
			.env("TARGET", target)
			.env("SYSROOT", sysroot)
			.env("JOBS", format!("{}", jobs))
			.current_dir(build_dir)
			.status()
			.map(|status| status.success())
	}

	/// Creates the archive of the package after being build.
	///
	/// `output_path` is the path at which the package's archive will be created.
	pub fn create_archive(&self, output_path: &Path) -> io::Result<()> {
		let build_desc_path = self.input_path.join("package.json");

		let Some(ref sysroot) = self.sysroot else {
			// TODO
			todo!();
		};

		let tar_gz = File::create(output_path)?;
		let enc = GzEncoder::new(tar_gz, Compression::default());
		let mut tar = tar::Builder::new(enc);
		tar.follow_symlinks(false);
		tar.append_path_with_name(build_desc_path, "package.json")?;
		tar.append_dir_all("data", sysroot)?;
		// TODO add install/update/remove hooks

		tar.finish()
	}

	/// Set whether temporary files must be cleaned up on drop.
	pub fn set_clean_on_drop(&mut self, clean: bool) {
		self.clean_on_drop = clean;
	}
}

impl Drop for BuildProcess {
	fn drop(&mut self) {
		if self.clean_on_drop {
			match self.build_dir {
				Some(ref path) => {
					let _ = fs::remove_dir_all(path);
				}
				None => {}
			}

			match self.sysroot {
				Some(ref path) => {
					let _ = fs::remove_dir_all(path);
				}
				None => {}
			}
		}
	}
}

/// Returns the triplet of the host on which the package is to be built.
///
/// If the triplet cannot be retrieved, the function returns None.
pub fn get_host_triplet() -> Option<String> {
	let output = Command::new("cc").arg("-dumpmachine").output();

	if let Ok(out) = output {
		if let Ok(triplet) = str::from_utf8(&out.stdout) {
			return Some(triplet.trim().to_owned());
		}
	}

	None
}
