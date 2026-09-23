use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use serde::Serialize;
use uuid::Uuid;

use crate::application::service::AppService;

pub fn routes() -> Router<AppService> {
    Router::new()
        .route("/api/users", get(list).post(create))
        .route("/api/users/{id}", delete(remove))
        .route("/api/users/{id}/toggle", patch(toggle))
        .route("/api/users/{id}/regenerate", post(regenerate))
}

#[derive(Serialize)]
struct UserResponse {
    id: Uuid,
    email: String,
    api_key: String,
    plan_id: Uuid,
    active: bool,
    created_at: String,
}

async fn list(State(svc): State<AppService>) -> impl IntoResponse {
    let users: Vec<UserResponse> = svc
        .list_users()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|u| UserResponse {
            id: u.id,
            email: u.email,
            api_key: u.api_key,
            plan_id: u.plan_id,
            active: u.active,
            created_at: u.created_at.to_rfc3339(),
        })
        .collect();
    Json(users)
}

async fn create(
    State(svc): State<AppService>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match svc.create_user(&payload).await {
        Ok(u) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "id": u.id,
                "email": u.email,
                "api_key": u.api_key,
                "plan_id": u.plan_id,
                "active": u.active,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn remove(State(svc): State<AppService>, Path(id): Path<Uuid>) -> StatusCode {
    svc.delete_user(id).await.ok();
    StatusCode::NO_CONTENT
}

async fn toggle(State(svc): State<AppService>, Path(id): Path<Uuid>) -> StatusCode {
    svc.toggle_user(id).await.ok();
    StatusCode::OK
}

async fn regenerate(
    State(svc): State<AppService>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match svc.regenerate_key(id).await {
        Ok(Some(key)) => Json(serde_json::json!({"api_key": key})).into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}
