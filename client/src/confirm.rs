//! Confirmation prompt.

use common::maestro_utils;

/// Asks for confirmation. If yes, true is returned. Else, false is returned.
pub fn prompt() -> bool {
	loop {
		let Some(input) = maestro_utils::prompt::prompt(Some("Confirm? [Y/n] "), false) else {
			// Input closed, abort
			break false;
		};
		let input = input.trim().to_lowercase();
		match input.as_str() {
			"" | "y" | "ye" | "yes" => break true,
			"n" | "no" => break false,
			// Retry
			_ => {}
		}
	}
}
