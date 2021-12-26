mod confirm;
mod lockfile;
mod remote;

use common::package::Package;
use remote::Remote;
use std::collections::HashMap;
use std::env;
use std::process::exit;

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

/// Installs the given list of packages.
/// On success, the function returns `true`. On failure, it returns `false`.
fn install(names: &[String]) -> bool {
    // Changed to `false` if a problem is found
    let mut valid = true;

    // The list of packages to install
    let mut packages = HashMap::<String, Package>::new();

    for p in names {
        match Package::get_latest(&p.clone()) {
            Some(package) => {
                packages.insert(package.get_name().clone(), package);
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
        if !package.resolve_dependencies(&mut total_packages) {
            valid = false;
        }
    }

    if !valid {
        return false;
    }

    println!("Packages to be installed:");
    // The total download size in bytes
    let mut total_size = 0;
    for (name, package) in &total_packages {
        println!("\t- {} ({})", name, package.get_version());

        if package.is_in_cache() {
            // TODO Run in async: total_size += package.get_size();
        }
    }
    println!("Download size: {} bytes", total_size); // TODO Format to be human readable

    if !confirm::prompt() {
        println!("Aborting.");
        return false;
    }

    println!("Downloading packages...");
    // TODO Add progress bar
    // TODO Download in async
    for (name, package) in &total_packages {
        if !package.is_in_cache() {
            // TODO Run in async: package.download();
        } else {
            println!("`{}` is in cache.", name);
        }
    }

    true
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
            // TODO
            todo!();
        },

        "upgrade" => {
            let packages = &args[2..];

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
            let remotes = Remote::list();
            if remotes.is_err() {
                eprintln!("IO error :(");
                return false;
            }
            let remotes = remotes.unwrap();

            println!("Remotes list:");
            for r in remotes {
                let host = r.get_host();

                match r.get_motd() {
                    Ok(m) => println!("- {} (status: UP): {}", host, m),
                    Err(_) => println!("- {} (status: DOWN)", host),
                }
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
