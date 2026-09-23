mod chain;
mod http;
mod socks;

pub use chain::Socks5Chain;
pub use http::RoutingHttpHandler;
pub use socks::RoutingSocksHandler;

use async_trait::async_trait;
use parallax_core::{ProxyError, Result, TargetAddr};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpStream;

#[async_trait]
pub trait RouteResolver: Send + Sync {
    fn resolve(&self, carrier: &str) -> Result<SocketAddr>;
    fn verify_password(&self, password: &str) -> Result<()>;
}

#[async_trait]
pub trait UpstreamConnector: Send + Sync {
    async fn connect(
        &self,
        upstream: SocketAddr,
        target: TargetAddr,
        port: u16,
    ) -> Result<TcpStream>;
}

pub fn parse_username(raw: &str) -> Result<(&str, &str)> {
    raw.split_once('-').ok_or(ProxyError::InvalidRequest)
}

pub struct RouteTable {
    routes: HashMap<String, SocketAddr>,
    password: String,
}

impl RouteTable {
    pub fn from_env() -> Option<Self> {
        let raw = std::env::var("ROUTES").ok()?;
        let password = std::env::var("PROXY_PASS").expect("PROXY_PASS required with ROUTES");
        let mut routes = HashMap::new();
        for entry in raw.split(',') {
            let entry = entry.trim();
            let (carrier, addr) = entry
                .split_once('=')
                .expect("invalid route: expected carrier=host:port");
            let sockaddr: SocketAddr = addr.parse().expect("invalid upstream address");
            routes.insert(carrier.to_lowercase(), sockaddr);
        }
        Some(Self { routes, password })
    }

    pub fn carriers(&self) -> Vec<&str> {
        self.routes.keys().map(|k| k.as_str()).collect()
    }
}

#[async_trait]
impl RouteResolver for RouteTable {
    fn resolve(&self, carrier: &str) -> Result<SocketAddr> {
        self.routes
            .get(&carrier.to_lowercase())
            .copied()
            .ok_or(ProxyError::InvalidAddress)
    }

    fn verify_password(&self, password: &str) -> Result<()> {
        if password == self.password {
            Ok(())
        } else {
            Err(ProxyError::AuthFailed)
        }
    }
}
