use serde::{Deserialize, Serialize};
use topcoat::{
    Result,
    router::{content::Json, route},
};

#[derive(Deserialize)]
struct CreateUserRequest {
    email: String,
    plan_id: String,
}

#[derive(Serialize)]
struct UserResponse {
    id: String,
    email: String,
    api_key: String,
    active: bool,
}

#[route(POST)]
async fn users(Json(req): Json<CreateUserRequest>) -> Result<Json<UserResponse>> {
    let id = uuid::Uuid::new_v4();
    let api_key = uuid::Uuid::new_v4();
    Ok(Json(UserResponse {
        id: id.to_string(),
        email: req.email,
        api_key: api_key.to_string(),
        active: true,
    }))
}
