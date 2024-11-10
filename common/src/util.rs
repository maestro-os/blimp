//! This module implements utility functions.

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
	fs,
	fs::{File, OpenOptions},
	io,
	io::{BufReader, BufWriter, Read},
	os::unix,
	path::{Path, PathBuf},
};
use tar::Archive;
use xz2::read::XzDecoder;

fn create_tmp<T, F: Fn(&Path) -> io::Result<T>>(parent: &Path, f: F) -> io::Result<(PathBuf, T)> {
	fs::create_dir_all(parent)?;
	let parent = parent.canonicalize()?;
	for _ in 0..100 {
		let name: String = thread_rng()
			.sample_iter(&Alphanumeric)
			.take(16)
			.map(char::from)
			.collect();
		let path = parent.join(name);
		match f(&path) {
			Ok(res) => return Ok((path, res)),
			Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
			Err(e) => return Err(e),
		}
	}
	Err(io::Error::new(io::ErrorKind::Other, "too many tries"))
}

/// Creates a temporary directory. The function returns the path to the directory.
///
/// `parent` is the path to the parent of the temporary file.
pub fn create_tmp_dir<P: AsRef<Path>>(parent: P) -> io::Result<PathBuf> {
	Ok(create_tmp(parent.as_ref(), |p| fs::create_dir(p))?.0)
}

/// Creates a temporary file. The function returns the path to the file and the file itself.
///
/// `parent` is the path to the parent of the temporary file.
pub fn create_tmp_file<P: AsRef<Path>>(parent: P) -> io::Result<(PathBuf, File)> {
	create_tmp(parent.as_ref(), |path| {
		OpenOptions::new()
			.read(true)
			.write(true)
			.create_new(true)
			.open(path)
	})
}

fn decompress_impl<R: Read>(stream: R, dest: &Path) -> io::Result<()> {
	let mut archive = Archive::new(stream);
	archive.set_overwrite(true);
	archive.set_preserve_permissions(true);
	archive.unpack(dest)?;
	Ok(())
}

/// Decompresses the given archive file `src` to the given location `dest`.
pub fn decompress(src: &Path, dest: &Path) -> io::Result<()> {
	let file_type = infer::get_from_path(src)?.map(|t| t.mime_type());
	let file = File::open(src)?;
	match file_type {
		Some("application/gzip") => decompress_impl(GzDecoder::new(file), dest),
		Some("application/x-xz") => decompress_impl(XzDecoder::new(file), dest),
		Some("application/x-bzip2") => decompress_impl(BzDecoder::new(file), dest),
		_ => Err(io::Error::new(
			io::ErrorKind::Other,
			format!(
				"Invalid or unsupported archive format: {}",
				file_type.unwrap_or("<not detected>")
			),
		)),
	}
}

/// Reads the package archive at the given path and returns an instance for it.
pub fn read_package_archive(path: &Path) -> io::Result<Archive<GzDecoder<File>>> {
	let mut archive = Archive::new(GzDecoder::new(File::open(path)?));
	archive.set_overwrite(true);
	archive.set_preserve_permissions(true);
	Ok(archive)
}

/// Copies the content of the directory `src` to the directory `dst` recursively.
pub fn recursive_copy(src: &Path, dst: &Path) -> io::Result<()> {
	for entry in fs::read_dir(src)? {
		let entry = entry?;
		let file_type = entry.file_type()?;
		let to = dst.join(entry.file_name());

		if file_type.is_dir() {
			// TODO Set timestamps, permissions and owner
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

// TODO: rework to allow deserialize from structs with lifetimes (currently unefficient)
/// Reads a JSON file.
pub fn read_json<T: for<'a> Deserialize<'a>>(file: &Path) -> io::Result<T> {
	let file = File::open(file)?;
	let reader = BufReader::new(file);
	Ok(serde_json::from_reader(reader)?)
}

/// Writes a JSON file.
pub fn write_json<T: Serialize>(file: &Path, data: &T) -> io::Result<()> {
	let file = File::create(file)?;
	let writer = BufWriter::new(file);
	Ok(serde_json::to_writer_pretty(writer, &data)?)
}

/// Concatenates the given paths.
///
/// This function is different from the [`Path::join`] in that it does not suppress the
/// first path if the second is absolute.
pub fn concat_paths<P0: AsRef<Path>, P1: AsRef<Path>>(base: P0, other: P1) -> PathBuf {
	let other = other.as_ref();
	let other = other.strip_prefix("/").unwrap_or(other);
	base.as_ref().join(other)
}
