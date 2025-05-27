//! This module handles files download.

use crate::USER_AGENT;
use anyhow::Result;
use bytes::Bytes;
use futures_util::stream::{Stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs::File, io::Write, pin::Pin};

/// A download task, running until the file has been downloaded entirely.
pub struct DownloadTask<'f> {
	/// The response byte stream.
	stream: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
	/// The destination file.
	file: &'f File,

	/// The current downloaded size in bytes.
	cur_size: u64,
	/// Download progress bar.
	progress_bar: ProgressBar,
}

impl<'f> DownloadTask<'f> {
	/// Creates a new task.
	///
	/// Arguments:
	/// - `url` is the URL to download the file from
	/// - `file` is the file where the data is to be written
	pub async fn new(url: &str, file: &'f File) -> Result<Self> {
		let client = reqwest::Client::new();
		let response = client
			.get(url)
			.header("User-Agent", USER_AGENT)
			.send()
			.await?;
		// Truncate file
		file.set_len(0)?;
		// Setup progress bar
		let progress_bar = response
			.content_length()
			.map(ProgressBar::new)
			.unwrap_or_else(ProgressBar::no_length);
		let progress_style = ProgressStyle::with_template(
			"[{elapsed_precise}] {bar:40.cyan/blue} {decimal_bytes}/{decimal_total_bytes}   {percent}%",
		)
		.unwrap()
		.progress_chars("=> ");
		progress_bar.set_style(progress_style);
		Ok(Self {
			stream: Box::pin(response.bytes_stream()),
			file,
			cur_size: 0,
			progress_bar,
		})
	}

	/// Pulls the next chunk of data and returns the number of bytes downloaded.
	pub async fn next(&mut self) -> Result<usize> {
		let Some(chunk) = self.stream.next().await else {
			self.progress_bar.finish();
			return Ok(0);
		};
		let chunk = chunk?;
		if chunk.is_empty() {
			self.progress_bar.finish();
			return Ok(0);
		}
		self.cur_size += chunk.len() as u64;
		self.file.write_all(&chunk)?;
		self.progress_bar.set_position(self.cur_size);
		Ok(chunk.len())
	}
}
