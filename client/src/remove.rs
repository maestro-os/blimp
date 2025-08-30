//! TODO doc

use common::{anyhow::Result, Environment};
use common::anyhow::bail;

/// Removes the given list of packages.
///
/// Arguments:
/// - `names` is the list of packages to remove.
/// - `env` is the blimp environment.
pub fn remove(names: &[String], _env: &mut Environment) -> Result<()> {
	if names.is_empty() {
		bail!("must specify at least one package");
	}
	todo!()
}
