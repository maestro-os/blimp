//! The blimp server serves packages to be installed by the package manager.

mod config;
mod global_data;
mod package;
mod util;

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use common::repository::Repository;
use config::Config;
use global_data::GlobalData;
use std::env;
use std::io;

#[get("/")]
async fn root() -> impl Responder {
	let body = format!("Blimp server version {}", env!("CARGO_PKG_VERSION"));
	HttpResponse::Ok().body(body)
}

#[get("/motd")]
async fn motd(data: web::Data<GlobalData>) -> impl Responder {
	HttpResponse::Ok().body(data.motd.clone())
}

#[actix_web::main]
async fn main() -> io::Result<()> {
	// Reading config and initializing global data
	let config = Config::read()?;
	let port = config.port;

	let data = web::Data::new(GlobalData {
		motd: config.motd,

		repo: Repository::load(config.repo_path.clone())?,
	});

	// Enabling logging
	env::set_var("RUST_LOG", "actix_web=info");
	env_logger::init();

	HttpServer::new(move || {
		App::new()
			.wrap(middleware::Logger::new(
				"[%t] %a: %r - Response: %s (in %D ms)",
			))
			.app_data(data.clone())
			.service(root)
			.service(motd)
			.service(package::list)
			.service(package::info)
			.service(package::archive)
	})
	.bind(format!("0.0.0.0:{}", port))?
	.run()
	.await
}
