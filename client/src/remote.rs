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

//! Remotes management.

use common::{repository::remote::Remote, Environment};

/// Lists remotes.
pub async fn list(env: &Environment) -> std::io::Result<()> {
	let remotes = Remote::load_list(env)?;
	println!("Remotes list:");
	for remote in remotes {
		let host = &remote.host;
		match remote.fetch_motd().await {
			Ok(Some(motd)) => println!("- {host} (status: UP): {motd}"),
			Ok(None) => println!("- {host} (status: UP)"),
			Err(err) => println!("- {host} (status: DOWN): {err}"),
		}
	}
	Ok(())
}

/// Adds a remote.
///
/// Arguments:
/// - `env` is the environment
/// - `remote` is the remote to add
pub fn add(env: &mut Environment, remote: String) -> std::io::Result<()> {
	let mut remotes = Remote::load_list(env)?;
	if remotes.contains(remote.as_str()) {
		eprintln!("Remote `{remote}` already exists");
	} else {
		println!("Add remote `{remote}`");
		remotes.insert(Remote {
			host: remote,
		});
	}
	Remote::save_list(env, remotes.into_iter())?;
	Ok(())
}

/// Removes a remote.
///
/// Arguments:
/// - `env` is the environment
/// - `remote` is the remote to remove
pub fn remove(env: &mut Environment, remote: String) -> std::io::Result<()> {
	let mut remotes = Remote::load_list(env)?;
	let existed = remotes.remove(remote.as_str());
	if !existed {
		eprintln!("Remote `{remote}` not found");
	}
	Remote::save_list(env, remotes.into_iter())?;
	Ok(())
}
