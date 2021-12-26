//! The version structure represents the version of a package.

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use std::cmp::Ordering;
use std::cmp::min;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;
use std::num::ParseIntError;

/// Structure representing a version.
#[derive(Clone, Eq)]
pub struct Version {
    /// Vector containing the version numbers.
    numbers: Vec<u32>,
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_string(&s).map_err(D::Error::custom)
    }
}

impl Version {
    /// Creates an instance from a given string.
    pub fn from_string(string: &str) -> Result<Self, ParseIntError> {
        let mut s = Self {
            numbers: Vec::new(),
        };

        for n in string.split(".") {
            s.numbers.push(n.parse::<u32>()?);
        }

        Ok(s)
    }

    /// Returns the version as string.
    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for (i, n) in self.numbers.iter().enumerate() {
            s += &n.to_string();

            if i + 1 < self.numbers.len() {
                s += ".";
            }
        }

        s
    }
}

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
