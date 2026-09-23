use serde::Serialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, route},
};

use crate::db::Database;

#[derive(Serialize)]
struct CarrierResponse {
    name: String,
    country: String,
    upstream: String,
    online: bool,
}

#[route(GET "/api/carriers")]
async fn list_carriers(cx: &Cx) -> Result<Json<Vec<CarrierResponse>>> {
    let db = app_context::<Database>(cx);
    let carriers = db
        .list_carriers()
        .await
        .into_iter()
        .map(|c| CarrierResponse {
            name: c.name,
            country: c.country,
            upstream: c.upstream_addr,
            online: c.online,
        })
        .collect();
    Ok(Json(carriers))
}
