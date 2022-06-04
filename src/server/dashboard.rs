//! TODO doc

use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::get;
use actix_web::web;
use common::build_desc::BuildDescriptor;
use common::package::Package;
use common::version::Version;
use crate::global_data::GlobalData;
use crate::job::Job;
use crate::util;
use std::sync::Mutex;

// TODO login

#[get("/dashboard")]
async fn home(data: web::Data<Mutex<GlobalData>>) -> impl Responder {
	let data = data.lock().unwrap();
	let mut body = include_str!("../../assets/pages/home.html").to_owned();

	// Filling available packages
    match Package::server_list() {
        Ok(mut packages) => {
			packages.sort_by(| n0, n1 | {
				n0.get_name().cmp(n1.get_name())
			});

			let html = if packages.is_empty() {
				"<p><b>No available packages</b></p>".to_owned()
			} else {
				let mut html = String::new();

				for p in packages {
					html += &format!("<li>{}</li>\n", &p.get_name());
					html += "<ul>\n";

					// TODO
					/*for v in p.get_versions() {
						html += &format!("<li><a href=\"/dashboard/package/{}/version/{}\">{}</a></li>\n", p.get_name(), v, v);
					}*/
					html += &format!("<li><a href=\"/dashboard/package/{}/version/{}\">{}</a></li>\n", p.get_name(), p.get_version(), p.get_version());

					html += "</ul>\n";
				}

				html
			};

			body = body.replace("{available_packages}", &html);
		},

        Err(e) => return HttpResponse::InternalServerError()
			.body(format!("Error: {}", e.to_string())),
    }

	// Filling available build descriptors
    match BuildDescriptor::server_list() {
        Ok(mut descs) => {
			descs.sort_by(| n0, n1 | {
				n0.1.get_package().get_name().cmp(n1.1.get_package().get_name())
			});

			let html = if descs.is_empty() {
				"<p><b>No available packages</b></p>".to_owned()
			} else {
				let mut html = String::new();

				for (_, d) in descs {
					let p = d.get_package();

					html += &format!("<li>{}</li>\n", &p.get_name());
					html += "<ul>\n";

					// TODO
					/*for v in p.get_versions() {
						html += &format!("<li><a href=\"/dashboard/package_desc/{}/version/{}\">{}</a></li>\n", p.get_name(), v, v);
					}*/
					html += &format!("<li><a href=\"/dashboard/package_desc/{}/version/{}\">{}</a></li>\n", p.get_name(), p.get_version(), p.get_version());

					html += "</ul>\n";
				}

				html
			};

			body = body.replace("{packages}", &html);
		},

        Err(e) => return HttpResponse::InternalServerError()
			.body(format!("Error: {}", e.to_string())),
    }

	// Filling jobs list
	let jobs = data.get_jobs();
	if jobs.is_empty() {
		body = body.replace("{jobs}", "<p><b>No jobs</b></p>");
	} else {
		let mut html = String::new();

		for j in jobs {
			html = format!("{}{}", html, j.get_list_html());
		}

		body = body.replace("{jobs}", &html);
	}

	HttpResponse::Ok().body(body)
}

#[get("/dashboard/package_desc/{name}/version/{version}")]
async fn package_desc(
	_data: web::Data<Mutex<GlobalData>>,
	web::Path((name, version)): web::Path<(String, Version)>,
) -> impl Responder {
	if !util::is_correct_name(&name) {
		return HttpResponse::NotFound().finish();
	}

	// TODO Handle error
    // Getting descriptor
    match BuildDescriptor::server_get(&name.to_owned(), &version).unwrap() {
		Some((_, desc)) => {
			let mut body = include_str!("../../assets/pages/package_desc.html").to_owned();

			let package = desc.get_package();
			body = body.replace("{name}", package.get_name());
			body = body.replace("{version}", &package.get_version().to_string());
			body = body.replace("{description}", package.get_description());
			// TODO Build deps
			// TODO Run deps

			HttpResponse::Ok().body(body)
		},

		None => HttpResponse::NotFound().finish(),
	}
}

// TODO Check for a better solution
#[get("/assets/css/style.css")]
async fn style_css() -> impl Responder {
	include_str!("../../assets/css/style.css")
}
