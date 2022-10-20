//! TODO doc

/// Structure representing a local repository.
pub struct LocalRepository {
	/// The path to the repository.
	path: String,
}

impl LocalRepository {
	/// Creates a new instance from the given path.
	pub fn new(path: String) -> Self {
		Self {
			path,
		}
	}

	/// Returns the latest version of the package with name `name` along with its associated
	/// repository.
	/// If the package doesn't exist, the function returns None.
	///
	/// Arguments:
	/// - `repos` is the list of repositories to check on.
	/// - `sysroot` is the path to the system's root.
	pub fn get_latest_package<'a>(
		repos: &'a [Repository],
		sysroot: &str,
		name: &str
	) -> io::Result<Option<(&'a Repository, Package)>> {
		// TODO
		todo!();
	}
}
