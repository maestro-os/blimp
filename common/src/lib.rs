//! This library contains common code between the client and the server.

pub mod build_desc;
pub mod install;
pub mod lockfile;
pub mod package;
pub mod repository;
pub mod request;
pub mod util;
pub mod version;

#[cfg(feature = "network")]
pub mod download;
