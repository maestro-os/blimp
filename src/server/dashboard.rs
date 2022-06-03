//! TODO doc

use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::get;
use actix_web::post;
use actix_web::web;
use crate::global_data::GlobalData;
use serde::Deserialize;
use std::fs;
use std::sync::Mutex;

// TODO login
// TODO clean body

#[get("/dashboard")]
async fn home(data: web::Data<Mutex<GlobalData>>) -> impl Responder {
    let mut data = data.lock().unwrap();
    let packages = data.get_packages();

	let mut body = "Available packages:\n".to_string();

    match packages {
        Ok(packages) => {
			for p in packages {
				body = format!("{}- {}\n", body, p.get_name());
			}
		},

        Err(e) => return HttpResponse::InternalServerError()
			.body(format!("Error: {}", e.to_string())),
    }

	body = format!("{}Available package descriptors:\n", body);
	// TODO

	HttpResponse::Ok().body(body)
}

/// Structure representing the query for a request which returns the build logs of a package.
#[derive(Deserialize)]
struct BuildLogsQuery {
	/// The name of the package.
	name: String,
	/// The version of the package.
	version: String,
}

#[get("/dashboard/build_logs")]
async fn build_logs(
	data: web::Data<Mutex<GlobalData>>,
	web::Query(query): web::Query<BuildLogsQuery>,
) -> impl Responder {
	// TODO Put build logs directory name in constant
	// TODO Check names and versions (security)
	let path = format!("build_logs/{}_{}.log", query.name, query.version);

	fs::read_to_string(&path).unwrap() // TODO Handle error
}

#[get("/dashboard/job")]
async fn job_list(
	data: web::Data<Mutex<GlobalData>>,
) -> impl Responder {
	// TODO
	HttpResponse::Ok().body("TODO")
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

/// Structure representing the query for a request which stops a build job.
#[derive(Deserialize)]
struct JobStopQuery {
	/// The job's ID.
	id: String,
}

#[post("/dashboard/job/stop")]
async fn job_stop(
	data: web::Data<Mutex<GlobalData>>,
	web::Query(query): web::Query<JobStopQuery>,
) -> impl Responder {
	// TODO
	HttpResponse::Ok().body("TODO")
}
