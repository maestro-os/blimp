//! This module implements utility functions.

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::os::unix;
use std::path::Component::Normal;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tar::Archive;
use xz2::read::XzDecoder;

// TODO Add a maximum try count
/// Creates a temporary directory. The function returns the path to the directory.
pub fn create_tmp_dir() -> io::Result<PathBuf> {
	let mut i = 0;

	loop {
		let path = PathBuf::from(format!("/tmp/blimp-{}", i));

		if fs::create_dir(&path).is_ok() {
			return Ok(path);
		}

		i += 1;
	}
}

// TODO Add a maximum try count
/// Creates a temporary file. The function returns the path to the file and the file itself.
pub fn create_tmp_file() -> io::Result<(PathBuf, File)> {
	let mut i = 0;

	loop {
		let path = PathBuf::from(format!("/tmp/blimp-{}", i));

		let result = OpenOptions::new()
			.read(true)
			.write(true)
			.create_new(true)
			.open(path.clone());

		if let Ok(file) = result {
			return Ok((path, file));
		}

		i += 1;
	}
}

/// TODO doc
///
/// `unwrap` tells whether the tarball shall be unwrapped.
fn uncompress_<R: Read>(mut archive: Archive<R>, dest: &Path, unwrap: bool) -> io::Result<()> {
	if unwrap {
		for entry in archive.entries()? {
			let mut entry = entry?;

			let path: PathBuf = entry
				.path()?
				.components()
				.skip(1)
				.filter(|c| matches!(c, Normal(_)))
				.collect();
			let path = dest.join(path);

			entry.unpack(path)?;
		}
	} else {
		archive.unpack(dest)?;
	}

	Ok(())
}

/// Uncompresses the given archive file `src` to the given location `dest`.
///
/// `unwrap` tells whether the tarball shall be unwrapped.
///
/// If the tarball contains directories at the root, the unwrap operation unwraps their content
/// instead of the directories themselves.
pub fn uncompress(src: &Path, dest: &Path, unwrap: bool) -> io::Result<()> {
	// Trying to uncompress .tar.gz
	{
		let file = File::open(src)?;
		let tar = GzDecoder::new(file);
		let archive = Archive::new(tar);

		if uncompress_(archive, dest, unwrap).is_ok() {
			return Ok(());
		}
	}

	// Trying to uncompress .tar.xz
	{
		let file = File::open(src)?;
		let tar = XzDecoder::new(file);
		let archive = Archive::new(tar);

		if uncompress_(archive, dest, unwrap).is_ok() {
			return Ok(());
		}
	}

	// Trying to uncompress .tar.bz2
	{
		let file = File::open(src)?;
		let tar = BzDecoder::new(file);
		let archive = Archive::new(tar);

		uncompress_(archive, dest, unwrap)
	}
}

/// Uncompresses the given .tar.gz file `archive` into a temporary directory, executes the given
/// function `f` with the path to the temporary directory as argument, then removes the directory
/// and returns the result of the call to `f`.
pub fn uncompress_wrap<T, F: FnOnce(&Path) -> T>(archive: &Path, f: F) -> io::Result<T> {
	// Uncompressing
	let tmp_dir = create_tmp_dir()?;
	uncompress(archive, &tmp_dir, false)?;

	let v = f(&tmp_dir);

	// Removing temporary directory
	fs::remove_dir_all(&tmp_dir)?;

	Ok(v)
}

/// Reads the package archive at the given path and returns an instance for it.
pub fn read_package_archive(path: &Path) -> io::Result<Archive<GzDecoder<File>>> {
	Ok(Archive::new(GzDecoder::new(File::open(path)?)))
}

/// Run the hook at the given path.
///
/// Arguments:
/// - `hook_path` is the path to the hook to be executed.
/// - `sysroot` is the sysroot.
///
/// If the hook succeeded, the function returns `true`. If it didn't, it returns `false`.
/// If the hook doesn't exist, the function does nothing and returns successfully.
pub fn run_hook(hook_path: &Path, sysroot: &Path) -> io::Result<bool> {
	if !Path::new(hook_path).exists() {
		return Ok(true);
	}

	let status = Command::new(hook_path)
		.env("SYSROOT", sysroot.as_os_str())
		.status()?;

	Ok(status.success())
}

/// Copies the content of the directory `src` to the directory `dst` recursively.
pub fn recursive_copy(src: &Path, dst: &Path) -> io::Result<()> {
	for entry in fs::read_dir(src)? {
		let entry = entry?;
		let file_type = entry.file_type()?;
		let to = dst.join(entry.file_name());

		if file_type.is_dir() {
			fs::create_dir_all(&to)?;
			recursive_copy(&entry.path(), &to)?;
		} else if file_type.is_symlink() {
			let _metadata = fs::symlink_metadata(entry.path())?;
			let target = fs::read_link(entry.path())?;

			// TODO Set timestamps and owner
			unix::fs::symlink(target, &to)?;
		} else {
			fs::copy(entry.path(), &to)?;
		}
	}

	Ok(())
}

// TODO cleanup (reuse the version in maestro-utils?)
/// Prints the given size in bytes into a human-readable form.
pub fn print_size(mut size: u64) {
	let mut level = 0;
	while level < 6 && size > 1024 {
		size /= 1024;
		level += 1;
	}

	let suffix = match level {
		0 => "bytes",
		1 => "KiB",
		2 => "MiB",
		3 => "GiB",
		4 => "TiB",
		5 => "PiB",
		6 => "EiB",

		_ => return,
	};

	print!("{size} {suffix}");
}

/// Reads a JSON file.
pub fn read_json<T: for<'a> Deserialize<'a>>(file: &Path) -> io::Result<T> {
	let file = File::open(file)?;
	let reader = BufReader::new(file);

	serde_json::from_reader(reader).map_err(|e| {
		let msg = format!("JSON deserializing failed: {}", e);
		io::Error::new(io::ErrorKind::Other, msg)
	})
}

/// Writes a JSON file.
pub fn write_json<T: Serialize>(file: &Path, data: &T) -> io::Result<()> {
	let file = File::create(file)?;
	let writer = BufWriter::new(file);

	serde_json::to_writer_pretty(writer, &data).map_err(|e| {
		let msg = format!("JSON serializing failed: {}", e);
		io::Error::new(io::ErrorKind::Other, msg)
	})
}

/// Concatenates the given paths.
///
/// If the second path begins at the root of the filesystem, this doesn't ignore the first path,
/// as opposed to the `join` function available in `Path`.
pub fn concat_paths(path0: &Path, path1: &Path) -> PathBuf {
	let path1 = match path1.to_str() {
		Some(path1_str) if path1_str.starts_with('/') => Path::new(&path1_str[1..]),
		_ => path1,
	};

	path0.join(path1)
}
