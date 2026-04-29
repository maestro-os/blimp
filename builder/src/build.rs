/*
 * Copyright 2025 Luc Lenôtre
 *
 * This file is part of Maestro.
 *
 * Maestro is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Maestro is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Maestro. If not, see <https://www.gnu.org/licenses/>.
 */

//! Implementation of the package building procedure.

use crate::desc::BuildDescriptor;
use common::{
	anyhow::{anyhow, bail, Result},
	flate2::{write::GzEncoder, Compression},
	maestro_utils::{fhs, user::get_euid},
	package::{DependencyType, Package},
	repository::{
		get_package_with_constraint, get_recursive_dependencies, remote::download_packages,
		PackagesWithRepositoryMap, Repository,
	},
	tar, tokio,
	util::{create_tmp_dir, current_arch},
	Environment,
};
use std::{
	ffi::CString,
	fs::{self, File},
	io,
	os::unix::{ffi::OsStrExt, fs::chroot, process::CommandExt},
	path::{Path, PathBuf},
	process::Command,
	str,
	sync::Arc,
};

/// Get original build-hook file path
fn get_build_hook_path(input_path: &Path) -> io::Result<PathBuf> {
	let absolute_input = fs::canonicalize(input_path)?;
	Ok(absolute_input.join("build-hook"))
}

/// Populates `sysroot/dev/` with the basic character device nodes that build hooks expect. Without
/// these, redirections like `>/dev/null` create regular files that accumulate output and corrupt
/// later reads.
fn create_dev_nodes(sysroot: &Path) -> io::Result<()> {
	let nodes: &[(&str, u32, u32)] = &[
		("dev/null", 1, 3),
		("dev/zero", 1, 5),
		("dev/full", 1, 7),
		("dev/random", 1, 8),
		("dev/urandom", 1, 9),
		("dev/tty", 5, 0),
	];
	for (rel, major, minor) in nodes {
		let path = sysroot.join(rel);
		match fs::remove_file(&path) {
			Ok(_) => {}
			Err(e) if e.kind() == io::ErrorKind::NotFound => {}
			Err(e) => return Err(e),
		}
		let cpath = CString::new(path.as_os_str().as_bytes())
			.map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
		let mode = libc::S_IFCHR | 0o666;
		let dev = libc::makedev(*major, *minor);
		let rc = unsafe { libc::mknod(cpath.as_ptr(), mode, dev) };
		if rc != 0 {
			return Err(io::Error::last_os_error());
		}
	}
	Ok(())
}

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
	pub(crate) build_dir: PathBuf,
	/// The path to the directory in which the package is installed.
	pub(crate) install_path: PathBuf,
	/// The path to the system root.
	pub(crate) sysroot: PathBuf,
	/// Building in chroot environment?
	chroot: bool,
}

/// Creates a sysroot for building the package, with its dependencies installed.
///
/// Arguments:
/// - `sysroot` the path to the system root
/// - `input_path` is the path to the directory containing information to build the package
/// - `package` is the package to build
async fn create_sysroot(sysroot: &Path, input_path: &Path, package: &Package) -> Result<()> {
	if let Err(e) = fhs::create_dirs(sysroot, false) {
		bail!("FHS creation failed: {e}");
	}
	create_dev_nodes(sysroot)?;
	fs::copy(
		get_build_hook_path(input_path)?,
		sysroot.join("bin/build-hook"),
	)?;

	let arch = current_arch();
	let host_env =
		Environment::acquire(Path::new("/"), arch)?.expect("unexpected environment lock");
	let repos = host_env.list_repositories()?;
	for r in &repos {
		if let Some(remote) = r.get_remote() {
			remote.fetch_index(&host_env).await?;
		}
	}
	let pkgs: PackagesWithRepositoryMap = package
		.deps
		.iter()
		.map(|dep| {
			get_package_with_constraint(&repos, arch, &dep.name, Some(&dep.version_constraint))?
				.map(|p| (p.1, p.0))
				.ok_or_else(|| anyhow!("dependency `{}` not found in repositories", dep.name))
		})
		.collect::<Result<_>>()?;
	let deps = get_recursive_dependencies(&pkgs, &repos, DependencyType::Build, arch)?
		.into_iter()
		.collect();
	download_packages(&deps, arch).await?;
	drop(host_env);
	let mut target_env =
		Environment::acquire(sysroot, arch)?.expect("unexpected environment lock");
	target_env.install_packages(&deps)?;
	Ok(())
}

