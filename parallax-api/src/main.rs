mod adapter;
mod application;
mod domain;

use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use adapter::postgres::PostgresRepo;
use application::service::AppService;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    let repo = PostgresRepo::connect(&db_url).await;
    repo.migrate().await;

    let repo = Arc::new(repo);
    let service = AppService::new(
        Arc::clone(&repo) as Arc<dyn domain::port::UserRepository>,
        Arc::clone(&repo) as Arc<dyn domain::port::PlanRepository>,
        Arc::clone(&repo) as Arc<dyn domain::port::CarrierRepository>,
        repo as Arc<dyn domain::port::SessionRepository>,
    );

    let listen: SocketAddr = std::env::var("API_LISTEN")
        .unwrap_or_else(|_| "0.0.0.0:3001".into())
        .parse()
        .expect("invalid API_LISTEN");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = adapter::http::router(service).layer(cors);

    info!(addr = %listen, "api server starting");
    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
