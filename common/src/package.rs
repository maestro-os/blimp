//! A package is a software that can be installed using the package manager.
//! Packages are usualy downloaded from a remote host.

use crate::install;
use crate::version::Version;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// The directory storing packages' descriptions on the serverside.
pub const SERVER_PACKAGES_DESC_DIR: &str = "public_desc";
/// The directory storing packages' archives on the serverside.
pub const SERVER_PACKAGES_DIR: &str = "public_packages";

/// Structure representing a package dependency.
#[derive(Clone, Eq, Deserialize, Serialize)]
pub struct Dependency {
	/// The dependency's name.
	name: String,
	/// The dependency's version.
	version: Version, // TODO Add constraints (less, equal or greater)
}

impl Dependency {
	/// Returns the name of the package.
	pub fn get_name(&self) -> &String {
		&self.name
	}

	/// Returns the version of the package.
	pub fn get_version(&self) -> &Version {
		&self.version
	}
}

impl Ord for Dependency {
	fn cmp(&self, other: &Self) -> Ordering {
		self.name
			.cmp(&other.name)
			.then_with(|| self.version.cmp(&other.version))
	}
}

impl PartialOrd for Dependency {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for Dependency {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name && self.version == other.version
	}
}

/// Structure representing a package.
#[derive(Clone, Deserialize, Serialize)]
pub struct Package {
	/// The package's name.
	name: String,
	/// The package's version.
	version: Version,

	/// The package's description.
	description: String,

	/// Dependencies required to build the package.
	build_deps: Vec<Dependency>,
	/// Dependencies required to run the package.
	run_deps: Vec<Dependency>,
}

impl Package {
	/// Loads a package from the given path.
	///
	/// If the package doesn't exist, the function returns None.
	pub fn load(path: PathBuf) -> io::Result<Option<Package>> {
		match fs::read_to_string(path.join("desc")) {
			Ok(content) => Ok(Some(serde_json::from_str(&content)?)),
			Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
			Err(e) => Err(e),
		}
	}

	/// Returns the name of the package.
	pub fn get_name(&self) -> &str {
		&self.name
	}

	/// Returns the version of the package.
	pub fn get_version(&self) -> &Version {
		&self.version
	}

	/// Returns the description of the package.
	pub fn get_description(&self) -> &str {
		&self.description
	}

	/// Returns the list of build dependencies.
	pub fn get_build_deps(&self) -> &Vec<Dependency> {
		&self.build_deps
	}

	/// Returns the list of run dependencies.
	pub fn get_run_deps(&self) -> &Vec<Dependency> {
		&self.run_deps
	}

	// TODO Move printing out of this function
	/// Resolves the dependencies of the package and inserts them into the given HashMap
	/// `packages`.
	///
	/// Arguments:
	/// - `sysroot` is the path to the system's root.
	/// - `f` is a function used to get a package from its name and version.
	///
	/// If the package doesn't exist, the function returns None.
	///
	/// The function makes use of packages that are already in the HashMap and those which are
	/// already installed to determine if there is a dependency error.
	///
	/// If an error occurs, the function returns `false`.
	pub fn resolve_dependencies<F>(
		&self,
		sysroot: &Path,
		packages: &mut HashMap<String, Self>,
		f: &mut F,
	) -> io::Result<bool>
		where F: FnMut(&str, &Version) -> Option<Self>,
	{
		// Tells whether dependencies are valid
		let mut valid = true;

		for d in &self.run_deps {
			// Getting the package. Either in the installation list or already installed
			let pkg = install::get_installed(sysroot, d.get_name())?
				.or_else(|| Some(packages.get(d.get_name())?.clone()));

			// Checking for version conflict
			if let Some(p) = pkg {
				// If versions don't correspond, error
				if d.get_version() != p.get_version() {
					eprintln!(
						"Conflicting version `{}` and `{}` on `{}`!",
						d.get_version(),
						p.get_version(),
						d.get_name()
					);
					valid = false;
				}

				continue;
			}

			// Resolving the package, then resolving its dependencies
			if let Some(p) = f(d.get_name(), d.get_version()) {
				p.resolve_dependencies(sysroot, packages, f)?; // FIXME Possible stack overflow
				packages.insert(p.get_name().to_owned(), p);
			} else {
				eprintln!(
					"Unresolved dependency `{}` version `{}`!",
					d.get_name(),
					d.get_version()
				);
				valid = false;
			}
		}

		Ok(valid)
	}
}
