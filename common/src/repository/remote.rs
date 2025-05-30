//! A remote is a remote host from which packages can be downloaded.

use crate::{package::Package, Environment, USER_AGENT};
use anyhow::{anyhow, bail, Result};
use reqwest::StatusCode;
use std::{
	borrow::Borrow,
	collections::HashSet,
	fs::{File, OpenOptions},
	io,
	io::{BufRead, BufReader, BufWriter, Write},
};

/// The file which contains the list of remotes.
const REMOTES_FILE: &str = "var/lib/blimp/remotes_list";

/// A remote host.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Remote {
	/// The host's address and port (optional).
	pub host: String,
}

impl Borrow<str> for Remote {
	fn borrow(&self) -> &str {
		&self.host
	}
}

impl Remote {
	/// Loads and returns the list of remote hosts.
	pub fn load_list(env: &Environment) -> io::Result<HashSet<Self>> {
		let path = env.sysroot().join(REMOTES_FILE);
		let file = File::open(path)?;
		let reader = BufReader::new(file);
		reader
			.lines()
			.map(|host| {
				host.map(|host| Self {
					host,
				})
			})
			.collect()
	}

	/// Saves the list of remote hosts.
	pub fn save_list(env: &Environment, remotes: impl Iterator<Item = Remote>) -> io::Result<()> {
		let path = env.sysroot().join(REMOTES_FILE);
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.truncate(true)
			.open(path)?;
		let mut writer = BufWriter::new(file);
		for r in remotes {
			writer.write_all(r.host.as_bytes())?;
			writer.write_all(b"\n")?;
		}
		Ok(())
	}

	/// Returns the remote's motd.
	pub async fn fetch_motd(&self) -> Result<String> {
		let client = reqwest::Client::new();
		let url = format!("https://{}/motd", &self.host);
		let response = client
			.get(url)
			.header("User-Agent", USER_AGENT)
			.send()
			.await?;
		let status = response.status();
		match status {
			StatusCode::OK => Ok(response.text().await?),
			_ => bail!("Failed to retrieve motd (status {status})"),
		}
	}

	/// Fetches the list of all the packages from the remote.
	pub async fn fetch_list(&self) -> Result<Vec<Package>> {
		let client = reqwest::Client::new();
		let url = format!("https://{}/package", self.host);
		let response = client
			.get(url)
			.header("User-Agent", USER_AGENT)
			.send()
			.await?;
		let status = response.status();
		match status {
			StatusCode::OK => Ok(response.json().await?),
			_ => bail!("Failed to retrieve packages list from remote (status {status})"),
		}
	}

	/// Returns the download URL for the given `package`.
	pub fn download_url(&self, package: &Package) -> String {
		format!(
			"https://{}/package/{}/version/{}/archive",
			self.host, package.name, package.version
		)
	}

	/// Returns the download size of the package `package` in bytes.
	pub async fn get_size(&self, package: &Package) -> Result<u64> {
		let client = reqwest::Client::new();
		client
			.head(self.download_url(package))
			.header("User-Agent", USER_AGENT)
			.send()
			.await?
			.content_length()
			.ok_or_else(|| anyhow!("Content-Length field not present in response"))
	}
}
