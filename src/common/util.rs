//! This module implements utility functions.

use flate2::read::GzDecoder;
use std::fs::File;
use std::fs;
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

/// TODO doc
pub fn uncompress_wrap<T, F: FnOnce(&str) -> T>(archive: &str, f: F) -> io::Result<T> {
    // Uncompressing
    let tmp_dir = create_tmp_dir()?;
    uncompress(archive, &tmp_dir)?;

    let v = f(&tmp_dir);

    // Removing temporary directory
    // TODO rm

    Ok(v)
}

/// Run the hook at the given path.
/// `sysroot` is the sysroot.
/// If the hook succeeded, the function returns `true`. If it didn't, it returns `false`.
pub fn run_hook(hook_path: &str, sysroot: &str) -> io::Result<bool> {
    let status = Command::new(hook_path)
        .env("SYSROOT", sysroot)
        .status()?;

    if let Some(code) = status.code() {
        Ok(code == 0)
    } else {
        Ok(false)
    }
}
