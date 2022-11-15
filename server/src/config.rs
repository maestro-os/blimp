//! This module handles the server's configuration file.

use common::util;
use serde::Deserialize;
use serde::Serialize;
use std::io;
use std::path::Path;

/// The path to the configuration file.
const CONFIG_FILE: &str = "config.json";

/// Structure representing the server's configuration.
#[derive(Deserialize, Serialize)]
pub struct Config {
	/// The server's port.
	pub port: u16,

	/// The server's motd.
	pub motd: String,
}

impl Config {
	/// Tells whether the configuration file exists.
	pub fn exists() -> bool {
		Path::new(CONFIG_FILE).exists()
	}

	/// Reads the configuration from file.
	pub fn read() -> io::Result<Self> {
		util::read_json(CONFIG_FILE)
	}

	pub fn write(&self) -> io::Result<()> {
		util::write_json(CONFIG_FILE, self)
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			port: 80,

			motd: "This is a dummy blimp server".to_owned(),
		}
	}
}
