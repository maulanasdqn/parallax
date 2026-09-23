use serde::Serialize;
use topcoat::{
    Result,
    router::{content::Json, route},
};

use crate::domain::entity::Carrier;

#[derive(Serialize)]
struct CarrierListResponse {
    carriers: Vec<CarrierResponse>,
}

#[derive(Serialize)]
struct CarrierResponse {
    name: String,
    country: String,
    online: bool,
}

impl From<Carrier> for CarrierResponse {
    fn from(c: Carrier) -> Self {
        Self {
            name: c.name,
            country: c.country,
            online: c.online,
        }
    }
}

#[route(GET)]
async fn carriers() -> Result<Json<CarrierListResponse>> {
    Ok(Json(CarrierListResponse { carriers: vec![] }))
}
