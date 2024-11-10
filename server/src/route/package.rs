//! Package endpoints.

use crate::Context;
use axum::{
	body::Body,
	extract::{Path, State},
	http::{header::CONTENT_TYPE, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use common::{package, tokio::fs::File, tokio_util::io::ReaderStream, version::Version};
use std::sync::Arc;
use tracing::error;

/// Endpoint to list packages.
pub async fn list(State(ctx): State<Arc<Context>>) -> Response {
	let res = ctx.repo.list_packages();
	match res {
		Ok(packages) => Json(packages).into_response(),
		Err(error) => {
			error!(%error, "could not list packages");
			(StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
		}
	}
}

/// Endpoint to get information about a package.
pub async fn info(
	Path((name, version)): Path<(String, String)>,
	State(ctx): State<Arc<Context>>,
) -> Response {
	if !package::is_valid_name(&name) {
		return (StatusCode::BAD_REQUEST, "invalid package name").into_response();
	}
	let Ok(version) = Version::try_from(version.as_str()) else {
		return (StatusCode::BAD_REQUEST, "invalid package version").into_response();
	};
	let res = ctx.repo.get_package(&name, &version);
	match res {
		Ok(Some(pkg)) => Json(pkg).into_response(),
		Ok(None) => (StatusCode::NOT_FOUND, "package or version not found").into_response(),
		Err(error) => {
			error!(%error, name, %version, "could read package");
			(StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
		}
	}
}

/// Endpoint to get the package's archive.
pub async fn archive(
	Path((name, version)): Path<(String, String)>,
	State(ctx): State<Arc<Context>>,
) -> Response {
	if !package::is_valid_name(&name) {
		return (StatusCode::BAD_REQUEST, "invalid package name").into_response();
	}
	let Ok(version) = Version::try_from(version.as_str()) else {
		return (StatusCode::BAD_REQUEST, "invalid package version").into_response();
	};
	// Check package exists
	let res = ctx.repo.get_package(&name, &version);
	match res {
		Ok(Some(_)) => {}
		Ok(None) => {
			return (StatusCode::NOT_FOUND, "package or version not found").into_response()
		}
		Err(error) => {
			error!(%error, name, %version, "could read package");
			return (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response();
		}
	}
	// Read archive
	let archive_path = ctx.repo.get_archive_path(&name, &version);
	let res = File::open(&archive_path).await;
	let file = match res {
		Ok(f) => f,
		Err(error) => {
			error!(%error, name, %version, path = %archive_path.display(), "could not read package archive");
			return (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response();
		}
	};
	let body = Body::from_stream(ReaderStream::new(file));
	([(CONTENT_TYPE, "application/x-gzip-compressed")], body).into_response()
}
