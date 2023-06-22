//! This module handles packages list updating.

use common::repository::remote::Remote;
use common::Environment;
use std::error::Error;

/// Updates the packages list.
pub async fn update(env: &mut Environment) -> Result<(), Box<dyn Error>> {
	let remotes =
		Remote::load_list(env).map_err(|e| format!("Could not update packages list: {}", e))?;

	println!("Updating from remotes...");

	let mut futures = Vec::new();
	for r in remotes {
		let host = r.get_host().to_owned();
        // TODO limit the number of concurrent tasks running
		futures.push((host, tokio::spawn(async move {
            r.fetch_list().await
        })));
	}

	let mut err = false;
	for (host, f) in futures {
		match f.await? {
			Ok(packages) => {
				println!("Remote `{host}`: Found {} package(s).", packages.len());

				// TODO
				todo!();
			}

			Err(e) => {
				eprintln!("Remote `{host}`: {e}");
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
