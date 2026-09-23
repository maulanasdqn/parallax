#![allow(dead_code)]

mod app;
mod domain;
mod service;

use topcoat::router::{Router, RouterBuilderDiscoverExt};

#[tokio::main]
async fn main() {
    topcoat::start(Router::builder().discover().build())
        .await
        .unwrap();
}
