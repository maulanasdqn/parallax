use async_trait::async_trait;
use parallax_core::Result;

use crate::domain::entity::{Credential, User, UserId};

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn authenticate_api_key(&self, api_key: &str) -> Result<User>;
    async fn authenticate_proxy(&self, credential: &Credential) -> Result<User>;
    async fn generate_api_key(&self, user_id: UserId) -> Result<String>;
}
