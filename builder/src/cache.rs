//! Packages sources cache.

use base64::{prelude::BASE64_STANDARD, Engine};
use file_lock::{FileLock, FileOptions};
use sha2::{Digest, Sha256};
use std::{
	env, fs,
	fs::File,
	io,
	io::{Read, Seek, SeekFrom, Write},
	path::{Path, PathBuf},
};

fn cache_directory() -> io::Result<PathBuf> {
	// TODO handle error
	let home = env::var_os("HOME").unwrap();
	// Create the cache directory if not present
	let dir_path = PathBuf::from(home).join(".cache/blimp/sources");
	fs::create_dir_all(&dir_path)?;
	Ok(dir_path)
}

fn compute_checksum(file: &mut FileLock) -> io::Result<[u8; 32]> {
	let mut hasher = Sha256::new();
	file.file.seek(SeekFrom::Start(0))?;
	const BUF_SIZE: usize = 4096;
	let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	loop {
		let l = file.file.read(&mut buf)?;
		if l == 0 {
			break;
		}
		hasher.write_all(&buf[..l])?;
	}
	let hash = hasher.finalize();
	Ok(hash[..].try_into().unwrap())
}

fn verify_checksum(dir_path: &Path, encoded_key: &str, checksum: &[u8]) -> io::Result<bool> {
	// `.` is not part of the base64 character set
	let path = dir_path.join(format!("{encoded_key}.checksum"));
	// Open file
	let opt = FileOptions::new().read(true);
	let res = FileLock::lock(path, true, opt);
	let mut file = match res {
		Ok(f) => f,
		Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
		Err(e) => return Err(e),
	};
	// Read file
	let mut buf = Vec::new();
	file.file.read_to_end(&mut buf)?;
	// Compare
	Ok(buf == checksum)
}

/// Entry in the cache.
pub struct CacheEntry {
	encoded_key: String,
	file: FileLock,
	cached: bool,
}

impl CacheEntry {
	/// Returns a reference to the inner file.
	#[inline]
	pub fn file(&mut self) -> &mut File {
		&mut self.file.file
	}

	/// Tells whether the entry already existed before it was fetched.
	#[inline]
	pub fn cached(&self) -> bool {
		self.cached
	}

	/// Flushes the entry to the cache by computing and storing its checksum.
	pub fn flush(&mut self) -> io::Result<()> {
		let dir_path = cache_directory()?;
		let path = dir_path.join(&self.encoded_key);
		// `.` is not part of the base64 character set
		let checksum_path = dir_path.join(format!("{}.checksum", self.encoded_key));
		// Compute checksum
		let checksum = compute_checksum(&mut self.file)?;
		// Open checksum file
		let opt = FileOptions::new().write(true).create(true).truncate(true);
		let mut file = FileLock::lock(checksum_path, true, opt)?;
		file.file.write_all(&checksum)?;
		Ok(())
	}
}

/// Retrieves or insert the entry with the given `key`.
///
/// The function returns the entry's file, along with a boolean indicating whether the file existed
/// before.
pub fn get_or_insert(key: &[u8]) -> io::Result<CacheEntry> {
	let dir_path = cache_directory()?;
	let encoded_key = BASE64_STANDARD.encode(key);
	let path = dir_path.join(&encoded_key);
	// Open file
	let opt = FileOptions::new().read(true).write(true).create(true);
	let mut file = FileLock::lock(path, true, opt)?;
	// Verify checksum
	let checksum = compute_checksum(&mut file)?;
	let valid = verify_checksum(&dir_path, &encoded_key, &checksum)?;
	if !valid {
		// Invalid checksum. Truncate file
		file.file.set_len(0)?;
		file.file.seek(SeekFrom::Start(0))?;
	}
	Ok(CacheEntry {
		encoded_key,
		file,
		cached: valid,
	})
}
