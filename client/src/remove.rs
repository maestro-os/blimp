//! TODO doc

use common::package;
use common::Environment;
use std::error::Error;

// TODO ask for confirm before remove

/// Removes the given list of packages.
///
/// Arguments:
/// - `names` is the list of packages to remove.
/// - `env` is the blimp environment.
pub fn remove(names: &[String], env: &mut Environment) -> Result<(), Box<dyn Error>> {
	// The list of remaining packages after the remove operation
	let remaining = {
		let mut installed = env.get_installed_list()?;
		installed.retain(|name, _| !names.contains(name));

		installed
	};

	// Check for dependency breakages
	let unmatched_deps = package::list_unmatched_dependencies(&remaining);
	if !unmatched_deps.is_empty() {
		eprintln!("The following dependencies would be broken:");

		for (pkg, dep) in unmatched_deps {
			eprintln!(
				"- for package `{}` (version `{}`): {}",
				pkg.desc.get_name(),
				pkg.desc.get_version(),
				dep
			);
		}

		return Err("dependencies would break".into());
	}

	let mut failed = false;

	let installed = env.get_installed_list()?;

	// Remove packages
	for name in names {
		if let Some(installed) = installed.get(name) {
			env.remove(installed).map_err(|e| {
				format!(
					"failed to remove package `{}`: {}",
					installed.desc.get_name(),
					e
				)
			})?;
		} else {
			eprintln!("Package `{}` not found!", name);
			failed = true;
		}
	}
	if failed {
		return Err("cannot remove packages".into());
	}

	Ok(())
}
