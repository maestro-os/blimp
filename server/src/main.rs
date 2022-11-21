mod config;
mod global_data;
mod package;
mod util;

use actix_web::{
	App,
	HttpResponse,
	HttpServer,
	Responder,
	get,
	middleware,
	web,
};
use common::repository::Repository;
use config::Config;
use global_data::GlobalData;
use std::sync::Mutex;

/// The server's version.
const VERSION: &str = "0.1";

#[get("/")]
async fn root() -> impl Responder {
	let body = format!("Blimp server version {}", VERSION);
	HttpResponse::Ok().body(body)
}

#[get("/motd")]
async fn motd(data: web::Data<Mutex<GlobalData>>) -> impl Responder {
	let data = data.lock().unwrap();
	HttpResponse::Ok().body(&data.motd)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	// Reading config and initializing global data
	let config = Config::read().unwrap(); // TODO Handle error
	let port = config.port;

	let data = web::Data::new(Mutex::new(GlobalData {
		motd: config.motd,

		repo: Repository::load(config.repo_path.clone()).unwrap(), // TODO Handle error
	}));

	// Enabling logging
	std::env::set_var("RUST_LOG", "actix_web=info");
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
			.service(package::size)
			.service(package::archive)
	})
	.bind(format!("0.0.0.0:{}", port))?
	.run()
	.await
}
