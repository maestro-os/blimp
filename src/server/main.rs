mod config;

use actix_web::{
    get,
    http::header::ContentType,
    middleware,
    web,
    App,
    HttpResponse,
    HttpServer,
    Responder
};
use common::package::Package;
use config::Config;
use serde_json::json;
use std::sync::Mutex;

/// The server's version.
const VERSION: &str = "0.1";

/// Structure storing data used all across the server.
struct GlobalData {
    /// The server's configuration.
    config: Config,
}

#[get("/")]
async fn root() -> impl Responder {
    let body = format!("Blimp server version {}", VERSION);
    HttpResponse::Ok().body(body)
}

#[get("/motd")]
async fn motd(data: web::Data<Mutex<GlobalData>>) -> impl Responder {
    let data = data.lock().unwrap();
    HttpResponse::Ok().body(data.config.motd.clone())
}

#[get("/package")]
async fn package_list() -> impl Responder {
    let json = json!({
        "packages": Package::server_list(),
    });

    HttpResponse::Ok().set(ContentType::json()).body(json.to_string())
}

// TODO Add endpoint to get details on a specific package/version
// TODO Add endpoint to download the package with the given version

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // If the config doesn't exist, create it
    if !Config::exists() {
        Config::default().write().unwrap(); // TODO Handle error
    }

    // Reading config and initializing global data
    let config = Config::read().unwrap(); // TODO Handle error
    let port = config.port;

    let data = web::Data::new(Mutex::new(GlobalData {
        config,
    }));

    // Enabling logging
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::new("[%t] %a: %r - Response: %s (in %D ms)"))
            .app_data(data.clone())
            .service(root)
            .service(motd)
            .service(package_list)
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
