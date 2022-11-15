//! A remote is a remote host from which packages can be downloaded.

use crate::download::DownloadTask;
use crate::package::Package;
use crate::repository::Repository;
use crate::request::PackageListResponse;
use crate::request::PackageSizeResponse;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;

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
	/// `sysroot` is the path to the system's root.
	pub fn load_list(sysroot: &str) -> io::Result<Vec<Self>> {
		let path = format!("{}/{}", sysroot, REMOTES_FILE);
		let file = File::open(path)?;
		let reader = BufReader::new(file);

		reader
			.lines()
			.map(|s| Ok(Self::new(s?)))
			.collect::<io::Result<Vec<Self>>>()
	}

	/// Saves the list of remote hosts.
	pub fn save_list(sysroot: &str, remotes: &[Self]) -> io::Result<()> {
		let path = format!("{}/{}", sysroot, REMOTES_FILE);
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.truncate(true)
			.open(path)?;
		let mut writer = BufWriter::new(file);
		for r in remotes {
			writer.write(r.get_host().as_bytes())?;
			writer.write(b"\n")?;
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
	/// `sysroot` is the path to the system's root.
	pub async fn fetch_list(&self, sysroot: &str) -> Result<Vec<Package>, Box<dyn Error>> {
		let url = format!("http://{}/package", &self.host);
		let response = reqwest::get(url).await?;
		let status = response.status();
		let content = response.text().await?;

		match status {
			reqwest::StatusCode::OK => {
				let json: PackageListResponse = serde_json::from_str(&content)?;
				Ok(json.packages)
			}

			_ => Err(format!("Failed to retrieve packages list from remote: {}", status).into()),
		}
	}

	/// Returns the download size of the package `package` in bytes.
	pub async fn get_size(&self, package: &Package) -> Result<u64, String> {
		let url = format!(
			"http://{}/package/{}/version/{}/size",
			self.host,
			package.get_name(),
			package.get_version()
		);
		let response = reqwest::get(url)
			.await
			.or_else(|e| Err(format!("HTTP request failed: {}", e)))?;
		let content = response
			.text()
			.await
			.or_else(|e| Err(format!("HTTP request failed: {}", e)))?;

		let json: PackageSizeResponse = serde_json::from_str(&content)
			.or_else(|e| Err(format!("Failed to parse JSON response: {}", e)))?;
		Ok(json.size)
	}

	/// Downloads the archive of package `package` to the given repository `repo`.
	///
	/// Arguments:
	/// `sysroot` is the path to the system's root.
	pub async fn fetch_archive(
		&self,
		repo: &Repository,
		package: &Package,
	) -> Result<DownloadTask, Box<dyn Error>> {
		let url = format!(
			"http://{}/package/{}/version/{}/archive",
			self.host,
			package.get_name(),
			package.get_version()
		);

		let path = repo.get_cache_archive_path(package);
		DownloadTask::new(&url, &path).await
	}
}
