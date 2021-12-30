//! Blimp is a simple package manager for Unix systems.

mod confirm;
mod install;
mod lockfile;
mod remote;

use install::install;
use remote::Remote;
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

            install(names, &sysroot).is_ok()
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
