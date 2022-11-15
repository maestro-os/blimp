//! This module implements a confirmation prompt.

use std::io;
use std::io::BufRead;
use std::io::Write;

/// Asks for confirmation. If yes, true is returned. Else, false is returned.
pub fn prompt() -> bool {
	let mut first = true;
	let stdin = io::stdin();
	let mut response;

	loop {
		if first {
			print!("Confirm? [y/n] ");
		} else {
			print!("Please type `y` or `n`. ");
		}

		let _ = io::stdout().flush();

		// Waiting for an input line
		let input = stdin.lock().lines().next().unwrap().unwrap();
		response = input == "y";

		// If the input is correct, stop asking
		if input == "y" || input == "n" {
			break;
		}

		first = false;
	}

	println!();

	response
}
