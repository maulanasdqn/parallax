use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use uuid::Uuid;

use crate::application::service::AppService;

pub fn routes() -> Router<AppService> {
    Router::new().route("/api/carriers", get(list))
}

#[derive(Serialize)]
struct CarrierResponse {
    id: Uuid,
    name: String,
    country: String,
    upstream_addr: String,
    online: bool,
}

async fn list(State(svc): State<AppService>) -> Json<Vec<CarrierResponse>> {
    let carriers = svc
        .list_carriers()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|c| CarrierResponse {
            id: c.id,
            name: c.name,
            country: c.country,
            upstream_addr: c.upstream_addr,
            online: c.online,
        })
        .collect();
    Json(carriers)
}
