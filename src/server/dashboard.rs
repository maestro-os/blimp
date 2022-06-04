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

#[get("/dashboard")]
async fn home(data: web::Data<Mutex<GlobalData>>) -> impl Responder {
    let mut data = data.lock().unwrap();
    let packages = data.get_packages();

	let mut body = include_str!("../../assets/pages/home.html").to_owned();

    match packages {
        Ok(packages) => {
			let available_packages = if packages.len() == 0 {
				"<p><b>No available packages</b></p>".to_owned()
			} else {
				let mut available_packages = String::new();

				for p in packages {
					available_packages += &format!("<li>{}</li>\n", &p.get_name());
					available_packages += "<ul>\n";

					for v in p.get_versions() {
						available_packages += &format!("<li><a href=\"/dashboard/package/{}/version/{}\">{}</a></li>\n", p.get_name(), v, v);
					}

					available_packages += "</ul>\n";
				}

				available_packages
			};

			body = body.replace("{available_packages}", &available_packages);
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
