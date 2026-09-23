use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use uuid::Uuid;

use crate::application::service::AppService;

pub fn routes() -> Router<AppService> {
    Router::new().route("/api/sessions", get(list))
}

#[derive(Serialize)]
struct SessionResponse {
    id: Uuid,
    user_id: Uuid,
    carrier_id: Option<Uuid>,
    bytes_up: i64,
    bytes_down: i64,
    started_at: String,
    active: bool,
}

async fn list(State(svc): State<AppService>) -> Json<Vec<SessionResponse>> {
    let sessions = svc
        .list_sessions()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|s| SessionResponse {
            id: s.id,
            user_id: s.user_id,
            carrier_id: s.carrier_id,
            bytes_up: s.bytes_up,
            bytes_down: s.bytes_down,
            started_at: s.started_at.to_rfc3339(),
            active: s.ended_at.is_none(),
        })
        .collect();
    Json(sessions)
}
