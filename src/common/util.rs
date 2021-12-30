//! This module implements utility functions.

use flate2::read::GzDecoder;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::fs;
use std::io::BufReader;
use std::io::BufWriter;
use std::io;
use std::path::Path;
use std::process::Command;
use tar::Archive;

/// Creates a temporary directory. The function returns the path to the directory.
pub fn create_tmp_dir() -> io::Result<String> {
    let mut i = 0;

    loop {
        let s = format!("/tmp/blimp-{}", i);

        let path = Path::new(&s);
        if !path.exists() {
            fs::create_dir(path)?;
            return Ok(s);
        }

        i += 1;
    }
}

/// Uncompresses the given .tar.gz file `src` to the given location `dest`.
pub fn uncompress(src: &str, dest: &str) -> io::Result<()> {
    let tar_gz = File::open(src)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(dest)
}

/// Uncompresses the given .tar.gz file `archive` into a temporary directory, executes the given
/// function `f` with the path to the temporary directory as argument, then removes the directory
/// and returns the result of the call to `f`.
pub fn uncompress_wrap<T, F: FnOnce(&str) -> T>(archive: &str, f: F) -> io::Result<T> {
    // Uncompressing
    let tmp_dir = create_tmp_dir()?;
    uncompress(archive, &tmp_dir)?;

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

	let err = Err(io::Error::new(io::ErrorKind::Other, "JSON deserializing failed"));
    serde_json::from_reader(reader).or(err)
}

/// Writes a JSON file.
pub fn write_json<T: Serialize>(file: &str, data: &T) -> io::Result<()> {
    let file = File::create(file)?;
    let writer = BufWriter::new(file);

	let err = Err(io::Error::new(io::ErrorKind::Other, "JSON serializing failed"));
    serde_json::to_writer_pretty(writer, &data).or(err)
}
