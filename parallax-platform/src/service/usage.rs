use async_trait::async_trait;
use parallax_core::Result;

use crate::domain::entity::{Plan, UsageRecord, UserId};

#[async_trait]
pub trait UsageService: Send + Sync {
    async fn get_usage(&self, user_id: UserId) -> Result<UsageRecord>;
    async fn check_quota(&self, user_id: UserId) -> Result<bool>;
    async fn get_plan(&self, user_id: UserId) -> Result<Plan>;
}
