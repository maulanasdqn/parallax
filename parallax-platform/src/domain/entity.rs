use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type UserId = Uuid;
pub type PlanId = Uuid;
pub type CarrierId = Uuid;
pub type SessionId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub plan_id: PlanId,
    pub api_key: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: PlanId,
    pub name: String,
    pub bandwidth_limit_bytes: u64,
    pub concurrent_limit: u32,
    pub price_cents: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Carrier {
    pub id: CarrierId,
    pub name: String,
    pub country: String,
    pub upstream_addr: String,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub carrier: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub carrier_id: CarrierId,
    pub bytes_up: u64,
    pub bytes_down: u64,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub user_id: UserId,
    pub total_bytes: u64,
    pub session_count: u64,
}
