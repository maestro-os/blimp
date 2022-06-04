use actix_files::NamedFile;
use actix_web::{
    get,
    http::header::ContentType,
    web,
    HttpResponse,
    Responder
};
use common::package::Package;
use common::package;
use common::request::PackageListResponse;
use common::request::PackageSizeResponse;
use common::version::Version;
use crate::GlobalData;
use serde_json::json;
use std::fs::File;
use std::sync::Mutex;

#[get("/package")]
async fn list(data: web::Data<Mutex<GlobalData>>) -> impl Responder {
    let mut data = data.lock().unwrap();

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
    let version = Version::from_string(&version).unwrap(); // TODO Handle error

    // Getting package
    let package = Package::get(&name.to_owned(), &version).unwrap(); // TODO Handle error

    match package {
        Some(_) => {
            let archive_path = format!("{}/{}-{}", package::SERVER_PACKAGES_DIR, name, version);
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
	web::Path((name, version)): web::Path<(String, String)>,
) -> impl Responder {
    let version = Version::from_string(&version).unwrap(); // TODO Handle error

    let archive_path = format!("{}/{}-{}", package::SERVER_PACKAGES_DIR, name, version);
    NamedFile::open(archive_path) // TODO Make the error message cleaner
}
