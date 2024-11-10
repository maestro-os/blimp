//! TODO doc

use common::{
	anyhow::{anyhow, bail, Result},
	package, Environment,
};

// TODO ask for confirm before remove

/// Removes the given list of packages.
///
/// Arguments:
/// - `names` is the list of packages to remove.
/// - `env` is the blimp environment.
pub fn remove(names: &[String], env: &mut Environment) -> Result<()> {
	let installed = env.load_installed_list()?;

	// The list of remaining packages after the remove operation
	let remaining = {
		let mut installed = installed.clone();
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
				pkg.desc.name, pkg.desc.version, dep
			);
		}

		bail!("dependencies would break");
	}

	let mut failed = false;
	// Remove packages
	for name in names {
		if let Some(installed) = installed.get(name) {
			env.remove(installed)
				.map_err(|e| anyhow!("failed to remove package `{}`: {e}", installed.desc.name))?;
		} else {
			eprintln!("Package `{}` not found!", name);
			failed = true;
		}
	}
	if failed {
		bail!("cannot remove packages");
	}

	Ok(())
}
