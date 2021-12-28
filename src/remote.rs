//! A remote is a remote host from which packages can be downloaded.

use common::package::Package;
use common::request::PackageListResponse;
use common::request::PackageSizeResponse;
use common::version::Version;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::io;

/// The file which contains the list of remotes.
const REMOTES_FILE: &str = "/usr/lib/blimp/remotes_list";
/// The path to the database storing the list of packages for every remotes.
const DATABASE_PATH: &str = "/usr/lib/blimp/database";

/// Structure representing a remote host.
#[derive(Clone)]
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

    /// Returns the path to the remote's package database.
    pub fn get_database_path(&self) -> String {
        format!("{}/{}", DATABASE_PATH, self.get_host())
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

    /// Fetches the list of all the packages from the remote.
    /// `save` tells whether the result of the request must be saved in the database if the request
    /// succeeded.
    pub fn fetch_all(&self, save: bool) -> Result<Vec<Package>, String> {
        let url = format!("http://{}/package", &self.host);
        let response = reqwest::blocking::get(url).or(Err("HTTP request failed"))?;
        let status = response.status();
        let content = response.text().or(Err("HTTP request failed"))?;

        match status {
            reqwest::StatusCode::OK => {
                let json: PackageListResponse = serde_json::from_str(&content)
                    .or(Err("Failed to parse JSON response"))?;

                if save {
                    let file = File::create(self.get_database_path()).or(Err("TODO"))?; // TODO
                    let writer = BufWriter::new(file);
                    serde_json::to_writer_pretty(writer, &json).or(Err("TODO"))?; // TODO
                }

                Ok(json.packages)
            },

            _ => Err("TODO".to_string()), // TODO
        }
    }

    /// Returns the package with the given name `name` and version `version`.
    /// If the package doesn't exist on the remote, the function returns None.
    pub fn get_package(name: &str, version: &Version) -> io::Result<Option<(Self, Package)>> {
        // Iterating over remotes
        for r in Remote::list()? {
            let file = File::open(r.get_database_path())?;
            let reader = BufReader::new(file);

            let json: PackageListResponse = serde_json::from_reader(reader)?;

            // Iterating over packages on the remote
            for p in json.packages {
                if p.get_name() == name && p.get_version() == version {
                    return Ok(Some((r, p)));
                }
            }
        }

        Ok(None)
    }

    /// Returns the latest version of the package with name `name`.
    /// If the package doesn't exist, the function returns None.
    pub fn get_latest(name: &String) -> io::Result<Option<(Self, Package)>> {
        // Iterating over remotes
        for r in Remote::list()? {
            let file = File::open(r.get_database_path())?;
            let reader = BufReader::new(file);

            let json: PackageListResponse = serde_json::from_reader(reader)?;

            // TODO Take highest version
            // Iterating over packages on the remote
            for p in json.packages {
                if p.get_name() == name {
                    return Ok(Some((r, p)));
                }
            }
        }

        Ok(None)
    }

    /// Returns the download size of the package `package` in bytes.
    pub async fn get_size(&self, package: &Package) -> Result<u64, String> {
        let url = format!("http://{}/package/{}/version/{}/size",
            self.host, package.get_name(), package.get_version());
        let response = reqwest::get(url).await.or(Err("HTTP request failed"))?;
        let content = response.text().await.or(Err("HTTP request failed"))?;

        let json: PackageSizeResponse = serde_json::from_str(&content)
            .or(Err("Failed to parse JSON response"))?;
        Ok(json.size)
    }

    // TODO Do not keep the whole file in RAM before writting
    /// Downloads the package `package` and writes it in cache.
    pub async fn download(&self, package: &Package) -> Result<(), String> {
        let url = format!("http://{}/package/{}/version/{}/archive",
            self.host, package.get_name(), package.get_version());
        let response = reqwest::get(url).await.or(Err("HTTP request failed"))?;
        let content = response.bytes().await.or(Err("HTTP request failed"))?;

        let mut file = File::create(package.get_cache_path())
            .or(Err("Failed to create cache file"))?;
        file.write(&content).or(Err("IO error"))?;

        Ok(())
    }

    // TODO serialize function
}
