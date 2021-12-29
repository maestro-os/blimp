//! Blimp is a simple package manager for Unix systems.

#![feature(async_closure)]

mod confirm;
mod lockfile;
mod remote;

use common::package::Package;
use common::util;
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
/// `sysroot` is the path to the system's root.
async fn download_package(sysroot: &str, remote: &Remote, package: &Package) -> bool {
    match remote.download(sysroot, package).await {
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
/// `sysroot` is the path to the root of the system on which the packages will be installed.
/// On success, the function returns `true`. On failure, it returns `false`.
fn install(names: &[String], sysroot: &str) -> bool {
    // Changed to `false` if a problem is found
    let mut valid = true;

    // The list of packages to install
    let mut packages = HashMap::<String, Package>::new();
    // The list of remotes for each packages
    let mut remotes = HashMap::<String, Remote>::new();

    for p in names {
        let r = Remote::get_latest(sysroot, &p.clone());
        if let Err(e) = r {
            println!("IO error: {}", e);
            return false;
        }
        let r = r.unwrap();

        match r {
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
            if let Ok(r) = Remote::get_package(sysroot, &name, &version) {
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
        if package.is_in_cache(sysroot) {
            println!("\t- {} ({}) - cached", name, package.get_version());
        } else {
            println!("\t- {} ({})", name, package.get_version());
        }

        if !package.is_in_cache(sysroot) {
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
    print!("Download size: ");
    util::print_size(total_size);
    println!();

    if !confirm::prompt() {
        println!("Aborting.");
        return false;
    }

    println!("Downloading packages...");
    let mut futures = Vec::new();

    for (name, package) in &total_packages {
        if !package.is_in_cache(sysroot) {
            let remote = &remotes[package.get_name()];
            futures.push(download_package(sysroot, remote, package));
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
    println!();

    println!("Installing packages...");

    // Installing all packages
    for (name, package) in total_packages {
        println!("Installing `{}`...", name);

        if let Err(e) = package.install(sysroot) {
            eprintln!("Failed to install `{}`: {}", name, e);
            valid = false;
        }
    }

    println!();

    valid
}

/// Updates the packages list.
/// `sysroot` is the path to the root of the system.
fn update(sysroot: &str) -> bool {
    match Remote::list(sysroot) {
        Ok(remotes) => {
            println!("Updating from remotes...");

            // TODO async?
            for r in remotes {
                let host = r.get_host();

                println!("Updating from {}...", host);

                match r.fetch_all(true, &sysroot) {
                    Ok(packages) => println!("Found {} package(s).", packages.len()),
                    Err(e) => eprintln!("{}", e),
                }
            }

            true
        },

        Err(e) => {
            eprintln!("IO error: {}", e);
            false
        },
    }
}

/// Lists remotes.
/// `sysroot` is the path to the root of the system.
fn remote_list(sysroot: &str) -> bool {
    match Remote::list(sysroot) {
        Ok(remotes) => {
            println!("Remotes list:");

            for r in remotes {
                let host = r.get_host();

                match r.get_motd() {
                    Ok(m) => println!("- {} (status: UP): {}", host, m),
                    Err(_) => println!("- {} (status: DOWN)", host),
                }
            }

            true
        },
        Err(e) => {
            eprintln!("IO error: {}", e);
            false
        },
    }
}

// TODO Parse options
fn main_(sysroot: &str) -> bool {
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

            install(names, &sysroot)
        },

        "update" => update(sysroot),

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

        "remote-list" => remote_list(sysroot),

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

            false
        },
    }
}

fn main() {
    // Getting the sysroot
    let sysroot = env::var("SYSROOT").unwrap_or("/".to_string());

    // Creating a lock file if possible
    if !lockfile::lock(&sysroot) {
        eprintln!("Error: failed to acquire lockfile");
        exit(1);
    }

    let success = main_(&sysroot);

    lockfile::unlock(&sysroot);

    if !success {
        exit(1);
    }
}
