use serde::Deserialize;

#[derive(Clone)]
pub struct ApiClient {
    base: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub api_key: String,
    pub plan_id: String,
    pub active: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CarrierDto {
    pub id: String,
    pub name: String,
    pub country: String,
    pub upstream_addr: String,
    pub online: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionDto {
    pub id: String,
    pub user_id: String,
    pub bytes_up: i64,
    pub bytes_down: i64,
    pub active: bool,
}

impl ApiClient {
    pub fn new(base: String) -> Self {
        Self { base, http: reqwest::Client::new() }
    }

    pub async fn health(&self) -> bool {
        self.http
            .get(format!("{}/api/health", self.base))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub async fn list_users(&self) -> Vec<UserDto> {
        self.get("/api/users").await.unwrap_or_default()
    }

    pub async fn list_carriers(&self) -> Vec<CarrierDto> {
        self.get("/api/carriers").await.unwrap_or_default()
    }

    pub async fn list_sessions(&self) -> Vec<SessionDto> {
        self.get("/api/sessions").await.unwrap_or_default()
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Option<T> {
        self.http
            .get(format!("{}{path}", self.base))
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()
    }
}
