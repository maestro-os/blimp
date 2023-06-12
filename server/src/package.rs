use crate::global_data::GlobalData;
use crate::util;
use actix_files::NamedFile;
use actix_web::{error, get, web, HttpRequest, HttpResponse, Responder};
use common::version::Version;

#[get("/package")]
async fn list(data: web::Data<GlobalData>) -> actix_web::Result<impl Responder> {
	data.repo
		.list_packages()
		.map(|packages| HttpResponse::Ok().json(packages))
		.map_err(|e| error::ErrorInternalServerError(e.to_string()))
}

#[get("/package/{name}/version/{version}")]
async fn info(
	path: web::Path<(String, String)>,
	data: web::Data<GlobalData>,
) -> actix_web::Result<impl Responder> {
	let (name, version) = path.into_inner();

	if !util::is_correct_name(&name) {
		return Err(error::ErrorBadRequest("invalid package name `{name}`"));
	}
	let version =
		Version::try_from(version.as_str()).map_err(|e| error::ErrorNotFound(e.to_string()))?;
	let package = data
		.repo
		.get_package(&name, &version)
		.map_err(|e| error::ErrorInternalServerError(e.to_string()))?;

	match package {
		Some(p) => Ok(HttpResponse::Ok().json(p)),
		None => Ok(HttpResponse::NotFound().finish()),
	}
}

#[get("/package/{name}/version/{version}/archive")]
async fn archive(
	req: HttpRequest,
	path: web::Path<(String, String)>,
	data: web::Data<GlobalData>,
) -> actix_web::Result<impl Responder> {
	let (name, version) = path.into_inner();

	if !util::is_correct_name(&name) {
		return Err(error::ErrorBadRequest("invalid package name `{name}`"));
	}
	let version =
		Version::try_from(version.as_str()).map_err(|e| error::ErrorNotFound(e.to_string()))?;
	let package = data
		.repo
		.get_package(&name, &version)
		.map_err(|e| error::ErrorInternalServerError(e.to_string()))?;

	if package.is_some() {
		let archive_path = data.repo.get_archive_path(&name, &version);

		let req = NamedFile::open(archive_path)
			.map_err(|e| error::ErrorInternalServerError(e.to_string()))?
			.into_response(&req);
		Ok(req)
	} else {
		Ok(HttpResponse::NotFound().finish())
	}
}
