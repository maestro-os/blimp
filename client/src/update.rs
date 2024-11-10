//! This module handles packages list updating.

use common::{
	anyhow::{anyhow, bail, Result},
	repository::remote::Remote,
	Environment,
};

/// Updates the packages list.
pub async fn update(env: &mut Environment) -> Result<()> {
	let remotes = Remote::load_list(env)
		.map_err(|error| anyhow!("Could not update packages list: {error}"))?;
	println!("Updating from remotes...");
	let mut futures = Vec::new();
	for r in &remotes {
		futures.push((&r.host, r.fetch_list()));
	}
	let mut failed = false;
	for (host, f) in futures {
		match f.await {
			Ok(packages) => {
				println!("Remote `{host}`: Found {} package(s).", packages.len());
				todo!()
			}
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
