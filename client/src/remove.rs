//! TODO doc

use common::Environment;
use std::error::Error;
use std::path::PathBuf;

/// Removes the given list of packages.
///
/// Arguments:
/// - `names` is the list of packages to remove.
/// - `env` is the blimp environment.
/// - `local_repos` is the list of paths to local package repositories.
pub fn remove(
	_names: &[String],
	_env: &mut Environment,
	_local_repos: &[PathBuf],
) -> Result<(), Box<dyn Error>> {
	// TODO
	todo!();
}
