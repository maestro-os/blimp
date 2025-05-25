//! This module handles files download.

use crate::USER_AGENT;
use anyhow::Result;
use bytes::Bytes;
use futures_util::stream::{Stream, StreamExt};
use std::{
	fs::{File, OpenOptions},
	io::Write,
	path::Path,
	pin::Pin,
};

/// A download task, running until the file has been downloaded entirely.
pub struct DownloadTask {
	/// The response byte stream.
	stream: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
	/// The destination file.
	file: File,

	/// The total size to be downloaded in bytes. If unknown, the value is None.
	total_size: Option<u64>,
	/// The current downloaded size in bytes.
	curr_size: u64,
}

impl DownloadTask {
	/// Creates a new task.
	///
	/// Arguments:
	/// - `url` is the URL to download the file from.
	/// - `path` is the path to which the file has to be saved.
	pub async fn new(url: &str, path: &Path) -> Result<Self> {
		let client = reqwest::Client::new();
		let response = client
			.get(url)
			.header("User-Agent", USER_AGENT)
			.send()
			.await?;
		let file = OpenOptions::new()
			.create(true)
			.write(true)
			.truncate(true)
			.open(path)?;
		let total_size = response.content_length();
		Ok(Self {
			stream: Box::pin(response.bytes_stream()),
			file,
			total_size,
			curr_size: 0,
		})
	}

	/// Returns the total size if known.
	pub fn get_total_size(&self) -> Option<u64> {
		self.total_size
	}

	/// Returns the downloaded size in bytes.
	pub fn get_current_size(&self) -> u64 {
		self.curr_size
	}

	/// Pulls the next chunk of data and returns the number of bytes downloaded.
	pub async fn next(&mut self) -> Result<usize> {
		if let Some(chunk) = self.stream.next().await {
			let chunk = chunk?;
			self.curr_size += chunk.len() as u64;
			self.file.write_all(&chunk)?;
			Ok(chunk.len())
		} else {
			Ok(0)
		}
	}
}

// TODO File integrity verification
