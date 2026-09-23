mod carriers;
mod health;
mod sessions;
mod users;

use axum::Router;

use crate::application::service::AppService;

pub fn router(service: AppService) -> Router {
    Router::new()
        .merge(health::routes())
        .merge(users::routes())
        .merge(carriers::routes())
        .merge(sessions::routes())
        .with_state(service)
}
