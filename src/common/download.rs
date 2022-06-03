//! This module handles files download.

use futures_util::stream::StreamExt;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;

/// Downloads the file at URL `url` and places at the given path `path`.
pub async fn download_file(url: &str, path: &str) -> Result<(), Box<dyn Error>> {
	let response = reqwest::get(url).await?;
	let mut stream = response.bytes_stream();

	let mut file = OpenOptions::new()
		.write(true)
		.truncate(true)
		.open(path)?;

	// TODO Progress bar
	while let Some(chunk) = stream.next().await {
		file.write(&chunk?)?;
	}

	Ok(())
}
