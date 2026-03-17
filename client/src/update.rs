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

//! This module handles packages list updating.

use common::{
	anyhow::{anyhow, bail, Result},
	repository::remote::Remote,
	Environment,
};

/// Updates the packages list.
pub async fn update(env: &mut Environment) -> Result<()> {
	let remotes = Remote::load_list(env)
		.map_err(|error| anyhow!("could not update packages list: {error}"))?;
	println!("Update from remotes...");
	let mut futures = Vec::new();
	for r in &remotes {
		futures.push((&r.host, r.fetch_index(env)));
	}
	let mut failed = false;
	for (host, f) in futures {
		match f.await {
			Ok(cnt) => println!("Remote `{host}`: Found {cnt} package(s)."),
			Err(e) => {
				eprintln!("Remote `{host}`: {e}");
				failed = true;
			}
		}
	}
	if failed {
		bail!("update failed");
	}
	Ok(())
}
