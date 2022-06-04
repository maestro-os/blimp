//! This module implements utility functions.

/// Tells whether the given package name is correct.
pub fn is_correct_name(name: &str) -> bool {
	name.chars().all(| c | {
		c.is_ascii_alphanumeric() || c == '-' || c == '_'
	})
}

/// Tells whether the given string is a correct job ID.
pub fn is_correct_job_id(id: &str) -> bool {
	id.chars().all(| c | {
		c.is_ascii_alphanumeric()
	})
}