impl BuildProcess {
	/// Creates a new instance.
	///
	/// Arguments:
	/// - `input_path` is the path to the directory containing information to build the package.
	/// - `install_path` is the path to the install directory. If `None`, a directory is created.
	/// - `work_dir` is the directory where build directories are located
	/// - `chroot` for building in chroot environment
	pub async fn new(
		input_path: PathBuf,
		install_path: Option<PathBuf>,
		work_dir: &Path,
		chroot: bool,
	) -> Result<Self> {
		// TODO replace root user check by CAP_SYS_CHROOT and CAP_MKNOD when implemented in kernel
		if chroot && get_euid() != 0 {
			bail!("--chroot requires root privileges!");
		}
		let build_desc_path = input_path.join("metadata.toml");
		let build_desc = fs::read_to_string(build_desc_path)?;
		let build_desc = toml::from_str::<BuildDescriptor>(&build_desc)?;
		build_desc.package.validate()?;

		let sysroot_exists = install_path.is_some();
		let (build_dir, install_path, sysroot) = if chroot {
			let sysroot = create_tmp_dir(work_dir)?;
			let pkg_dir_name =
				format!("{}-{}", build_desc.package.name, build_desc.package.version);
			let build_dir = sysroot.join("usr/src").join(&pkg_dir_name);
			let install_dir = sysroot.join("var/lib").join(pkg_dir_name);
			(build_dir, install_dir, sysroot)
		} else {
			(
				create_tmp_dir(work_dir)?,
				install_path
					.as_ref()
					.map(fs::canonicalize)
					.unwrap_or_else(|| create_tmp_dir(work_dir))?,
				install_path
					.map(fs::canonicalize)
					.unwrap_or_else(|| create_tmp_dir(work_dir))?,
			)
		};
		if !sysroot_exists {
			create_sysroot(&sysroot, &input_path, &build_desc.package).await?;
		}

		Ok(Self {
			input_path,
			build_desc,
			build_dir,
			install_path,
			sysroot,
			chroot,
		})
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
		let hook_path = if self.chroot {
			PathBuf::from("/bin/build-hook")
		} else {
			get_build_hook_path(&self.input_path)?
		};
		// TODO refactor
		// In chroot mode, paths exposed to the build hook must be relative to the
		// chroot root
		let (sysroot_env, install_path_env): (PathBuf, PathBuf) = if self.chroot {
			let sysroot = PathBuf::from("/");
			let install_path = Path::new("/").join(
				self.install_path
					.strip_prefix(&self.sysroot)
					.unwrap_or(&self.install_path),
			);
			(sysroot, install_path)
		} else {
			(self.sysroot.clone(), self.install_path.clone())
		};
		let mut cmd = Command::new(hook_path);
		if self.chroot {
			let sysroot = self.sysroot.clone();
			unsafe {
				cmd.pre_exec(move || chroot(&sysroot));
			}
		}
		cmd.env("BUILD", build)
			.env("HOST", host)
			.env("TARGET", target)
			.env("SYSROOT", &sysroot_env)
			.env("INSTALL_PATH", &install_path_env)
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
		let path = repo.get_metadata_path(
			arch,
			&self.build_desc.package.name,
			&self.build_desc.package.version,
		);
		// Make sure the parent directory exists
		fs::create_dir_all(path.parent().unwrap())?;
		// Create metadata
		let metadata = toml::to_string(&self.build_desc.package)?;
		fs::write(path, metadata)?;
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
		let enc = GzEncoder::new(archive, Compression::default());
		let mut tar = tar::Builder::new(enc);
		tar.follow_symlinks(false);
		tar.append_path_with_name(build_desc_path, "metadata.toml")?;
		tar.append_dir_all("data", &self.install_path)?;
		// TODO add install/update/remove hooks
		tar.finish()
	}

	/// Cleans files created by the build process.
	///
	/// `is_packaged`: if `true`, the install directory is also deleted.
	pub fn cleanup(self, is_packaged: bool) -> io::Result<()> {
		fs::remove_dir_all(self.build_dir)?;
		if is_packaged {
			fs::remove_dir_all(self.install_path)?;
		}
		Ok(())
	}
}
