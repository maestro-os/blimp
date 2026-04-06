/*
 * Copyright 2025 Luc Lenôtre
 *
 * This file is part of Maestro.
 *
 * Maestro is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Maestro is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Maestro. If not, see <https://www.gnu.org/licenses/>.
 */

//! TODO doc

use common::{
	anyhow::{bail, Result},
	Environment,
};

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
