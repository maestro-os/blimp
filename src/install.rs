//! This module handles package installation.

use common::package::Package;
use common::repository::Repository;
use common::repository::remote::Remote;
use common::repository;
use common::util;
use common::version::Version;
use crate::confirm;
use std::collections::HashMap;
use std::error::Error;
use tokio::runtime::Runtime;

// TODO Clean
/// Installs the given list of packages.
/// `names` is the list of packages to install.
/// `sysroot` is the path to the root of the system on which the packages will be installed.
/// `local_repos` is the list of paths to local package repositories.
/// On success, the function returns `true`. On failure, it returns `false`.
pub fn install(names: &[String], sysroot: &str, local_repos: &[String])
	-> Result<(), Box<dyn Error>> {
    let mut failed = false;

	// The list of repositories
	let repos = Repository::load_all(sysroot, local_repos)?;
    // The list of packages to install with their respective repository
    let mut packages = HashMap::<String, (Package, &Repository)>::new();

    for p in names {
        match repository::get_latest_package(&repos, sysroot, &p)? {
            Some((repo, package)) => {
				packages.insert(p.to_owned(), (package, repo));
			},

            None => {
                eprintln!("Package `{}` not found!", p);
                failed = true;
            },
        }
    }
	if failed {
		return Ok(());
	}

    println!("Resolving dependencies...");

    // The list of all packages, dependencies included
    let mut total_packages = packages.clone();

    // Resolving dependencies
    for (_, (package, _)) in packages {
        let valid = package.resolve_dependencies(
			sysroot,
			&mut total_packages,
			| name: String, version: Version | {
				let r = repository::get_package(&repos, sysroot, &name, &version)
					.or_else(| e | {
						eprintln!("error: {}", e);
						Err(())
					})
					.ok()?;

				let (remote, package) = r?;
				Some((package, remote))
			}
		)?;

        if !valid {
            failed = true;
        }
    }
	if failed {
		return Ok(());
	}

    // Creating the async runtime
    let rt = Runtime::new().unwrap();
    let mut futures = Vec::new();

    println!("Packages to be installed:");

    for (name, (package, repo)) in &total_packages {
        if package.is_in_cache(sysroot) {
            println!("\t- {} ({}) - cached", name, package.get_version());
        } else {
            println!("\t- {} ({})", name, package.get_version());
        }

        if !repo.has_package(package) {
			if let Some(remote) = repo.get_remote() {
				futures.push(remote.get_size(package));
			}
        }
    }

    // The total download size in bytes
    let mut total_size = 0;
    for f in futures {
        total_size += rt.block_on(f)?;
    }

    print!("Download size: ");
    util::print_size(total_size);
    println!();

    if !confirm::prompt() {
        println!("Aborting.");
        return Ok(());
    }

    println!("Downloading packages...");
    let mut futures = Vec::new();

    for (name, (package, repo)) in &total_packages {
        if package.is_in_cache(sysroot) {
            println!("`{}` is in cache.", name);
			continue;
        }

		if let Some(remote) = repo.get_remote() {
			futures.push(Remote::fetch_archive(remote, package, repo));
		}
    }

    // TODO Add progress bar
    for f in futures {
        if !rt.block_on(f) {
            failed = true;
        }
    }

    if failed {
		return Ok(());
	}
    println!();

    println!("Installing packages...");

    // Installing all packages
    for (name, (package, _)) in total_packages {
        println!("Installing `{}`...", name);

        if let Err(e) = package.install(sysroot) {
            eprintln!("Failed to install `{}`: {}", name, e);
        }
    }

    println!();
	Ok(())
}
