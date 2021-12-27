//! A remote is a remote host from which packages can be downloaded.

use common::package::Package;
use common::request::PackageListResponse;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io;

/// The file which contains the list of remotes.
const REMOTES_FILE: &str = "/usr/lib/blimp/remotes_list";

/// Structure representing a remote host.
pub struct Remote {
    /// The host's address and port (optional).
    host: String,
}

impl Remote {
    /// Creates a new instance.
    pub fn new(host: String) -> Self {
        Self{
            host,
        }
    }

    /// Returns the list of remote hosts.
    pub fn list() -> io::Result<Vec<Self>> {
        let mut v = Vec::new();

        let file = File::open(REMOTES_FILE)?;
        let reader = BufReader::new(file);

        for l in reader.lines() {
            v.push(Self::new(l?));
        }

        Ok(v)
    }

    /// Returns the host for the remote.
    pub fn get_host(&self) -> &String {
        &self.host
    }

    /// Returns the remote's motd.
    pub fn get_motd(&self) -> Result<String, String> {
        let url = format!("http://{}/motd", &self.host);
        let response = reqwest::blocking::get(url).or(Err("HTTP request failed"))?;
        let status = response.status();
        let content = response.text().or(Err("HTTP request failed"))?;

        match status {
            reqwest::StatusCode::OK => {
                Ok(content)
            },

            _ => Err("TODO".to_string()), // TODO
        }
    }

    /// Returns the list of all packages on the remote.
    pub fn get_all(&self) -> Result<Vec<Package>, String> {
        let url = format!("http://{}/motd", &self.host);
        let response = reqwest::blocking::get(url).or(Err("HTTP request failed"))?;
        let status = response.status();
        let content = response.text().or(Err("HTTP request failed"))?;

        match status {
            reqwest::StatusCode::OK => {
                let json: PackageListResponse = serde_json::from_str(&content)
                    .or(Err("Failed to parse JSON response"))?;
                Ok(json.packages)
            },

            _ => Err("TODO".to_string()), // TODO
        }
    }

    // TODO serialize function
}
