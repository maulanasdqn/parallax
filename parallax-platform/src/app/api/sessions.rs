use serde::Serialize;
use topcoat::{
    Result,
    router::{content::Json, route},
};

#[derive(Serialize)]
struct SessionListResponse {
    sessions: Vec<SessionResponse>,
}

#[derive(Serialize)]
struct SessionResponse {
    id: String,
    carrier: String,
    bytes_up: u64,
    bytes_down: u64,
    started_at: String,
}

#[route(GET)]
async fn sessions() -> Result<Json<SessionListResponse>> {
    Ok(Json(SessionListResponse { sessions: vec![] }))
}
