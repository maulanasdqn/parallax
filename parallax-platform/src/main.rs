#![allow(dead_code)]

mod app;
mod db;
mod domain;
mod infra;
mod service;

use db::Database;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{Router, RouterBuilderDiscoverExt},
};

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    let db = Database::connect(&db_url).await;
    db.migrate().await;

    topcoat::start(
        Router::builder()
            .discover()
            .assets(AssetBundle::load().unwrap())
            .app_context(db)
            .build(),
    )
    .await
    .unwrap();
}
