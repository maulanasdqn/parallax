use async_trait::async_trait;
use uuid::Uuid;

use super::entity::{Carrier, Plan, Session, User};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<User>>;
    async fn find_by_api_key(&self, api_key: &str) -> Result<Option<User>>;
    async fn create(&self, email: String, plan_id: Uuid) -> Result<User>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn toggle_active(&self, id: Uuid) -> Result<()>;
    async fn regenerate_api_key(&self, id: Uuid) -> Result<Option<String>>;
}

#[async_trait]
pub trait PlanRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Plan>>;
}

#[async_trait]
pub trait CarrierRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Carrier>>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Session>>;
    async fn count_active(&self) -> Result<i64>;
    async fn total_bandwidth(&self) -> Result<i64>;
}
