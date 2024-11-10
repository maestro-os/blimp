//! The blimp server serves packages to be installed by the package manager.

mod route;

use axum::{routing::get, Router};
use common::{repository::Repository, tokio};
use std::{io, sync::Arc};
use std::path::PathBuf;
use serde::Deserialize;

/// The server's configuration.
#[derive(Deserialize)]
pub struct Config {
	/// The server's port.
	pub port: u16,
	/// The server's motd.
	pub motd: String,
	/// The path to the repository containing the server's packages.
	pub repo_path: String,
}

/// The server's global context.
pub struct Context {
	/// The server's motd.
	motd: String,
	/// The server's repository.
	repo: Repository,
}

#[tokio::main]
async fn main() -> io::Result<()> {
	tracing_subscriber::fmt::init();
	let config: Config = envy::from_env().expect("configuration");
	let ctx = Arc::new(Context {
		motd: config.motd,
		repo: Repository::load(PathBuf::from(config.repo_path))?,
	});
	let router = Router::new()
		.route("/", get(route::root))
		.route("/motd", get(route::motd))
		.route("/package", get(route::package::list))
		.route("/package/:name/version/:version", get(route::package::info))
		.route(
			"/package/:name/version/:version/archive",
			get(route::package::archive),
		)
		// TODO logging layer
		.with_state(ctx);
	let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
	axum::serve(listener, router).await
}
