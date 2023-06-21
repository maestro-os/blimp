//! This module handles files download.

use bytes::Bytes;
use futures_util::stream::Stream;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::pin::Pin;

/// A download task, running until the file has been downloaded entirely.
pub struct DownloadTask {
	/// The response byte stream.
	stream: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
	/// The total size to be downloaded in bytes. If unknown, the value is None.
	total_size: Option<u64>,

	/// The destination file.
	file: File,
	/// The current downloaded size in bytes.
	curr_size: u64,

	/// Tells whether the task is running. If `false`, downloading is over.
	running: bool,
}

impl DownloadTask {
	/// Creates a new task.
	///
	/// Arguments:
	/// - `url` is the URL to download the file from.
	/// - `path` is the path to which the file has to be saved.
	pub async fn new(url: &str, path: &Path) -> Result<Self, Box<dyn Error>> {
		let response = reqwest::get(url).await?;
		let total_size = response.content_length();
		let stream = response.bytes_stream();

		let file = OpenOptions::new()
			.create(true)
			.write(true)
			.truncate(true)
			.open(path)?;

		Ok(Self {
			stream: Box::pin(stream),
			total_size,

			file,
			curr_size: 0,

			running: true,
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

	/// Pulls the next chunk of data.
	///
	/// If the task is over, the function returns `false`.
	pub async fn next(&mut self) -> Result<bool, Box<dyn Error>> {
		let mut running = true;

		if let Some(chunk) = self.stream.next().await {
			let chunk = chunk?;
			self.curr_size += chunk.len() as u64;
			self.file.write(&chunk)?;
		} else {
			running = false;
		};

		self.running = running;
		Ok(self.running)
	}
}

// TODO File integrity verification
