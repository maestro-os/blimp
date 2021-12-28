//! A package is a software that can be installed using the package manager.
//! Packages are usualy downloaded from a remote host.

use crate::util;
use crate::version::Version;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::fs;
use std::io::BufReader;
use std::io;
use std::path::Path;

/// The directory containing cached packages.
const CACHE_DIR: &str = "/usr/lib/blimp/cache";

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
    /// Lists available packages on the server. If not on a server, the function is undefined.
    pub fn server_list() -> io::Result<Vec<Self>> {
        let mut packages = Vec::new();

        let files = fs::read_dir(SERVER_PACKAGES_DESC_DIR)?;
        for p in files {
            let path = p?.path();

            let file = File::open(path)?;
            let reader = BufReader::new(file);
            packages.push(serde_json::from_reader(reader)?);
        }

        Ok(packages)
    }

    /// Returns the package with name `name` and version `version`.
    /// `server` tells whether the function runs on serverside.
    /// If the package doesn't exist, the function returns None.
    pub fn get(name: &String, version: &Version, server: bool) -> io::Result<Option<Self>> {
        if server {
            let desc_path = SERVER_PACKAGES_DESC_DIR.to_owned() + "/" + name + "-"
                + &version.to_string();

            if let Ok(file) = File::open(desc_path) {
                let reader = BufReader::new(file);
                Ok(Some(serde_json::from_reader(reader)?))
            } else {
                Ok(None)
            }
        } else {
            // TODO
            Ok(None)
        }
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

    /// Tells whether the package is installed on the system.
    pub fn is_installed(&self) -> bool {
        // TODO
        todo!();
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
    /// `f` is a function used to get a package from its name and version. If the package doesn't
    /// exist, the function returns None.
    /// The function makes use of packages that are already in the HashMap and those which are
    /// already installed to determine if there is a dependency error.
    /// If an error occurs, the function returns `false`.
    pub fn resolve_dependencies<F>(&self, packages: &mut HashMap<String, Self>, f: &mut F) -> bool
        where F: FnMut(String, Version) -> Option<Self> {
        // Tells whether dependencies are valid
        let mut valid = true;

        for d in &self.run_deps {
            // Getting the package. Either in the installation list or already installed
            let pkg = Self::get_installed(d.get_name()).or_else(|| {
                Some(packages.get(d.get_name())?.clone())
            });

            // Checking for version conflict
            if let Some(p) = pkg {
                // If versions don't correspond, error
                if d.get_version() != p.get_version() {
                    eprintln!("Conflicting version `{}` and `{}` on `{}`!",
                        d.get_version(), p.get_version(), d.get_name());
                    valid = false;
                }

                continue;
            }

            // Resolving the package, then resolving its dependencies
            if let Some(p) = f(d.get_name().clone(), d.get_version().clone()) {
                p.resolve_dependencies(packages, f); // FIXME Possible stack overflow
                packages.insert(p.get_name().clone(), p);
            } else {
                eprintln!("Unresolved dependency `{}` version `{}`!",
                    d.get_name(), d.get_version());
                valid = false;
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

    /// Installs the package. If the package is already installed, the function does nothing.
    /// `sysroot` is the root of the system at which the package is to be installed.
    /// The function assumes the running dependencies of the package are already installed.
    pub fn install(&self, sysroot: String) -> io::Result<()> {
        if self.is_installed() {
            return Ok(());
        }

        // Uncompressing the package
        util::uncompress_wrap(&self.get_cache_path(), | tmp_dir | {
            // Running the pre-install hook
            if !util::run_hook(&format!("{}/pre-install-hook", tmp_dir), &sysroot)? {
                return Err(io::Error::new(io::ErrorKind::Other, "Pre-install hook failed!"));
            }

            // Uncompresses data at sysroot, installing the package
            let data_path = format!("{}/data.tar.gz", tmp_dir);
            util::uncompress(&data_path, &sysroot)?;

            if !util::run_hook(&format!("{}/post-install-hook", tmp_dir), &sysroot)? {
                return Err(io::Error::new(io::ErrorKind::Other, "Post-install hook failed!"));
            }

            Ok(())
        })?
    }

    /// Upgrades the package.
    /// `sysroot` is the root of the system at which the package is to be removed.
    pub fn upgrade(&self, sysroot: String) -> io::Result<()> {
        // Uncompressing the package
        util::uncompress_wrap(&self.get_cache_path(), | tmp_dir | {
            // Running the pre-upgrade hook
            if !util::run_hook(&format!("{}/pre-upgrade-hook", tmp_dir), &sysroot)? {
                return Err(io::Error::new(io::ErrorKind::Other, "Pre-upgrade hook failed!"));
            }

            // TODO Patch files corresponding to the ones in inner data archive

            // Running the post-upgrade hook
            if !util::run_hook(&format!("{}/post-upgrade-hook", tmp_dir), &sysroot)? {
                return Err(io::Error::new(io::ErrorKind::Other, "Post-upgrade hook failed!"));
            }

            Ok(())
        })?
    }

    /// Removes the package.
    /// `sysroot` is the root of the system at which the package is to be removed.
    pub fn remove(&self, sysroot: String) -> io::Result<()> {
        // Uncompressing the package
        util::uncompress_wrap(&self.get_cache_path(), | tmp_dir | {
            // Running the pre-remove hook
            if !util::run_hook(&format!("{}/pre-remove-hook", tmp_dir), &sysroot)? {
                return Err(io::Error::new(io::ErrorKind::Other, "Pre-remove hook failed!"));
            }

            // TODO Remove files corresponding to the ones in inner data archive

            // Running the post-remove hook
            if !util::run_hook(&format!("{}/post-remove-hook", tmp_dir), &sysroot)? {
                return Err(io::Error::new(io::ErrorKind::Other, "Post-remove hook failed!"));
            }

            Ok(())
        })?
    }
}
