//! The Blimp builder is a tool allowing to build a package.

use std::env;
use std::process::exit;

/// The software's current version.
const VERSION: &str = "0.1";

/// Prints command line usage.
fn print_usage(bin: &str) {
    eprintln!("blimp package builder version {}", VERSION);
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("\t{} <FROM> [TO]", bin);
    eprintln!();
    eprintln!("FROM is the path to the package's build files");
    eprintln!("TO is the path to the directory where the files will be written");
    eprintln!();
    eprintln!("The software builds the package according to the package's build files, then \
writes the package's description `package.json` and archive `package.tar.gz` into the given \
destination directory.");
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

    // If the argument count is incorrect, print usage
    if args.len() <= 1 || args.len() > 3 {
        print_usage(&bin);
        exit(1);
    }

    let from = args[1].clone();
    let to = {
		if args.len() < 3 {
			".".to_owned()
    	} else {
			args[2].clone()
    	}
    };

	// TODO
	todo!();
}
