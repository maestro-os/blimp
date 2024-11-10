//! A remote is a remote host from which packages can be downloaded.

use crate::{download::DownloadTask, package::Package, repository::Repository, Environment};
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
		let url = format!("https://{}/motd", &self.host);
		let response = reqwest::get(url).await?;
		let status = response.status();
		match status {
			StatusCode::OK => Ok(response.text().await?),
			_ => bail!("Failed to retrieve motd (status {status})"),
		}
	}

	/// Fetches the list of all the packages from the remote.
	pub async fn fetch_list(&self) -> Result<Vec<Package>> {
		let url = format!("https://{}/package", &self.host);
		let response = reqwest::get(url).await?;
		let status = response.status();
		match status {
			StatusCode::OK => Ok(response.json().await?),
			_ => bail!("Failed to retrieve packages list from remote (status {status})"),
		}
	}

	/// Returns the download size of the package `package` in bytes.
	pub async fn get_size(&self, package: &Package) -> Result<u64> {
		let url = format!(
			"https://{}/package/{}/version/{}/archive",
			self.host, package.name, package.version
		);
		let client = reqwest::Client::new();
		let response = client.head(url).send().await?;
		let len = response
			.content_length()
			.ok_or_else(|| anyhow!("Content-Length field not present in response"))?;
		Ok(len)
	}

	/// Downloads the archive of package `package` to the given repository `repo`.
	pub async fn fetch_archive(
		&self,
		repo: &Repository,
		package: &Package,
	) -> Result<DownloadTask> {
		let url = format!(
			"https://{}/package/{}/version/{}/archive",
			self.host, package.name, package.version
		);
		let path = repo.get_archive_path(&package.name, &package.version);
		DownloadTask::new(&url, &path).await.map_err(Into::into)
	}
}
