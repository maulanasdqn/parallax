use sqlx::postgres::PgPool;
use sqlx::Row;
use uuid::Uuid;

use crate::domain::entity::{Carrier, Plan, Session, User};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(url: &str) -> Self {
        let pool = PgPool::connect(url).await.expect("failed to connect to database");
        Self { pool }
    }

    pub async fn migrate(&self) {
        sqlx::query(include_str!("../migrations/001_init.sql"))
            .execute(&self.pool)
            .await
            .ok();
    }

    pub async fn list_users(&self) -> Vec<User> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, plan_id, api_key, active, created_at FROM users ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub async fn create_user(&self, email: String, plan_id: Uuid) -> User {
        let api_key = format!("px_{}", Uuid::new_v4().simple());
        sqlx::query_as::<_, User>(
            "INSERT INTO users (email, plan_id, api_key) VALUES ($1, $2, $3) RETURNING id, email, plan_id, api_key, active, created_at",
        )
        .bind(&email)
        .bind(plan_id)
        .bind(&api_key)
        .fetch_one(&self.pool)
        .await
        .expect("failed to create user")
    }

    pub async fn delete_user(&self, id: Uuid) {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .ok();
    }

    pub async fn toggle_user(&self, id: Uuid) {
        sqlx::query("UPDATE users SET active = NOT active WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .ok();
    }

    pub async fn regenerate_key(&self, id: Uuid) {
        let new_key = format!("px_{}", Uuid::new_v4().simple());
        sqlx::query("UPDATE users SET api_key = $1 WHERE id = $2")
            .bind(&new_key)
            .bind(id)
            .execute(&self.pool)
            .await
            .ok();
    }

    pub async fn list_plans(&self) -> Vec<Plan> {
        sqlx::query_as::<_, Plan>(
            "SELECT id, name, bandwidth_limit_bytes, concurrent_limit, price_cents FROM plans ORDER BY price_cents",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub async fn list_carriers(&self) -> Vec<Carrier> {
        sqlx::query_as::<_, Carrier>(
            "SELECT id, name, country, upstream_addr, online FROM carriers ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub async fn list_sessions(&self) -> Vec<Session> {
        sqlx::query_as::<_, Session>(
            "SELECT id, user_id, carrier_id, bytes_up, bytes_down, started_at, ended_at FROM sessions ORDER BY started_at DESC LIMIT 100",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub async fn user_count(&self) -> i64 {
        sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&self.pool)
            .await
            .map(|r| r.get::<i64, _>("count"))
            .unwrap_or(0)
    }

    pub async fn active_session_count(&self) -> i64 {
        sqlx::query("SELECT COUNT(*) as count FROM sessions WHERE ended_at IS NULL")
            .fetch_one(&self.pool)
            .await
            .map(|r| r.get::<i64, _>("count"))
            .unwrap_or(0)
    }

    pub async fn total_bandwidth(&self) -> i64 {
        sqlx::query("SELECT COALESCE(SUM(bytes_up + bytes_down), 0) as total FROM sessions")
            .fetch_one(&self.pool)
            .await
            .map(|r| r.get::<i64, _>("total"))
            .unwrap_or(0)
    }

    pub async fn carrier_count(&self) -> i64 {
        sqlx::query("SELECT COUNT(*) as count FROM carriers WHERE online = TRUE")
            .fetch_one(&self.pool)
            .await
            .map(|r| r.get::<i64, _>("count"))
            .unwrap_or(0)
    }
}
