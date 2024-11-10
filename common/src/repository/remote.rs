//! A remote is a remote host from which packages can be downloaded.

use crate::{download::DownloadTask, package::Package, repository::Repository, Environment};
use anyhow::{anyhow, Result};
use std::{
	fs::{File, OpenOptions},
	io,
	io::{BufRead, BufReader, BufWriter, Write},
};

// TODO Use https

/// The file which contains the list of remotes.
const REMOTES_FILE: &str = "/usr/lib/blimp/remotes_list";

/// Structure representing a remote host.
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Remote {
	/// The host's address and port (optional).
	host: String,
}

impl Remote {
	/// Creates a new instance.
	pub fn new(host: String) -> Self {
		Self {
			host,
		}
	}

	/// Loads and returns the list of remote hosts.
	pub fn load_list(env: &Environment) -> io::Result<Vec<Self>> {
		let path = env.get_sysroot().join(REMOTES_FILE);
		let file = File::open(path)?;
		let reader = BufReader::new(file);

		reader
			.lines()
			.map(|s| Ok(Self::new(s?)))
			.collect::<io::Result<Vec<Self>>>()
	}

	/// Saves the list of remote hosts.
	pub fn save_list(env: &Environment, remotes: &[Self]) -> io::Result<()> {
		let path = env.get_sysroot().join(REMOTES_FILE);
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.truncate(true)
			.open(path)?;
		let mut writer = BufWriter::new(file);
		for r in remotes {
			writer.write_all(r.get_host().as_bytes())?;
			writer.write_all(b"\n")?;
		}

		Ok(())
	}

	/// Returns the host for the remote.
	pub fn get_host(&self) -> &str {
		&self.host
	}

	/// Returns the remote's motd.
	pub fn get_motd(&self) -> Result<String, String> {
		let url = format!("http://{}/motd", &self.host);
		let response = reqwest::blocking::get(url).or(Err("HTTP request failed"))?;
		let status = response.status();
		let content = response.text().or(Err("HTTP request failed"))?;

		match status {
			reqwest::StatusCode::OK => Ok(content),

			_ => Err(format!("Failed to retrieve motd: {}", status)),
		}
	}

	/// Fetches the list of all the packages from the remote.
	pub async fn fetch_list(&self) -> Result<Vec<Package>> {
		let url = format!("http://{}/package", &self.host);
		let response = reqwest::get(url).await?;
		let status = response.status();
		let content = response.text().await?;

		match status {
			reqwest::StatusCode::OK => Ok(serde_json::from_str(&content)?),

			_ => Err(anyhow!(
				"Failed to retrieve packages list from remote: {}",
				status
			)),
		}
	}

	/// Returns the download size of the package `package` in bytes.
	pub async fn get_size(&self, package: &Package) -> Result<u64> {
		let url = format!(
			"http://{}/package/{}/version/{}/archive",
			self.host,
			package.get_name(),
			package.get_version()
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
			"http://{}/package/{}/version/{}/archive",
			self.host,
			package.get_name(),
			package.get_version()
		);

		let path = repo.get_archive_path(package.get_name(), package.get_version());
		DownloadTask::new(&url, &path).await.map_err(Into::into)
	}
}
