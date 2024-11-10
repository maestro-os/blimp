//! Remotes management.

use common::{repository::remote::Remote, Environment};

/// Lists remotes.
pub fn list(env: &Environment) -> std::io::Result<()> {
	let remotes = Remote::load_list(env)?;
	println!("Remotes list:");
	for remote in remotes {
		let host = &remote.host;
		match remote.fetch_motd() {
			Ok(motd) => println!("- {host} (status: UP): {motd}"),
			Err(_) => println!("- {host} (status: DOWN)"),
		}
	}
	Ok(())
}

/// Adds one or several remotes.
///
/// Arguments:
/// - `env` is the environment.
/// - `remotes` is the list of remotes to add.
pub fn add(env: &mut Environment, new_remotes: &[String]) -> std::io::Result<()> {
	let mut remotes = Remote::load_list(env)?;
	for remote in new_remotes {
		if remotes.contains(remote.as_str()) {
			eprintln!("Remote `{remote}` already exists");
		} else {
			println!("Add remote `{remote}`");
			remotes.insert(Remote {
				host: remote.clone(),
			});
		}
	}
	Remote::save_list(env, remotes.into_iter())?;
	Ok(())
}

/// Removes one or several remotes.
///
/// Arguments:
/// - `env` is the environment.
/// - `remotes` is the list of remotes to remove.
pub fn remove(env: &mut Environment, new_remotes: &[String]) -> std::io::Result<()> {
	let mut remotes = Remote::load_list(env)?;
	for remote in new_remotes {
		let existed = remotes.remove(remote.as_str());
		if !existed {
			eprintln!("Remote `{remote}` not found");
		}
	}
	Remote::save_list(env, remotes.into_iter())?;
	Ok(())
}
