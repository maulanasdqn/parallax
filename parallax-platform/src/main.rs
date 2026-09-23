mod api;
mod app;

use api::ApiClient;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{Router, RouterBuilderDiscoverExt},
};

#[tokio::main]
async fn main() {
    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:3001".into());
    let client = ApiClient::new(api_url);

    topcoat::start(
        Router::builder()
            .discover()
            .assets(AssetBundle::load().unwrap())
            .app_context(client)
            .build(),
    )
    .await
    .unwrap();
}
