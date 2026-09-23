use parallax_core::{ConnectionHandler, ProxyConfig};
use parallax_proxy::{
    HttpHandler, NoAuthenticator, PasswordAuthenticator, ProtocolHandler, SocksHandler,
    TokioResolver,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = ProxyConfig {
        listen_addr: std::env::var("PROXY_LISTEN")
            .unwrap_or_else(|_| "0.0.0.0:1080".into())
            .parse()
            .expect("invalid PROXY_LISTEN"),
        credentials: std::env::var("PROXY_USER")
            .ok()
            .zip(std::env::var("PROXY_PASS").ok()),
    };

    match config.credentials {
        Some((user, pass)) => {
            info!(addr = %config.listen_addr, "starting with password auth");
            let socks =
                SocksHandler::new(PasswordAuthenticator::new(user.clone(), pass.clone()), TokioResolver);
            let http = HttpHandler::new(Some((user, pass)), TokioResolver);
            serve(config.listen_addr, ProtocolHandler::new(socks, http)).await;
        }
        None => {
            info!(addr = %config.listen_addr, "starting with no auth");
            let socks = SocksHandler::new(NoAuthenticator, TokioResolver);
            let http = HttpHandler::new(None, TokioResolver);
            serve(config.listen_addr, ProtocolHandler::new(socks, http)).await;
        }
    }
}

async fn serve<H: ConnectionHandler + 'static>(addr: SocketAddr, handler: H) {
    let listener = TcpListener::bind(addr).await.expect("failed to bind");
    let handler = Arc::new(handler);

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, peer)) => {
                        let handler = Arc::clone(&handler);
                        tokio::spawn(async move {
                            if let Err(e) = handler.handle(stream).await {
                                error!(%peer, error = %e, "connection error");
                            }
                        });
                    }
                    Err(e) => error!(error = %e, "accept error"),
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("shutting down");
                break;
            }
        }
    }
}
