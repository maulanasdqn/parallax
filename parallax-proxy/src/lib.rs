mod http;
mod socks;

pub use http::HttpHandler;
pub use socks::{NoAuthenticator, PasswordAuthenticator, SocksHandler, TokioResolver};

use async_trait::async_trait;
use parallax_core::{ConnectionHandler, ProxyError, Result, SOCKS_VERSION};
use tokio::net::TcpStream;

pub struct ProtocolHandler<S: ConnectionHandler, H: ConnectionHandler> {
    socks: S,
    http: H,
}

impl<S: ConnectionHandler, H: ConnectionHandler> ProtocolHandler<S, H> {
    pub fn new(socks: S, http: H) -> Self {
        Self { socks, http }
    }
}

#[async_trait]
impl<S: ConnectionHandler + 'static, H: ConnectionHandler + 'static> ConnectionHandler
    for ProtocolHandler<S, H>
{
    async fn handle(&self, stream: TcpStream) -> Result<()> {
        let mut peek = [0u8; 1];
        stream.peek(&mut peek).await.map_err(ProxyError::Io)?;
        if peek[0] == SOCKS_VERSION {
            self.socks.handle(stream).await
        } else {
            self.http.handle(stream).await
        }
    }
}
