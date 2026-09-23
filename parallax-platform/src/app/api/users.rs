use serde::Serialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, error::bad_request, route},
};
use crate::db::Database;
use crate::domain::entity::CreateUserInput;

#[derive(Serialize)]
struct UserResponse {
    id: String,
    email: String,
    api_key: String,
    plan_id: String,
    active: bool,
}

#[route(GET "/api/users")]
async fn list_users(cx: &Cx) -> Result<Json<Vec<UserResponse>>> {
    let db = app_context::<Database>(cx);
    let users = db
        .list_users()
        .await
        .into_iter()
        .map(|u| UserResponse {
            id: u.id.to_string(),
            email: u.email,
            api_key: u.api_key,
            plan_id: u.plan_id.to_string(),
            active: u.active,
        })
        .collect();
    Ok(Json(users))
}

#[route(POST "/api/users")]
async fn create_user(cx: &Cx, Json(payload): Json<serde_json::Value>) -> Result<Json<UserResponse>> {
    let input = CreateUserInput::validate_and_parse(&payload)
        .map_err(|e| bad_request(e.to_string()))?;

    let db = app_context::<Database>(cx);
    let plan_id: uuid::Uuid = input.plan_id.parse().map_err(|_| bad_request("invalid plan_id"))?;
    let user = db.create_user(input.email, plan_id).await;
    Ok(Json(UserResponse {
        id: user.id.to_string(),
        email: user.email,
        api_key: user.api_key,
        plan_id: user.plan_id.to_string(),
        active: user.active,
    }))
}
