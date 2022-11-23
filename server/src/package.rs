use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use common::version::Version;
use crate::global_data::GlobalData;
use crate::util;
use std::sync::Mutex;

#[get("/package")]
async fn list(
	data: web::Data<Mutex<GlobalData>>,
) -> impl Responder {
	let repo = &data.lock().unwrap().repo;

	match repo.list_packages() {
		Ok(packages) => HttpResponse::Ok().json(packages),
		Err(_) => HttpResponse::InternalServerError().finish(),
	}
}

#[get("/package/{name}/version/{version}")]
async fn info(
	web::Path((name, version)): web::Path<(String, String)>,
	data: web::Data<Mutex<GlobalData>>,
) -> impl Responder {
	if !util::is_correct_name(&name) {
		return HttpResponse::NotFound().finish();
	}
	let version = Version::try_from(version.as_str()).unwrap(); // TODO Handle error

	let repo = &data.lock().unwrap().repo;
	let package = repo.get_package(&name, &version).unwrap(); // TODO Handle error

	match package {
		Some(p) => HttpResponse::Ok().json(p),
		None => HttpResponse::NotFound().finish(),
	}
}

#[get("/package/{name}/version/{version}/archive")]
async fn archive(
	req: HttpRequest,
	web::Path((name, version)): web::Path<(String, String)>,
	data: web::Data<Mutex<GlobalData>>,
) -> impl Responder {
	if !util::is_correct_name(&name) {
		return HttpResponse::NotFound().finish();
	}
	let version = Version::try_from(version.as_str()).unwrap(); // TODO Handle error

	let repo = &data.lock().unwrap().repo;
	let package = repo.get_package(&name, &version).unwrap(); // TODO Handle error

	if package.is_some() {
		let archive_path = repo.get_archive_path(&name, &version);

		// TODO Handle error
		NamedFile::open(archive_path)
			.unwrap()
			.into_response(&req)
			.unwrap()
	} else {
		HttpResponse::NotFound().finish()
	}
}
