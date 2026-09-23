use serde::Serialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, route},
};

use crate::db::Database;

#[derive(Serialize)]
struct SessionResponse {
    id: String,
    user_id: String,
    bytes_up: u64,
    bytes_down: u64,
    started_at: String,
    active: bool,
}

#[route(GET "/api/sessions")]
async fn list_sessions(cx: &Cx) -> Result<Json<Vec<SessionResponse>>> {
    let db = app_context::<Database>(cx);
    let sessions = db
        .list_sessions()
        .await
        .into_iter()
        .map(|s| SessionResponse {
            id: s.id.to_string(),
            user_id: s.user_id.to_string(),
            bytes_up: s.bytes_up as u64,
            bytes_down: s.bytes_down as u64,
            started_at: s.started_at.to_rfc3339(),
            active: s.ended_at.is_none(),
        })
        .collect();
    Ok(Json(sessions))
}
