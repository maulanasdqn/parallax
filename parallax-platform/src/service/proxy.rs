use async_trait::async_trait;
use parallax_core::Result;

use crate::domain::entity::{Credential, Session, UserId};

#[async_trait]
pub trait ProxyService: Send + Sync {
    async fn start_session(&self, credential: &Credential) -> Result<Session>;
    async fn end_session(&self, session: &Session, bytes_up: u64, bytes_down: u64) -> Result<()>;
    async fn active_sessions(&self, user_id: UserId) -> Result<Vec<Session>>;
    async fn generate_credentials(&self, user_id: UserId, carrier: &str) -> Result<Credential>;
}
