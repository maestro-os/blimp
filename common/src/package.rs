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

//! A package is a software that can be installed using the package manager.
//!
//! Packages are usually downloaded from a remote host.

use crate::{
	repository::Repository,
	version::{Version, VersionConstraint},
};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	fmt, fs,
	io::{self, ErrorKind},
	path::{Path, PathBuf},
};

/// Tells whether the given package name is valid.
pub fn is_valid_name(name: &str) -> bool {
	if name.len() < 2 {
		return false;
	}
	name.chars().enumerate().all(|(i, c)| {
		if i == 0 {
			c.is_ascii_lowercase()
		} else {
			c.is_ascii_lowercase() || c.is_ascii_digit() || "+-.".contains(c)
		}
	})
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

/// The type of dependency.
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum DependencyType {
	#[serde(rename = "build")]
	Build,
	#[serde(rename = "run")]
	Run,
	#[serde(rename = "build-and-run")]
	BuildAndRun,
}

impl Default for DependencyType {
	fn default() -> Self {
		Self::Run
	}
}

/// A package dependency.
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Dependency {
	/// The type of the dependency
	#[serde(rename = "type", default = "DependencyType::default")]
	pub dep_type: DependencyType,
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
	/// The package's name
	pub name: String,
	/// The package's version
	pub version: Version,
	/// The package's description
	pub description: String,

	/// Dependencies required to build/run the package
	#[serde(default, skip_serializing_if = "Vec::is_empty", rename = "dep")]
	pub deps: Vec<Dependency>,
}

impl Package {
	/// Loads a package from the metadata file.
	///
	/// If the package does not exist, the function returns `None`.
	pub fn from_file(metadata_path: &Path) -> Result<Option<Package>> {
		match fs::read_to_string(metadata_path) {
			Ok(content) => Ok(Some(toml::from_str(&content)?)),
			Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
			Err(e) => Err(e.into()),
		}
	}

	/// Validates the package's metadata
	pub fn validate(&self) -> Result<()> {
		if !is_valid_name(&self.name) {
			bail!("invalid package name: {}", self.name);
		}
		for d in &self.deps {
			if !is_valid_name(&d.name) {
				bail!("invalid dependency name: {}", d.name);
			}
		}
		Ok(())
	}

	/// Resolves the dependencies of the package and inserts them into the given `HashMap`.
	///
	/// Arguments:
	/// - `packages` is the `HashMap` which associates packages with their respective repository.
	/// - `dep_type` is the type of dependencies to resolve. `BuildAndRun` resolves everything.
	/// - `f` is a function used to get a package from its name and version.
	///
	/// The function makes use of packages that are already in the `HashMap` and those which are
	/// already installed to determine if there is a dependency error.
	///
	/// If one or more packages cannot be resolved, the function returns the list of errors.
	pub fn resolve_dependencies<'r, F>(
		&self,
		packages: &mut HashMap<Self, &'r Repository>,
		dep_type: DependencyType,
		f: &mut F,
	) -> io::Result<Result<(), Vec<ResolveError>>>
	where
		F: FnMut(&str, &VersionConstraint) -> Option<(Self, &'r Repository)>,
	{
		let mut errors = vec![];

		for d in &self.deps {
			// if filter by build & run, get all deps
			if dep_type != DependencyType::BuildAndRun
			    // if dep is build & run, we need it anyway
				&& d.dep_type != DependencyType::BuildAndRun
				// apply filter
				&& d.dep_type != dep_type
			{
				continue;
			}
			// TODO check already installed packages
			// Get package in the installation list
			let pkg = packages.keys().find(|p| p.name == d.name);
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
				// At this point, we should only need run dependencies, as we are past the build
				// step.
				let res = p.resolve_dependencies(packages, DependencyType::Run, f)?;
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
				.deps
				.iter()
				.filter(|dep| {
					dep.dep_type != DependencyType::Build
						&& pkgs
							.get(&dep.name)
							.map(|p| dep.version_constraint.is_valid(&p.desc.version))
							.unwrap_or(false)
				})
				.map(move |dep| (pkg, dep))
		})
		.collect()
}
