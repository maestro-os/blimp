//! This module implements utility functions.

/// Tells whether the given package name is correct.
pub fn is_correct_name(name: &str) -> bool {
	name.chars().all(| c | {
		c.is_ascii_alphanumeric() || c == '-' || c == '_'
	})
}
