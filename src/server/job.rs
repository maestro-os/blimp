//! TODO doc

use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::get;
use actix_web::post;
use actix_web::web;
use common::version::Version;
use crate::global_data::GlobalData;
use serde::Deserialize;
use std::fs;
use std::sync::Mutex;

/// Enumeration of possible job status.
pub enum JobStatus {
	/// The job is pending for one or several other jobs to finish.
	Pending,
	/// The job is running.
	InProgress,
	/// The job ended successfully.
	Success,
	/// The job failed.
	Failed,
	/// The job was aborted.
	Aborted,
}

/// Structure representing a job.
pub struct Job {
	/// The job's ID.
	id: String,

	/// The package's name.
	package: String,
	/// The package's version.
	version: Version,

	/// The job's current status.
	status: JobStatus,
}

impl Job {
	/// Returns the job's HTML representation in the jobs list.
	pub fn get_list_html(&self) -> String {
		let id = &self.id;
		let package = &self.package;
		let version = &self.version;
		let status_html = match self.status {
			JobStatus::Pending => "<td class=\"status-progress\">Pending</td>",
			JobStatus::InProgress => "<td class=\"status-progress\">In progress</td>",
			JobStatus::Success => "<td class=\"status-success\">Success</td>",
			JobStatus::Failed => "<td class=\"status-failed\">Failed</td>",
			JobStatus::Aborted => "<td class=\"status-failed\">Aborted</td>",
		};

		format!("<tr>
			<td><a href=\"/dashboard/job/{id}\">#ID</a></td>
			<td>{package}</td>
			<td>{version}</td>
			{status_html}
		</tr>")
	}
}

#[get("/dashboard/job")]
async fn job_list(
	data: web::Data<Mutex<GlobalData>>,
) -> impl Responder {
	// TODO
	HttpResponse::Ok().body("TODO")
}

#[get("/dashboard/job/{id}")]
async fn job_get(
	data: web::Data<Mutex<GlobalData>>,
	web::Path(id): web::Path<String>,
) -> impl Responder {
	// TODO
	HttpResponse::Ok().body("TODO")
}

#[get("/dashboard/job/{id}/logs")]
async fn job_logs(
	data: web::Data<Mutex<GlobalData>>,
	web::Path(id): web::Path<String>,
) -> impl Responder {
	// TODO Put build logs directory name in constant
	// TODO Check name (security)

	let path = format!("job_logs/{}.log", id);
	fs::read_to_string(&path).unwrap() // TODO Handle error
}

/// Structure representing the query for a request which starts a build job.
#[derive(Deserialize)]
struct JobStartQuery {
	/// The name of the package to build.
	name: String,
	/// The version of the package to build.
	version: String,
}

#[post("/dashboard/job/start")]
async fn job_start(
	data: web::Data<Mutex<GlobalData>>,
	web::Query(query): web::Query<JobStartQuery>,
) -> impl Responder {
	// TODO
	HttpResponse::Ok().body("TODO")
}

#[post("/dashboard/job/{id}/stop")]
async fn job_stop(
	data: web::Data<Mutex<GlobalData>>,
	web::Path(id): web::Path<String>,
) -> impl Responder {
	// TODO
	HttpResponse::Ok().body("TODO")
}
