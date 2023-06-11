//! A package is a software that can be installed using the package manager.
//! Packages are usualy downloaded from a remote host.

use crate::repository::Repository;
use crate::version::Version;
use crate::version::VersionConstraint;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::PathBuf;

/// The directory storing packages' descriptions on the serverside.
pub const SERVER_PACKAGES_DESC_DIR: &str = "public_desc";
/// The directory storing packages' archives on the serverside.
pub const SERVER_PACKAGES_DIR: &str = "public_packages";

/// Enumeration of possible package dependencies resolution error.
pub enum ResolveError {
	/// The dependency cannot be found.
	NotFound {
		/// The name of the dependency.
		name: String,
		/// The version constraints on the dependency.
		version_constraints: Vec<VersionConstraint>,
	},

	/// The dependency version conflicts another package or dependency.
	VersionConflict {
		/// The name of the package.
		name: String,

		/// Version of the required dependency.
		v0: Version,
		/// Version of the other element.
		v1: Version,
	},
}

impl fmt::Display for ResolveError {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::NotFound {
				name,
				version_constraints,
			} => {
				write!(fmt, "Unresolved dependency `{}` for constraints:\n", name)?;

				for c in version_constraints {
					write!(fmt, "- `{}`\n", c)?;
				}
			}

			Self::VersionConflict {
				name,

				v0,
				v1,
			} => {
				write!(
					fmt,
					"Conflicting version `{}` and `{}` on `{}`!",
					v0, v1, name
				)?;
			}
		}

		Ok(())
	}
}

/// Structure representing a package dependency.
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Dependency {
	/// The dependency's name.
	name: String,

	/// The dependency's version constraints.
	///
	/// The version of the package must match the intersection of all the constraints.
	version: Vec<VersionConstraint>,
}

impl Dependency {
	/// Returns the name of the package.
	pub fn get_name(&self) -> &String {
		&self.name
	}

	/// Returns the version of the package.
	pub fn get_version_constraints(&self) -> &[VersionConstraint] {
		&self.version
	}

	/// Tells whether the given version matches every containts.
	pub fn is_valid(&self, version: &Version) -> bool {
		self.version.iter().all(|c| c.is_valid(version))
	}
}

impl Display for Dependency {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}: ", self.name)?;

		for (i, c) in self.version.iter().enumerate() {
			write!(f, "{}", c)?;

			if i + 1 < self.version.len() {
				write!(f, ", ")?;
			}
		}

		Ok(())
	}
}

/// A package's description.
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
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

	/// Resolves the dependencies of the package and inserts them into the given HashMap
	/// `packages`.
	///
	/// Arguments:
	/// - `f` is a function used to get a package from its name and version.
	///
	/// The function makes use of packages that are already in the HashMap and those which are
	/// already installed to determine if there is a dependency error.
	///
	/// If one or more packages cannot be resolved, the function returns the list of errors.
	pub fn resolve_dependencies<'r, F>(
		&self,
		packages: &mut HashMap<Self, &'r Repository>,
		f: &mut F,
	) -> io::Result<Result<(), Vec<ResolveError>>>
	where
		F: FnMut(&str, &[VersionConstraint]) -> Option<(Self, &'r Repository)>,
	{
		let mut errors = vec![];

		// TODO Add support for build dependencies
		for d in &self.run_deps {
			// Get package in the installation list
			let pkg = packages
				.iter()
				.map(|(p, _)| p)
				.filter(|p| p.get_name() == d.get_name())
				.next();
			// Check for conflict
			if let Some(pkg) = pkg {
				if !d.is_valid(pkg.get_version()) {
					/*errors.push(ResolveError::VersionConflict {
						v0: d.get_version().clone(),
						v1: d.get_version().clone(),

						name: d.get_name().clone(),
					});*/
					todo!();
				}

				continue;
			}

			// Resolve package, then resolve its dependencies
			if let Some((p, repo)) = f(d.get_name(), d.get_version_constraints()) {
				// TODO Check for dependency cycles
				// FIXME Possible stack overflow
				let res = p.resolve_dependencies(packages, f)?;
				match res {
					Err(e) => return Ok(Err(e)),
					_ => {}
				}

				packages.insert(p, repo);
			} else {
				errors.push(ResolveError::NotFound {
					name: d.get_name().clone(),
					version_constraints: d.get_version_constraints().to_vec(),
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
	/// The list of absolute pathes to installed files.
	pub files: Vec<PathBuf>,
}

/// For the given list of packages, returns the list of dependencies that are not matched.
pub fn list_unmatched_dependencies<'p>(
	pkgs: &'p HashMap<String, InstalledPackage>,
) -> Vec<(&'p InstalledPackage, &'p Dependency)> {
	pkgs.iter()
		.map(|(_, pkg)| {
			pkg.desc
				.get_run_deps()
				.iter()
				.filter(|dep| {
					let matching = pkgs
						.get(dep.get_name())
						.map(|p| dep.is_valid(p.desc.get_version()))
						.unwrap_or(false);

					!matching
				})
				.map(|dep| (pkg, dep))
				.collect::<Vec<_>>()
		})
		.flatten()
		.collect()
}
