use topcoat::{Result, router::route};

#[route(GET)]
async fn health() -> Result<&'static str> {
    Ok("ok")
}
