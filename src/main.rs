mod remote;

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
}

fn main() {
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
        exit(1);
    }

    // Matching command
    match args[1].as_str() {
        "info" => {
            let packages = &args[2..];
            if packages.len() == 0 {
                eprintln!("Please specify one or several packages");
                exit(1);
            }

            // TODO
        },

        "install" => {
            let packages = &args[2..];
            if packages.len() == 0 {
                eprintln!("Please specify one or several packages");
                exit(1);
            }

            // TODO
        },

        "update" => {
            // TODO
        },

        "upgrade" => {
            let packages = &args[2..];

            // TODO
        },

        "remove" => {
            let packages = &args[2..];
            if packages.len() == 0 {
                eprintln!("Please specify one or several packages");
                exit(1);
            }

            // TODO
        },

        "remote-list" => {
            let remotes = Remote::list();

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
        },

        "remote-remove" => {
            // TODO
        },

        _ => {
            eprintln!("Command `{}` doesn't exist", args[1]);
            eprintln!();
            print_usage(&bin);
            exit(1);
        },
    }
}
