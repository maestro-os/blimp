//! TODO doc

use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::get;
use actix_web::web;
use common::build_desc::BuildDescriptor;
use crate::global_data::GlobalData;
use std::sync::Mutex;

// TODO login

#[get("/dashboard")]
async fn home(data: web::Data<Mutex<GlobalData>>) -> impl Responder {
	let mut body = include_str!("../../assets/pages/home.html").to_owned();
    let mut data = data.lock().unwrap();

	// Filling available packages
    match data.get_packages() {
        Ok(packages) => {
			let html = if packages.len() == 0 {
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
        Ok(descs) => {
			let html = if descs.len() == 0 {
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

	HttpResponse::Ok().body(body)
}

// TODO Check for a better solution
#[get("/assets/css/style.css")]
async fn style_css() -> impl Responder {
	include_str!("../../assets/css/style.css")
}
