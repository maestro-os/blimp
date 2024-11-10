//! A package is a software that can be installed using the package manager.
//!
//! Packages are usually downloaded from a remote host.

use crate::{
	repository::Repository,
	version::{Version, VersionConstraint},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, fs, io, io::ErrorKind, path::PathBuf};

/// Tells whether the given package name is valid.
pub fn is_valid_name(name: &str) -> bool {
	name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// Enumeration of package dependency resolution errors.
pub enum ResolveError {
	/// The dependency cannot be found.
	NotFound {
		/// The name of the dependency.
		name: String,
		/// The version constraints on the dependency.
		version_constraint: VersionConstraint,
	},
	/// The dependency version conflicts another package or dependency.
	VersionConflict {
		/// The name of the dependency.
		name: String,

		/// Version of the required dependency.
		required_version: VersionConstraint,
		/// Version of the other element.
		other_version: Version,
	},
}

impl fmt::Display for ResolveError {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::NotFound {
				name,
				version_constraint,
			} => {
				writeln!(
					fmt,
					"Unresolved dependency `{name}` for constraint `{version_constraint}`"
				)?;
			}
			Self::VersionConflict {
				name,
				required_version,
				other_version,
			} => {
				write!(
					fmt,
					"Conflicting version `{other_version}` and `{required_version}` on dependency `{name}`!",
				)?;
			}
		}
		Ok(())
	}
}

/// A package dependency.
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Dependency {
	/// The dependency's name.
	pub name: String,
	/// The dependency's version constraints.
	///
	/// The version of the package must match the intersection of all the constraints.
	#[serde(rename = "version")]
	pub version_constraint: VersionConstraint,
}

impl fmt::Display for Dependency {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: {}", self.name, self.version_constraint)
	}
}

/// A package's description.
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Package {
	/// The package's name.
	pub name: String,
	/// The package's version.
	pub version: Version,
	/// The package's description.
	pub description: String,

	/// Dependencies required to build the package.
	pub build_deps: Vec<Dependency>,
	/// Dependencies required to run the package.
	pub run_deps: Vec<Dependency>,
}

impl Package {
	/// Loads a package from the given path.
	///
	/// If the package does not exist, the function returns None.
	pub fn load(path: PathBuf) -> io::Result<Option<Package>> {
		match fs::read_to_string(path.join("desc")) {
			Ok(content) => Ok(Some(serde_json::from_str(&content)?)),
			Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
			Err(e) => Err(e),
		}
	}

	/// Resolves the dependencies of the package and inserts them into the given `HashMap`.
	///
	/// Arguments:
	/// - `packages` is the `HashMap` which associates packages with their respective repository.
	/// - `f` is a function used to get a package from its name and version.
	///
	/// The function makes use of packages that are already in the `HashMap` and those which are
	/// already installed to determine if there is a dependency error.
	///
	/// If one or more packages cannot be resolved, the function returns the list of errors.
	pub fn resolve_dependencies<'r, F>(
		&self,
		packages: &mut HashMap<Self, &'r Repository>,
		f: &mut F,
	) -> io::Result<Result<(), Vec<ResolveError>>>
	where
		F: FnMut(&str, &VersionConstraint) -> Option<(Self, &'r Repository)>,
	{
		let mut errors = vec![];

		// TODO Add support for build dependencies
		for d in &self.run_deps {
			// TODO check already installed packages
			// Get package in the installation list
			let pkg = packages.iter().map(|(p, _)| p).find(|p| p.name == d.name);
			// Check for conflict
			if let Some(pkg) = pkg {
				if !d.version_constraint.is_valid(&pkg.version) {
					errors.push(ResolveError::VersionConflict {
						name: d.name.clone(),

						required_version: d.version_constraint.clone(),
						other_version: pkg.version.clone(),
					});
				}

				continue;
			}

			// Resolve package, then resolve its dependencies
			if let Some((p, repo)) = f(&d.name, &d.version_constraint) {
				// TODO Check for dependency cycles
				// FIXME Possible stack overflow
				let res = p.resolve_dependencies(packages, f)?;
				if let Err(e) = res {
					return Ok(Err(e));
				}

				packages.insert(p, repo);
			} else {
				errors.push(ResolveError::NotFound {
					name: d.name.clone(),
					version_constraint: d.version_constraint.clone(),
				});
			}
		}

		let res = if errors.is_empty() {
			Ok(())
		} else {
			Err(errors)
		};
		Ok(res)
	}
}

/// Information on a package that is already installed on the system.
#[derive(Clone, Deserialize, Serialize)]
pub struct InstalledPackage {
	/// The package's description.
	pub desc: Package,
	/// The list of absolute paths to installed files.
	pub files: Vec<PathBuf>,
}

/// For the given list of packages, returns the list of dependencies that are not matched.
pub fn list_unmatched_dependencies(
	pkgs: &HashMap<String, InstalledPackage>,
) -> Vec<(&InstalledPackage, &Dependency)> {
	pkgs.iter()
		.flat_map(|(_, pkg)| {
			pkg.desc
				.run_deps
				.iter()
				.filter(|dep| {
					pkgs.get(&dep.name)
						.map(|p| dep.version_constraint.is_valid(&p.desc.version))
						.unwrap_or(false)
				})
				.map(move |dep| (pkg, dep))
		})
		.collect()
}
