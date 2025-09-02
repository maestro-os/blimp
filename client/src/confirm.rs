/*
 * Copyright 2025 Luc Lenôtre
 *
 * This file is part of Maestro.
 *
 * Maestro is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Maestro is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Maestro. If not, see <https://www.gnu.org/licenses/>.
 */

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
