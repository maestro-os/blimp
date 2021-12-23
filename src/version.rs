//! The version structure represents the version of a package.

use std::cmp::Ordering;
use std::cmp::min;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;

/// Structure representing a version.
#[derive(Clone, Eq)]
pub struct Version {
    /// Vector containing the version numbers.
    numbers: Vec<u32>,
}

// TODO Implement from_string and to_string

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in 0..min(self.numbers.len(), other.numbers.len()) {
            let cmp = self.numbers[i].cmp(&other.numbers[i]);

            match cmp {
                Ordering::Equal => {},

                _ => return cmp,
            }
        }

        Ordering::Equal
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Display for Version {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
	    for i in 0..self.numbers.len() {
            write!(f, "{}", self.numbers[i])?;

	        if i + 1 < self.numbers.len() {
                write!(f, ".")?;
	        }
	    }

	    Ok(())
    }
}
