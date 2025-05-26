//! Packages sources cache.

use crate::desc::SourceRemote;
use common::{serde_json, util::create_tmp_file};
use std::{
	collections::HashMap,
	env, fs,
	fs::{File, OpenOptions},
	io,
	io::{Read, Write},
	path::PathBuf,
};

// TODO metadata file locking
/// Creates a new entry in the cache.
pub fn new(remote: &SourceRemote) -> io::Result<(PathBuf, File)> {
	// TODO handle error
	let home = env::var_os("HOME").unwrap();
	// Create the cache directory if not present
	let dir_path = PathBuf::from(home).join(".cache/blimp/sources");
	fs::create_dir_all(&dir_path)?;
	// Open metadata file
	let metadata_path = dir_path.join("metadata.json");
	let mut metadata_file = OpenOptions::new()
		.create(true)
		.write(true)
		.open(metadata_path)?;
	// Read metadata
	let mut metadata = String::new();
	metadata_file.read_to_string(&mut metadata)?;
	let mut metadata: HashMap<SourceRemote, &str> =
		serde_json::from_str(&metadata).unwrap_or_default();
	// Create file and insert entry
	let cache_file = create_tmp_file(&dir_path)?;
	let cache_file_name = cache_file.0.file_name().unwrap().to_str().unwrap();
	metadata.insert(remote.clone(), cache_file_name);
	// Writeback
	let metadata = serde_json::to_string(&metadata)?;
	metadata_file.write_all(metadata.as_bytes())?;
	Ok(cache_file)
}
