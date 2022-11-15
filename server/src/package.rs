use actix_files::NamedFile;
use actix_web::{
	HttpRequest,
    HttpResponse,
    Responder,
    get,
    http::header::ContentType,
    web,
};
use common::package::Package;
use common::request::PackageListResponse;
use common::request::PackageSizeResponse;
use common::version::Version;
use crate::util;
use serde_json::json;
use std::fs::File;

#[get("/package")]
async fn list() -> impl Responder {
    match Package::server_list() {
        Ok(packages) => HttpResponse::Ok().json(PackageListResponse {
			packages: packages.to_vec(),
		}),

        Err(e) => HttpResponse::InternalServerError().json(json!({
			"error": e.to_string(),
		})),
    }
}

#[get("/package/{name}/version/{version}")]
async fn info(
	web::Path((name, version)): web::Path<(String, String)>,
) -> impl Responder {
	if !util::is_correct_name(&name) {
		return HttpResponse::NotFound().finish();
	}
    let version = Version::from_string(&version).unwrap(); // TODO Handle error

    // Getting package
    let package = Package::get(&name.to_owned(), &version).unwrap(); // TODO Handle error

    match package {
        Some(p) => HttpResponse::Ok().json(p),

        None => HttpResponse::NotFound().json(json!({
			"error": format!("Package `{}` with version `{}` not found", name, version),
		})),
    }
}

#[get("/package/{name}/version/{version}/size")]
async fn size(
	web::Path((name, version)): web::Path<(String, String)>,
) -> impl Responder {
	if !util::is_correct_name(&name) {
		return HttpResponse::NotFound().finish();
	}
    let version = Version::from_string(&version).unwrap(); // TODO Handle error

    // Getting package
    let package = Package::get(&name.to_owned(), &version).unwrap(); // TODO Handle error

    match package {
        Some(package) => {
			let archive_path = package.get_archive_path();
            let file = File::open(archive_path).unwrap(); // TODO Handle error
            let size = file.metadata().unwrap().len(); // TODO Handle error

            HttpResponse::Ok().json(PackageSizeResponse {
                size,
            })
        },

        None => {
            let json = json!({
                "error": format!("Package `{}` with version `{}` not found", name, version),
            });
            HttpResponse::NotFound().set(ContentType::json()).body(json)
        },
    }
}

#[get("/package/{name}/version/{version}/archive")]
async fn archive(
	req: HttpRequest,
	web::Path((name, version)): web::Path<(String, String)>,
) -> impl Responder {
	let version = Version::from_string(&version).unwrap(); // TODO Handle error

    // Getting package
    let package = Package::get(&name.to_owned(), &version).unwrap(); // TODO Handle error

    match package {
        Some(package) => {
			let archive_path = package.get_archive_path();
			// TODO Handle error
			NamedFile::open(archive_path)
				.unwrap()
				.into_response(&req)
				.unwrap()
        },

        None => {
            let json = json!({
                "error": format!("Package `{}` with version `{}` not found", name, version),
            });
            HttpResponse::NotFound().set(ContentType::json()).body(json)
        },
    }
}