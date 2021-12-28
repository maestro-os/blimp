//! TODO doc

#![feature(async_closure)]

mod confirm;
mod lockfile;
mod remote;

use common::package::Package;
use common::version::Version;
use remote::Remote;
use std::collections::HashMap;
use std::env;
use std::process::exit;
use tokio::runtime::Runtime;

/// The software's current version.
const VERSION: &str = "0.1";

/// Prints command line usage.
fn print_usage(bin: &String) {
    eprintln!("blimp package manager version {}", VERSION);
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("\t{} <COMMAND> [OPTIONS]", bin);
    eprintln!();
    eprintln!("COMMAND:");
    eprintln!("\tinfo <package...>: Prints informations about the given package(s)");
    eprintln!("\tinstall <package...>: Installs the given package(s)");
    eprintln!("\tupdate: Synchronizes packets informations from remote");
    eprintln!("\tupgrade [package...]: Upgrades the given package(s). If no package is specified, \
the package manager updates every packages that are not up to date");
    eprintln!("\tremove <package...>: Removes the given package(s)");
    eprintln!("\tclean: Clean the cache");
    eprintln!("\tremote-list: Lists remote servers");
    eprintln!("\tremote-add <remote>: Adds a remote server");
    eprintln!("\tremote-remove <remote>: Removes a remote server");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("\t--verbose: Enables verbose mode");
    eprintln!("\t--version <version>: When installing or upgrading, this option allows to\
specify a version");
    eprintln!();
    eprintln!("ENVIRONMENT VARIABLES:");
    eprintln!("\tSYSROOT: Specifies the path to the system's root");
    eprintln!("\tLOCAL_REPOSITORY: Specifies pathes separated by `:` at which packages are stored \
locally");
}

/// Downloads the package `package` from remote `remote`.
async fn download_package(remote: &Remote, package: &Package) -> bool {
    match remote.download(package).await {
        Ok(_) => {
            println!("Downloaded `{}`", package.get_name());
            true
        },

        Err(e) => {
            eprintln!("Failed to download `{}`: {}", package.get_name(), e);
            false
        },
    }
}

// TODO Clean
/// Installs the given list of packages.
/// On success, the function returns `true`. On failure, it returns `false`.
fn install(names: &[String]) -> bool {
    // Changed to `false` if a problem is found
    let mut valid = true;

    // The list of packages to install
    let mut packages = HashMap::<String, Package>::new();
    // The list of remotes for each packages
    let mut remotes = HashMap::<String, Remote>::new();

    for p in names {
        match Remote::get_latest(&p.clone()).unwrap() { // TODO Handle error
            Some((remote, package)) => {
                let name = package.get_name().clone();
                packages.insert(name.clone(), package);
                remotes.insert(name, remote);
            },

            None => {
                eprintln!("Package `{}` not found!", p);
                valid = false;
            },
        }
    }

    if !valid {
        return false;
    }

    println!("Resolving dependencies...");

    // The list of all packages, dependencies included
    let mut total_packages = packages.clone();

    // Resolving dependencies
    for (_, package) in &packages {
        // Closure to get a package
        let mut get_package = | name: String, version: Version | {
            if let Ok(r) = Remote::get_package(&name, &version) {
                let (remote, package) = r?;
                remotes.insert(remote.get_host().to_string(), remote);

                Some(package)
            } else {
                // TODO Print error
                None
            }
        };

        if !package.resolve_dependencies(&mut total_packages, &mut get_package) {
            valid = false;
        }
    }

    if !valid {
        return false;
    }

    // Creating the async runtime
    let rt = Runtime::new().unwrap();
    let mut futures = Vec::new();

    println!("Packages to be installed:");
    for (name, package) in &total_packages {
        println!("\t- {} ({})", name, package.get_version());

        if !package.is_in_cache() {
            let remote = &remotes[package.get_name()];
            futures.push(remote.get_size(package));
        }
    }

    // The total download size in bytes
    let mut total_size = 0;
    for f in futures {
        match rt.block_on(f) {
            Ok(size) => total_size += size,
            Err(e) => {
                eprintln!("Failed to retrieve package size: {}", e);
                valid = false;
            },
        }
    }

    if !valid {
        return false;
    }
    println!("Download size: {} bytes", total_size); // TODO Format to be human readable

    if !confirm::prompt() {
        println!("Aborting.");
        return false;
    }

    println!("Downloading packages...");
    let mut futures = Vec::new();

    for (name, package) in &total_packages {
        if !package.is_in_cache() {
            let remote = &remotes[package.get_name()];
            futures.push(download_package(remote, package));
        } else {
            println!("`{}` is in cache.", name);
        }
    }

    // TODO Add progress bar
    for f in futures {
        if !rt.block_on(f) {
            valid = false;
        }
    }

    if !valid {
        return false;
    }

    // TODO Install packages

    valid
}

// TODO Parse options
fn main_() -> bool {
    let args: Vec<String> = env::args().collect();
    // The name of the binary file
    let bin = {
        if args.len() == 0 {
            String::from("blimp")
        } else {
            args[0].clone()
        }
    };

    // If no argument is specified, print usage
    if args.len() <= 1 {
        print_usage(&bin);
        return false;
    }

    // Matching command
    match args[1].as_str() {
        "info" => {
            let packages = &args[2..];
            if packages.len() == 0 {
                eprintln!("Please specify one or several packages");
                return false;
            }

            // TODO
            todo!();
        },

        "install" => {
            let names = &args[2..];
            if names.len() == 0 {
                eprintln!("Please specify one or several packages");
                return false;
            }

            if !install(names) {
                return false;
            }
        },

        "update" => {
            if let Ok(remotes) = Remote::list() {
                println!("Updating from remotes...");

                // TODO async?
                for r in remotes {
                    let host = r.get_host();

                    println!("Updating from {}...", host);

                    match r.fetch_all(true) {
                        Ok(packages) => println!("Found {} package(s).", packages.len()),
                        Err(e) => eprintln!("{}", e),
                    }
                }
            } else {
                eprintln!("IO error");
                return false;
            }
        },

        "upgrade" => {
            let _packages = &args[2..];

            // TODO
            todo!();
        },

        "remove" => {
            let packages = &args[2..];
            if packages.len() == 0 {
                eprintln!("Please specify one or several packages");
                return false;
            }

            // TODO
            todo!();
        },

        "clean" => {
            // TODO
            todo!();
        },

        "remote-list" => {
            if let Ok(remotes) = Remote::list() {
                println!("Remotes list:");

                for r in remotes {
                    let host = r.get_host();

                    match r.get_motd() {
                        Ok(m) => println!("- {} (status: UP): {}", host, m),
                        Err(_) => println!("- {} (status: DOWN)", host),
                    }
                }
            } else {
                eprintln!("IO error");
                return false;
            }
        },

        "remote-add" => {
            // TODO
            todo!();
        },

        "remote-remove" => {
            // TODO
            todo!();
        },

        _ => {
            eprintln!("Command `{}` doesn't exist", args[1]);
            eprintln!();
            print_usage(&bin);
            return false;
        },
    }

    true
}

fn main() {
    // Creating a lock file if possible
    if !lockfile::lock() {
        eprintln!("Error: failed to acquire lockfile");
        exit(1);
    }

    let success = main_();

    lockfile::unlock();

    if !success {
        exit(1);
    }
}
