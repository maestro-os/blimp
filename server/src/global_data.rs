//! This module implements the global data structure.

use crate::config::Config;
use crate::job::Job;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

/// Structure storing data used all across the server.
pub struct GlobalData {
	/// The server's configuration.
	pub config: Config,

	/// The list of jobs.
	pub jobs: Vec<Job>,
}

impl GlobalData {
	/// Returns an immutable reference to the job with id `id`.
	pub fn get_job(&self, id: &str) -> Option<&Job> {
		self.jobs.iter()
			.filter(|j| j.get_desc().id == id)
			.next()
	}

	/// Returns a mutable reference to the job with id `id`.
	pub fn get_job_mut(&mut self, id: &str) -> Option<&mut Job> {
		self.jobs
			.iter_mut()
			.filter(|j| j.get_desc().id == id)
			.next()
	}

	// FIXME Possible data race. Create a function that creates the job directly
	/// Returns an unused ID for a new job.
	pub fn new_job_id(&self) -> String {
		loop {
			let id: String = thread_rng()
				.sample_iter(&Alphanumeric)
				.take(15)
				.map(char::from)
				.collect();

			if self.get_job(&id).is_none() {
				return id;
			}
		}
	}
}
