use async_trait::async_trait;
use parallax_core::Result;

use crate::domain::entity::{Carrier, CarrierId};

#[async_trait]
pub trait CarrierService: Send + Sync {
    async fn list_available(&self) -> Result<Vec<Carrier>>;
    async fn get_status(&self, id: CarrierId) -> Result<Carrier>;
    async fn register(&self, name: String, country: String, upstream: String) -> Result<Carrier>;
    async fn set_online(&self, id: CarrierId, online: bool) -> Result<()>;
}
