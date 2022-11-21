//! This module implements the global data structure.

use common::repository::Repository;

/// Structure storing data used all across the server.
pub struct GlobalData {
	/// The server's motd.
	pub motd: String,

	/// The server's repository.
	pub repo: Repository,
}
