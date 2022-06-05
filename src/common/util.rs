//! This module implements utility functions.

use flate2::read::GzDecoder;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io;
use std::path::Component::Normal;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tar::Archive;
use xz2::read::XzDecoder;

// TODO Add a maximum try count
/// Creates a temporary directory. The function returns the path to the directory.
pub fn create_tmp_dir() -> io::Result<String> {
    let mut i = 0;

    loop {
        let path = format!("/tmp/blimp-{}", i);
        if fs::create_dir(&path).is_ok() {
            return Ok(path);
        }

        i += 1;
    }
}

// TODO Add a maximum try count
/// Creates a temporary file. The function returns the path to the file and the file itself.
pub fn create_tmp_file() -> io::Result<(String, File)> {
    let mut i = 0;

    loop {
        let path = format!("/tmp/blimp-{}", i);
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
/// `unwrap` tells whether the tarball shall be unwrapped.
fn uncompress_<R: Read, D: AsRef<Path>>(mut archive: Archive<R>, dest: D, unwrap: bool)
	-> Result<(), Box<dyn Error>> {
	if unwrap {
		for entry in archive.entries()? {
			let mut entry = entry?;

			let path: PathBuf = entry.path()?
				.components()
				.skip(1)
				.filter(| c | matches!(c, Normal(_)))
				.collect();

			entry.unpack(dest.as_ref().join(path))?;
		}
	} else {
		archive.unpack(dest)?;
	}

	Ok(())
}

/// Uncompresses the given archive file `src` to the given location `dest`.
/// `unwrap` tells whether the tarball shall be unwrapped.
/// If the tarball contains directories at the root, the unwrap operation unwraps their content
/// instead of the directories themselves.
pub fn uncompress<S: AsRef<Path>, D: AsRef<Path>>(src: S, dest: D, unwrap: bool)
	-> Result<(), Box<dyn Error>> {
	// Trying to uncompress .tar.gz
	{
		let file = File::open(&src)?;
		let tar = GzDecoder::new(file);
		let archive = Archive::new(tar);

		if uncompress_(archive, &dest, unwrap).is_ok() {
			return Ok(());
		}
	}

	// Trying to uncompress .tar.xz
	{
		let file = File::open(&src)?;
		let tar = XzDecoder::new(file);
		let archive = Archive::new(tar);

		uncompress_(archive, &dest, unwrap)
	}
}

/// Uncompresses the given .tar.gz file `archive` into a temporary directory, executes the given
/// function `f` with the path to the temporary directory as argument, then removes the directory
/// and returns the result of the call to `f`.
pub fn uncompress_wrap<T, F: FnOnce(&str) -> T>(archive: &str, f: F) -> Result<T, Box<dyn Error>> {
    // Uncompressing
    let tmp_dir = create_tmp_dir()?;
    uncompress(archive, &tmp_dir, false)?;

    let v = f(&tmp_dir);

    // Removing temporary directory
    fs::remove_dir_all(&tmp_dir)?;

    Ok(v)
}

/// Run the hook at the given path.
/// `sysroot` is the sysroot.
/// If the hook succeeded, the function returns `true`. If it didn't, it returns `false`.
/// If the hook doesn't exist, the function does nothing and returns successfully.
pub fn run_hook(hook_path: &str, sysroot: &str) -> io::Result<bool> {
    if !Path::new(hook_path).exists() {
        return Ok(true);
    }

    // Runs the hook
    let status = Command::new(hook_path)
        .env("SYSROOT", sysroot)
        .status()?;

    if let Some(code) = status.code() {
        Ok(code == 0)
    } else {
        Ok(false)
    }
}

/// Copies the content of the directory `src` to the directory `dst` recursively.
pub fn recursive_copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
	for entry in fs::read_dir(src)? {
        let entry = entry?;

        let to = dst.as_ref().join(entry.file_name());

        if entry.file_type()?.is_dir() {
        	fs::create_dir_all(&to)?;
            recursive_copy(entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), &to)?;
        }
    }

	Ok(())
}

/// Prints the given size in bytes into a human-readable form.
pub fn print_size(mut size: u64) {
    let mut level = 0;
    while level < 6 && size > 1024 {
        size /= 1024;
        level += 1;
    }

    let suffix = match level {
        0 => " bytes",
        1 => " KiB",
        2 => " MiB",
        3 => " GiB",
        4 => " TiB",
        5 => " PiB",
        6 => " EiB",

        _ => return,
    };

    print!("{}{}", size, suffix);
}

/// Reads a JSON file.
pub fn read_json<T: for<'a> Deserialize<'a>>(file: &str) -> io::Result<T> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).or_else(|e| {
    	let msg = format!("JSON deserializing failed: {}", e);
		Err(io::Error::new(io::ErrorKind::Other, msg))
    })
}

/// Writes a JSON file.
pub fn write_json<T: Serialize>(file: &str, data: &T) -> io::Result<()> {
    let file = File::create(file)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &data).or_else(|e| {
    	let msg = format!("JSON serializing failed: {}", e);
		Err(io::Error::new(io::ErrorKind::Other, msg))
    })
}
