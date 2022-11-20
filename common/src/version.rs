//! The version structure represents the version of a package.

use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::cmp::min;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::num::ParseIntError;

/// Structure representing a version.
#[derive(Clone, Eq)]
pub struct Version {
	/// Vector containing the version numbers.
	numbers: Vec<u32>,
}

impl Serialize for Version {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> Deserialize<'de> for Version {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s: String = Deserialize::deserialize(deserializer)?;
		s.as_str()
			.try_into()
			.map_err(D::Error::custom)
	}
}

impl TryFrom<&str> for Version {
	type Error = ParseIntError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		value.split(".")
			.map(|n| n.parse::<u32>())
			.collect::<Result<Vec<_>, _>>()
			.map(|numbers| Self {
				numbers,
			})
	}
}

impl Ord for Version {
	fn cmp(&self, other: &Self) -> Ordering {
		for i in 0..min(self.numbers.len(), other.numbers.len()) {
			let cmp = self.numbers[i].cmp(&other.numbers[i]);

			match cmp {
				Ordering::Equal => {}

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
