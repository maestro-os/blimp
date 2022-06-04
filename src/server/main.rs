mod config;
mod dashboard;
mod global_data;
mod job;
mod package;
mod util;

use actix_web::{
    get,
    middleware,
    web,
    App,
    HttpResponse,
    HttpServer,
    Responder
};
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
    let mut data = data.lock().unwrap();

    HttpResponse::Ok().body(data.get_config().motd.clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // If the config doesn't exist, create it
    if !Config::exists() {
        Config::default().write().unwrap(); // TODO Handle error
    }

    // Reading config and initializing global data
    let config = Config::read().unwrap(); // TODO Handle error
    let port = config.port;

    let data = web::Data::new(Mutex::new(GlobalData::new(config)));

    // Enabling logging
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::new("[%t] %a: %r - Response: %s (in %D ms)"))
            .app_data(data.clone())
            .service(root)
            .service(motd)
            .service(dashboard::home)
            .service(dashboard::package_desc)
            .service(dashboard::style_css)
            .service(dashboard::job_js)
			.service(job::job_get)
			.service(job::job_logs)
			.service(job::job_start)
			.service(job::job_abort)
            .service(package::list)
            .service(package::info)
            .service(package::size)
            .service(package::archive)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
