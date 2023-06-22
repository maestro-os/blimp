//! This module handles the server's configuration file.

use common::util;
use serde::Deserialize;
use serde::Serialize;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// The path to the configuration file.
const CONFIG_FILE: &str = "config.json";

/// Structure representing the server's configuration.
#[derive(Deserialize, Serialize)]
pub struct Config {
	/// The server's port.
	pub port: u16,

	/// The server's motd.
	pub motd: String,

	/// The path to the repository containing the server's packages.
	pub repo_path: PathBuf,
}

impl Config {
	/// Reads the configuration from file.
	pub fn read() -> io::Result<Self> {
		util::read_json(Path::new(CONFIG_FILE))
	}
}
