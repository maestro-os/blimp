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
#[derive(Clone, Eq, Hash, PartialEq)]
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
		s.as_str().try_into().map_err(D::Error::custom)
	}
}

impl TryFrom<&str> for Version {
	type Error = ParseIntError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		value
			.split('.')
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

/// Enumeration of contraints on a package's dependencies.
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum VersionConstraint {
	/// Any version match.
	Any,
	/// The version must be equal to the given version.
	Equal(Version),
	/// The version must be less than or equal to the given version.
	LessOrEqual(Version),
	/// The version must be less than the given version.
	Less(Version),
	/// The version must be greater than or equal to the given version.
	GreaterOrEqual(Version),
	/// The version must be greater than the given version.
	Greater(Version),
}

impl Serialize for VersionConstraint {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> Deserialize<'de> for VersionConstraint {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s: String = Deserialize::deserialize(deserializer)?;
		s.as_str().try_into().map_err(D::Error::custom)
	}
}

impl TryFrom<&str> for VersionConstraint {
	type Error = ParseIntError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"*" => Ok(Self::Any),
			s if s.starts_with('=') => Ok(Self::Equal(Version::try_from(&s[1..])?)),
			s if s.starts_with("<=") => Ok(Self::LessOrEqual(Version::try_from(&s[2..])?)),
			s if s.starts_with('<') => Ok(Self::Less(Version::try_from(&s[1..])?)),
			s if s.starts_with(">=") => Ok(Self::GreaterOrEqual(Version::try_from(&s[2..])?)),
			s if s.starts_with('>') => Ok(Self::Greater(Version::try_from(&s[1..])?)),

			_ => Ok(Self::Equal(Version::try_from(value)?)),
		}
	}
}

impl VersionConstraint {
	/// Tells whether the given version matches the constraint.
	pub fn is_valid(&self, version: &Version) -> bool {
		match self {
			Self::Any => true,
			Self::Equal(v) => matches!(version.cmp(v), Ordering::Equal),
			Self::LessOrEqual(v) => matches!(version.cmp(v), Ordering::Less | Ordering::Equal),
			Self::Less(v) => matches!(version.cmp(v), Ordering::Less),
			Self::GreaterOrEqual(v) => {
				matches!(version.cmp(v), Ordering::Greater | Ordering::Equal)
			}
			Self::Greater(v) => matches!(version.cmp(v), Ordering::Greater),
		}
	}
}

impl Display for VersionConstraint {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Any => write!(f, "*"),
			Self::Equal(v) => write!(f, "={}", v),
			Self::LessOrEqual(v) => write!(f, "<={}", v),
			Self::Less(v) => write!(f, "<{}", v),
			Self::GreaterOrEqual(v) => write!(f, ">={}", v),
			Self::Greater(v) => write!(f, ">{}", v),
		}
	}
}
