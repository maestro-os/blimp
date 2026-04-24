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

use crate::{
	package::Package,
	repository::{Index, PackagesWithRepositoryVec, Repository},
	Environment, REMOTES, REMOTES_LIST, USER_AGENT,
};
use anyhow::{anyhow, bail, Result};
use reqwest::StatusCode;
use std::{
	borrow::Borrow,
	collections::HashSet,
	fs,
	fs::{File, OpenOptions},
	io,
	io::{BufRead, BufReader, BufWriter, Write},
};

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
		let path = env.sysroot().join(REMOTES_LIST);
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
		let path = env.sysroot().join(REMOTES_LIST);
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

	/// Returns the repository associated with the remote.
	pub fn load_repository(&self, env: &Environment) -> io::Result<Repository> {
		let path = env.sysroot().join(format!("{REMOTES}/{}", self.host));
		fs::create_dir_all(&path)?;
		Ok(Repository {
			path,
			remote: Some(self.clone()),
		})
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
			StatusCode::OK => Ok(Some(response.text().await?)),
			StatusCode::NOT_FOUND => Ok(None),
			_ => bail!("failed to retrieve remote metadata (status {status})"),
		}
	}

	/// Fetches the remote's index
	///
	/// The function returns the number of packages found
	pub async fn fetch_index(&self, env: &Environment) -> Result<usize> {
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
		// Check the index is valid and get package count
		let index = response.text().await?;
		let parsed_index: Index = toml::from_str(&index)?;
		let cnt = parsed_index
			.arch
			.get(env.arch())
			.map(|a| a.package.len())
			.unwrap_or(0);
		// Write to file
		let repo = self.load_repository(env)?;
		fs::write(repo.get_index_path(), index)?;
		Ok(cnt)
	}

	/// Returns the download URL for the given `package`.
	pub fn download_url(&self, arch: &str, package: &Package) -> String {
		format!(
			"https://{}/dist/{}/{}_{}.tar.gz",
			self.host, arch, package.name, package.version
		)
	}

	/// Returns the download size of the package `package` in bytes.
	pub async fn get_size(&self, arch: &str, package: &Package) -> Result<u64> {
		let client = reqwest::Client::new();
		client
			.head(self.download_url(arch, package))
			.header("User-Agent", USER_AGENT)
			.send()
			.await?
			.content_length()
			.ok_or_else(|| anyhow!("Content-Length field not present in response"))
	}
}

/// Download packages and print in case of cache or failure.
///
/// Arguments:
/// - `total_packages` is the whole list of packages to install
/// - `arch` is the environment to install on
pub async fn download_packages<'r>(
	total_packages: &PackagesWithRepositoryVec<'r>,
	arch: &str,
) -> Result<()> {
	let mut failed = false;
	let mut futures = Vec::new();
	// TODO download biggest packages first (sort_unstable by decreasing size)
	for (pkg, repo) in total_packages {
		if repo.is_in_cache(arch, &pkg.name, &pkg.version) {
			println!("`{}` is in cache.", &pkg.name);
			continue;
		}
		if let Some(remote) = repo.get_remote() {
			// TODO limit the number of packages downloaded concurrently
			futures.push((
				&pkg.name,
				&pkg.version,
				// TODO spawn task
				async {
					use crate::download::DownloadTask;
					use std::fs::File;

					let path = repo.get_archive_path(arch, &pkg.name, &pkg.version);
					// Ensure the parent directory exists
					if let Some(parent) = path.parent() {
						fs::create_dir_all(parent)?;
					}
					// Download
					let file = File::create(path)?;
					let url = remote.download_url(arch, pkg);
					let mut task = DownloadTask::new(&url, &file).await?;
					while task.next().await? > 0 {}
					Ok::<(), anyhow::Error>(())
				},
			));
		}
	}
	for (name, version, f) in futures {
		match f.await {
			Ok(()) => continue,
			Err(error) => {
				eprintln!("Failed to download `{name}` version `{version}`: {error}");
				failed = true;
			}
		}
	}
	if failed {
		bail!("installation failed");
	}
	Ok(())
}
