//! This module handles the server's configuration file.

use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
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
        let file = File::open(CONFIG_FILE)?;
        let reader = BufReader::new(file);

        Ok(serde_json::from_reader(reader)?)
    }

    pub fn write(&self) -> io::Result<()> {
        let file = File::create(CONFIG_FILE)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)?;

        Ok(())
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
