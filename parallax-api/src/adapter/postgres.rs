use async_trait::async_trait;
use sqlx::postgres::PgPool;
use sqlx::Row;
use uuid::Uuid;

use crate::domain::entity::{Carrier, Plan, Session, User};
use crate::domain::port::{
    CarrierRepository, PlanRepository, Result, SessionRepository, UserRepository,
};

#[derive(Clone)]
pub struct PostgresRepo {
    pool: PgPool,
}

impl PostgresRepo {
    pub async fn connect(url: &str) -> Self {
        let pool = PgPool::connect(url).await.expect("failed to connect to database");
        Self { pool }
    }

    pub async fn migrate(&self) {
        sqlx::query(include_str!("../../migrations/001_init.sql"))
            .execute(&self.pool)
            .await
            .ok();
    }
}

#[async_trait]
impl UserRepository for PostgresRepo {
    async fn find_all(&self) -> Result<Vec<User>> {
        Ok(sqlx::query_as::<_, User>(
            "SELECT id, email, plan_id, api_key, active, created_at FROM users ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_api_key(&self, api_key: &str) -> Result<Option<User>> {
        Ok(sqlx::query_as::<_, User>(
            "SELECT id, email, plan_id, api_key, active, created_at FROM users WHERE api_key = $1",
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn create(&self, email: String, plan_id: Uuid) -> Result<User> {
        let api_key = format!("px_{}", Uuid::new_v4().simple());
        Ok(sqlx::query_as::<_, User>(
            "INSERT INTO users (email, plan_id, api_key) VALUES ($1, $2, $3) RETURNING id, email, plan_id, api_key, active, created_at",
        )
        .bind(&email)
        .bind(plan_id)
        .bind(&api_key)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn toggle_active(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET active = NOT active WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn regenerate_api_key(&self, id: Uuid) -> Result<Option<String>> {
        let new_key = format!("px_{}", Uuid::new_v4().simple());
        let result = sqlx::query("UPDATE users SET api_key = $1 WHERE id = $2")
            .bind(&new_key)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(if result.rows_affected() > 0 { Some(new_key) } else { None })
    }
}

#[async_trait]
impl PlanRepository for PostgresRepo {
    async fn find_all(&self) -> Result<Vec<Plan>> {
        Ok(sqlx::query_as::<_, Plan>(
            "SELECT id, name, bandwidth_limit_bytes, concurrent_limit, price_cents FROM plans ORDER BY price_cents",
        )
        .fetch_all(&self.pool)
        .await?)
    }
}

#[async_trait]
impl CarrierRepository for PostgresRepo {
    async fn find_all(&self) -> Result<Vec<Carrier>> {
        Ok(sqlx::query_as::<_, Carrier>(
            "SELECT id, name, country, upstream_addr, online FROM carriers ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?)
    }
}

#[async_trait]
impl SessionRepository for PostgresRepo {
    async fn find_all(&self) -> Result<Vec<Session>> {
        Ok(sqlx::query_as::<_, Session>(
            "SELECT id, user_id, carrier_id, bytes_up, bytes_down, started_at, ended_at FROM sessions ORDER BY started_at DESC LIMIT 100",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn count_active(&self) -> Result<i64> {
        Ok(sqlx::query("SELECT COUNT(*) as count FROM sessions WHERE ended_at IS NULL")
            .fetch_one(&self.pool)
            .await
            .map(|r| r.get::<i64, _>("count"))?)
    }

    async fn total_bandwidth(&self) -> Result<i64> {
        Ok(sqlx::query("SELECT COALESCE(SUM(bytes_up + bytes_down), 0) as total FROM sessions")
            .fetch_one(&self.pool)
            .await
            .map(|r| r.get::<i64, _>("total"))?)
    }
}
