//! This module implements the global data structure. 

use common::package::Package;
use crate::config::Config;
use std::io;

/// Structure storing data used all across the server.
pub struct GlobalData {
    /// The server's configuration.
    config: Config,

    /// Lazily-loaded packages list.
    packages: Vec<Package>,
}

impl GlobalData {
    /// Creates a new instance with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,

            packages: Vec::new(),
        }
    }

    /// Returns a mutable refrence to the configuration.
    pub fn get_config(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Returns a reference to the list of packages.
    /// If the list is empty, the function loads it.
    pub fn get_packages(&mut self) -> io::Result<&Vec<Package>> {
        // If the list is empty, load it
        if self.packages.is_empty() {
            self.packages = Package::server_list()?;
        }

        Ok(&self.packages)
    }
}
