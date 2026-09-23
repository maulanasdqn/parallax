use async_trait::async_trait;
use parallax_core::Result;

use super::entity::{
    Carrier, CarrierId, Plan, PlanId, Session, SessionId, UsageRecord, User, UserId,
};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn find_by_api_key(&self, api_key: &str) -> Result<Option<User>>;
    async fn list(&self) -> Result<Vec<User>>;
    async fn create(&self, user: User) -> Result<User>;
    async fn update(&self, user: User) -> Result<User>;
    async fn delete(&self, id: UserId) -> Result<()>;
}

#[async_trait]
pub trait PlanRepository: Send + Sync {
    async fn find_by_id(&self, id: PlanId) -> Result<Option<Plan>>;
    async fn list(&self) -> Result<Vec<Plan>>;
    async fn create(&self, plan: Plan) -> Result<Plan>;
}

#[async_trait]
pub trait CarrierRepository: Send + Sync {
    async fn find_by_id(&self, id: CarrierId) -> Result<Option<Carrier>>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Carrier>>;
    async fn list(&self) -> Result<Vec<Carrier>>;
    async fn list_online(&self) -> Result<Vec<Carrier>>;
    async fn create(&self, carrier: Carrier) -> Result<Carrier>;
    async fn set_online(&self, id: CarrierId, online: bool) -> Result<()>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_by_id(&self, id: SessionId) -> Result<Option<Session>>;
    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<Session>>;
    async fn list_active(&self) -> Result<Vec<Session>>;
    async fn create(&self, session: Session) -> Result<Session>;
    async fn end_session(&self, id: SessionId, bytes_up: u64, bytes_down: u64) -> Result<()>;
}

#[async_trait]
pub trait UsageRepository: Send + Sync {
    async fn get_usage(&self, user_id: UserId) -> Result<UsageRecord>;
}
