//! Remotes management.

use common::{repository::remote::Remote, Environment};

/// Lists remotes.
pub async fn list(env: &Environment) -> std::io::Result<()> {
	let remotes = Remote::load_list(env)?;
	println!("Remotes list:");
	for remote in remotes {
		let host = &remote.host;
		match remote.fetch_metadata().await {
			Ok(metadata) => println!("- {host} (status: UP): {motd}", motd = metadata.motd),
			Err(_) => println!("- {host} (status: DOWN)"),
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
