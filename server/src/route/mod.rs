//! Endpoints implementations.

pub mod package;

use crate::Context;
use axum::extract::State;
use std::sync::Arc;

pub async fn root() -> &'static str {
	concat!("Blimp server version ", env!("CARGO_PKG_VERSION"))
}

pub async fn motd(State(ctx): State<Arc<Context>>) -> String {
	ctx.motd.clone()
}
