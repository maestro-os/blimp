//! This module implements the global data structure. 

use crate::config::Config;
use crate::job::Job;

/// Structure storing data used all across the server.
pub struct GlobalData {
    /// The server's configuration.
    config: Config,

    /// The list of jobs.
    jobs: Vec<Job>,
}

impl GlobalData {
    /// Creates a new instance with the given configuration.
    pub fn new(config: Config) -> Self {
		// TODO Read jobs from file

        Self {
            config,

            jobs: Vec::new(),
        }
    }

    /// Returns a mutable refrence to the configuration.
    pub fn get_config(&mut self) -> &mut Config {
        &mut self.config
    }

	/// Returns an immutable reference to the list of jobs.
	pub fn get_jobs(&self) -> &Vec<Job> {
		&self.jobs
	}

	/// Returns a mutable reference to the list of jobs.
	pub fn get_jobs_mut(&mut self) -> &mut Vec<Job> {
		&mut self.jobs
	}

	/// Returns an immutable reference to the job with id `id`.
	pub fn get_job(&self, id: &str) -> Option<&Job> {
		self.jobs.iter()
			.filter(| j | j.get_desc().id == id)
			.next()
	}

	/// Returns a mutable reference to the job with id `id`.
	pub fn get_job_mut(&mut self, id: &str) -> Option<&mut Job> {
		self.jobs.iter_mut()
			.filter(| j | j.get_desc().id == id)
			.next()
	}

	/// Returns an unused ID for a new job.
	pub fn new_job_id(&self) -> String {
		// TODO
		"TODO".to_owned()
	}
}
