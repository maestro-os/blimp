//! This module handles packages list updating.

use common::Environment;
use common::repository::remote::Remote;
use std::error::Error;
use tokio::runtime::Runtime;

/// Updates the packages list.
pub fn update(env: &mut Environment) -> Result<(), Box<dyn Error>> {
	let remotes = Remote::load_list(env)
		.map_err(|e| format!("Could not update packages list: {}", e))?;

	println!("Updating from remotes...");

	// Creating the async runtime
	let rt = Runtime::new().unwrap();
	let mut futures = Vec::new();

	let mut err = false;

	for r in remotes.iter() {
		let host = r.get_host();
		futures.push((host, r.fetch_list()));
	}
	for (host, f) in futures {
		match rt.block_on(f) {
			Ok(packages) => {
				println!("Remote `{}`: Found {} package(s).", host, packages.len());

				// TODO
				todo!();
			}

			Err(e) => {
				eprintln!("Remote `{}`: {}", host, e);
				err = true;
			}
		}
	}

	if err {
		Err("update failed".into())
	} else {
		Ok(())
	}
}
