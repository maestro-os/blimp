//! This module handles packages list updating.

use common::repository::remote::Remote;
use std::path::Path;
use tokio::runtime::Runtime;

/// Updates the packages list.
///
/// `sysroot` is the path to the root of the system.
pub fn update(sysroot: &Path) -> bool {
	let remotes = match Remote::load_list(sysroot) {
		Ok(remotes) => remotes,

		Err(e) => {
			eprintln!("Could not update packages list: {}", e);
			return false;
		}
	};

	println!("Updating from remotes...");

	// Creating the async runtime
	let rt = Runtime::new().unwrap();
	let mut futures = Vec::new();

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
			},

			Err(e) => eprintln!("Remote `{}`: {}", host, e), // TODO return false
		}
	}

	true
}
