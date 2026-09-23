use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zod_rs::prelude::*;

pub type UserId = Uuid;
pub type PlanId = Uuid;
pub type CarrierId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub plan_id: Uuid,
    pub api_key: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Plan {
    pub id: Uuid,
    pub name: String,
    pub bandwidth_limit_bytes: i64,
    pub concurrent_limit: i32,
    pub price_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Carrier {
    pub id: Uuid,
    pub name: String,
    pub country: String,
    pub upstream_addr: String,
    pub online: bool,
}

#[derive(Debug, Serialize, Deserialize, ZodSchema)]
pub struct CreateUserInput {
    #[zod(email)]
    pub email: String,
    #[zod(min_length(1))]
    pub plan_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub carrier: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ZodSchema)]
pub struct CreateCarrierInput {
    #[zod(min_length(2), max_length(50))]
    pub name: String,
    #[zod(min_length(2), max_length(10))]
    pub country: String,
    #[zod(min_length(5))]
    pub upstream_addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub carrier_id: Option<Uuid>,
    pub bytes_up: i64,
    pub bytes_down: i64,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub user_id: Uuid,
    pub total_bytes: u64,
    pub session_count: u64,
}
