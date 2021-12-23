//! A package is a software that can be installed using the package manager.
//! Packages are usualy downloaded from a remote host.

use crate::version::Version;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;

/// The directory containing cached packages.
const CACHE_DIR: &str = "/usr/lib/blimp/cache";

/// Structure representing a package dependency.
#[derive(Clone, Eq)]
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

    /// Returns the package corresponding to the dependency.
    pub fn get_package(&self) -> Option<Package> {
        // TODO
        todo!();
    }
}

impl Ord for Dependency {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name).then_with(|| {
            self.version.cmp(&other.version)
        })
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
#[derive(Clone)]
pub struct Package {
    /// The package's name.
    name: String,
    /// The package's version.
    version: Version,

    /// The number of bytes to download for this package.
    size: u64,
    /// The package's description.
    description: String,

    /// Tells whether the package is installed.
    installed: bool,

    /// Dependencies required to build the package.
    build_deps: Vec<Dependency>,
    /// Dependencies required to run the package.
    run_deps: Vec<Dependency>,
}

impl Package {
    /// Returns the package with name `name` and version `version`.
    /// If the package doesn't exist, the function returns None.
    pub fn get(_name: &String, _version: &Version) -> Option<Self> {
        // TODO
        None
    }

    /// Returns the latest version of the package with name `name`.
    /// If the package doesn't exist, the function returns None.
    pub fn get_latest(_name: &String) -> Option<Self> {
        // TODO
        None
    }

    /// Returns the installed package with name `name`.
    /// If the package isn't installed, the function returns None.
    pub fn get_installed(_name: &String) -> Option<Self> {
        // TODO
        None
    }

    /// Returns the latest version for the current package.
    pub fn get_latest_version(&self) -> Version {
        // TODO
        todo!();
    }

    /// Returns the name of the package.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Returns the version of the package.
    pub fn get_version(&self) -> &Version {
        &self.version
    }

    /// Returns the list of available versions for the current package.
    pub fn get_versions(&self) -> Vec<Version> {
        // TODO
        todo!();
    }

    /// Returns the download size of the package.
    pub fn get_size(&self) -> u64 {
        self.size
    }

    /// Tells whether the package is installed on the system.
    pub fn is_installed(&self) -> bool {
        self.installed
    }

    /// Tells whether the package is up to date.
    pub fn is_up_to_date(&self) -> bool {
        self.version >= self.get_latest_version()
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
    /// The function makes use of packages that are already in the HashMap to determine if there is
    /// a dependency error.
    /// If an error occurs, the function returns `false`.
    pub fn resolve_dependencies(&self, packages: &mut HashMap<String, Self>) -> bool {
        // Tells whether dependencies are valid
        let mut valid = true;

        for d in &self.run_deps {
            // Checking for version conflict if the package is already in the list
            if let Some(p) = packages.get(d.get_name()) {
                // If versions don't correspond, error
                if d.get_version() != p.get_version() {
                    eprintln!("Conflicting version `{}` and `{}` on `{}`!",
                        d.get_version(), p.get_version(), d.get_name());
                    valid = false;
                }
            } else {
                // Resolving the package, then resolving its dependencies
                if let Some(p) = d.get_package() {
                    p.resolve_dependencies(packages);
                    packages.insert(p.get_name().clone(), p);
                } else {
                    eprintln!("Unresolved dependency `{}` version `{}`!",
                        d.get_name(), d.get_version());
                    valid = false;
                }
            }
        }

        valid
    }

    /// Returns the path to the cache file for this package.
    pub fn get_cache_path(&self) -> String {
        format!("{}/{}-{}", CACHE_DIR, self.name, self.version)
    }

    /// Tells whether the package is in cache.
    pub fn is_in_cache(&self) -> bool {
        Path::new(&self.get_cache_path()).exists()
    }

    // TODO Make async
    /// Downloads the package. If the package is already in cache, the function does nothing.
    pub fn download(&self) {
        if self.is_in_cache() {
            return;
        }

        // TODO
    }
}
