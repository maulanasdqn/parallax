use axum::{Router, routing::get};

use crate::application::service::AppService;

pub fn routes() -> Router<AppService> {
    Router::new().route("/api/health", get(health))
}

async fn health() -> &'static str {
    "ok"
}
