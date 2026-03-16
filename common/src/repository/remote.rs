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
const REMOTES_FILE: &str = "var/lib/blimp/remotes";

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
		let file = match File::open(path) {
			Ok(file) => file,
			Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(HashSet::new()),
			Err(e) => return Err(e),
		};
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

	/// Fetches the remote's motd
	pub async fn fetch_motd(&self) -> Result<Option<String>> {
		let client = reqwest::Client::new();
		let url = format!("https://{}/motd", &self.host);
		let response = client
			.get(url)
			.header("User-Agent", USER_AGENT)
			.send()
			.await?;
		let status = response.status();
		match status {
			StatusCode::OK => {
				let s = response.text().await?;
				let metadata = toml::from_str(&s)?;
				Ok(Some(metadata))
			}
			StatusCode::NOT_FOUND => Ok(None),
			_ => bail!("failed to retrieve remote metadata (status {status})"),
		}
	}

	/// Fetches the remote's index
	pub async fn fetch_index(&self) -> Result<Vec<Package>> {
		let client = reqwest::Client::new();
		let url = format!("https://{}/index", self.host);
		let response = client
			.get(url)
			.header("User-Agent", USER_AGENT)
			.send()
			.await?;
		let status = response.status();
		if !status.is_success() {
			bail!("Failed to retrieve packages list from remote (status {status})");
		}
		todo!()
	}

	/// Returns the download URL for the given `package`.
	pub fn download_url(&self, env: &Environment, package: &Package) -> String {
		format!(
			"https://{}/dist/{}/{}_{}.tar.gz",
			self.host, env.arch, package.name, package.version
		)
	}

	/// Returns the download size of the package `package` in bytes.
	pub async fn get_size(&self, env: &Environment, package: &Package) -> Result<u64> {
		let client = reqwest::Client::new();
		client
			.head(self.download_url(env, package))
			.header("User-Agent", USER_AGENT)
			.send()
			.await?
			.content_length()
			.ok_or_else(|| anyhow!("Content-Length field not present in response"))
	}
}
