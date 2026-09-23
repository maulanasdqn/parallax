use std::sync::Arc;
use uuid::Uuid;
use zod_rs::prelude::*;

use crate::domain::entity::{Carrier, CreateUserInput, Plan, Session, User};
use crate::domain::port::{
    CarrierRepository, PlanRepository, Result, SessionRepository, UserRepository,
};

#[derive(Clone)]
pub struct AppService {
    users: Arc<dyn UserRepository>,
    plans: Arc<dyn PlanRepository>,
    carriers: Arc<dyn CarrierRepository>,
    sessions: Arc<dyn SessionRepository>,
}

impl AppService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        plans: Arc<dyn PlanRepository>,
        carriers: Arc<dyn CarrierRepository>,
        sessions: Arc<dyn SessionRepository>,
    ) -> Self {
        Self { users, plans, carriers, sessions }
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        self.users.find_all().await
    }

    pub async fn create_user(&self, payload: &serde_json::Value) -> Result<User> {
        let input = CreateUserInput::validate_and_parse(payload)
            .map_err(|e| format!("validation: {e}"))?;
        let plan_id: Uuid = input.plan_id.parse().map_err(|_| "invalid plan_id")?;
        self.users.create(input.email, plan_id).await
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        self.users.delete(id).await
    }

    pub async fn toggle_user(&self, id: Uuid) -> Result<()> {
        self.users.toggle_active(id).await
    }

    pub async fn regenerate_key(&self, id: Uuid) -> Result<Option<String>> {
        self.users.regenerate_api_key(id).await
    }

    pub async fn find_user_by_api_key(&self, api_key: &str) -> Result<Option<User>> {
        self.users.find_by_api_key(api_key).await
    }

    pub async fn list_plans(&self) -> Result<Vec<Plan>> {
        self.plans.find_all().await
    }

    pub async fn list_carriers(&self) -> Result<Vec<Carrier>> {
        self.carriers.find_all().await
    }

    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        self.sessions.find_all().await
    }

    pub async fn user_count(&self) -> Result<i64> {
        let users = self.users.find_all().await?;
        Ok(users.len() as i64)
    }

    pub async fn active_session_count(&self) -> Result<i64> {
        self.sessions.count_active().await
    }

    pub async fn total_bandwidth(&self) -> Result<i64> {
        self.sessions.total_bandwidth().await
    }
}
