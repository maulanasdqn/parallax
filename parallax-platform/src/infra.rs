use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::entity::{Carrier, Plan, Session, UsageRecord, User};

pub struct MemoryStore {
    pub users: Mutex<HashMap<Uuid, User>>,
    pub plans: Mutex<HashMap<Uuid, Plan>>,
    pub carriers: Mutex<HashMap<Uuid, Carrier>>,
    pub sessions: Mutex<HashMap<Uuid, Session>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        let store = Self {
            users: Mutex::new(HashMap::new()),
            plans: Mutex::new(HashMap::new()),
            carriers: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
        };
        store.seed();
        store
    }

    fn seed(&self) {
        let starter = Plan {
            id: Uuid::new_v4(),
            name: "Starter".into(),
            bandwidth_limit_bytes: 2 * 1024 * 1024 * 1024_i64,
            concurrent_limit: 2,
            price_cents: 5000,
        };
        let pro = Plan {
            id: Uuid::new_v4(),
            name: "Pro".into(),
            bandwidth_limit_bytes: 10 * 1024 * 1024 * 1024_i64,
            concurrent_limit: 5,
            price_cents: 15000,
        };
        let business = Plan {
            id: Uuid::new_v4(),
            name: "Business".into(),
            bandwidth_limit_bytes: 50 * 1024 * 1024 * 1024_i64,
            concurrent_limit: 20,
            price_cents: 50000,
        };

        let mut plans = self.plans.lock().unwrap();
        plans.insert(starter.id, starter.clone());
        plans.insert(pro.id, pro.clone());
        plans.insert(business.id, business.clone());
        drop(plans);

        let users_data = vec![
            ("admin@parallax.id", &business),
            ("john@proxy.id", &pro),
            ("sarah@proxy.id", &starter),
        ];

        let mut users = self.users.lock().unwrap();
        for (email, plan) in users_data {
            let user = User {
                id: Uuid::new_v4(),
                email: email.into(),
                plan_id: plan.id,
                api_key: format!("px_{}", Uuid::new_v4().simple()),
                active: true,
                created_at: Utc::now(),
            };
            users.insert(user.id, user);
        }
    }

    pub fn list_users(&self) -> Vec<User> {
        self.users.lock().unwrap().values().cloned().collect()
    }

    pub fn create_user(&self, email: String, plan_id: Uuid) -> User {
        let user = User {
            id: Uuid::new_v4(),
            email,
            plan_id,
            api_key: format!("px_{}", Uuid::new_v4().simple()),
            active: true,
            created_at: Utc::now(),
        };
        self.users.lock().unwrap().insert(user.id, user.clone());
        user
    }

    pub fn delete_user(&self, id: Uuid) {
        self.users.lock().unwrap().remove(&id);
    }

    pub fn toggle_user(&self, id: Uuid) {
        if let Some(user) = self.users.lock().unwrap().get_mut(&id) {
            user.active = !user.active;
        }
    }

    pub fn regenerate_key(&self, id: Uuid) -> Option<String> {
        let mut users = self.users.lock().unwrap();
        users.get_mut(&id).map(|u| {
            u.api_key = format!("px_{}", Uuid::new_v4().simple());
            u.api_key.clone()
        })
    }

    pub fn list_plans(&self) -> Vec<Plan> {
        self.plans.lock().unwrap().values().cloned().collect()
    }

    pub fn find_plan(&self, id: Uuid) -> Option<Plan> {
        self.plans.lock().unwrap().get(&id).cloned()
    }

    pub fn list_carriers(&self) -> Vec<Carrier> {
        self.carriers.lock().unwrap().values().cloned().collect()
    }

    pub fn list_sessions(&self) -> Vec<Session> {
        self.sessions.lock().unwrap().values().cloned().collect()
    }

    pub fn user_count(&self) -> usize {
        self.users.lock().unwrap().len()
    }

    pub fn active_session_count(&self) -> usize {
        self.sessions
            .lock()
            .unwrap()
            .values()
            .filter(|s| s.ended_at.is_none())
            .count()
    }

    pub fn total_bandwidth(&self) -> i64 {
        self.sessions
            .lock()
            .unwrap()
            .values()
            .map(|s| s.bytes_up + s.bytes_down)
            .sum()
    }

    pub fn user_usage(&self, user_id: Uuid) -> UsageRecord {
        let sessions = self.sessions.lock().unwrap();
        let user_sessions: Vec<_> = sessions.values().filter(|s| s.user_id == user_id).collect();
        UsageRecord {
            user_id,
            total_bytes: user_sessions.iter().map(|s| (s.bytes_up + s.bytes_down) as u64).sum(),
            session_count: user_sessions.len() as u64,
        }
    }
}
