use parallax_core::{ConnectionHandler, ProxyConfig};
use parallax_proxy::{
    DirectConnector, HttpHandler, NoAuthenticator, PasswordAuthenticator, ProtocolHandler,
    SocksHandler, TokioResolver,
};
use parallax_router::{RouteTable, RoutingHttpHandler, RoutingSocksHandler, Socks5Chain};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let listen_addr: SocketAddr = std::env::var("PROXY_LISTEN")
        .unwrap_or_else(|_| "0.0.0.0:1080".into())
        .parse()
        .expect("invalid PROXY_LISTEN");

    if let Some(routes) = RouteTable::from_env() {
        let carriers = routes.carriers().join(", ");
        info!(addr = %listen_addr, %carriers, "starting in routing mode");
        let resolver = Arc::new(routes);
        let connector = Arc::new(Socks5Chain);
        let socks = RoutingSocksHandler::new(Arc::clone(&resolver), Arc::clone(&connector));
        let http = RoutingHttpHandler::new(resolver, connector);
        serve(listen_addr, ProtocolHandler::new(socks, http)).await;
    } else {
        let config = ProxyConfig {
            listen_addr,
            credentials: std::env::var("PROXY_USER")
                .ok()
                .zip(std::env::var("PROXY_PASS").ok()),
        };
        let connector = DirectConnector::new(TokioResolver);
        match config.credentials {
            Some((user, pass)) => {
                info!(addr = %listen_addr, "starting with password auth");
                let socks = SocksHandler::new(PasswordAuthenticator::new(user.clone(), pass.clone()), connector);
                let http = HttpHandler::new(Some((user, pass)), DirectConnector::new(TokioResolver));
                serve(listen_addr, ProtocolHandler::new(socks, http)).await;
            }
            None => {
                info!(addr = %listen_addr, "starting with no auth");
                let socks = SocksHandler::new(NoAuthenticator, connector);
                let http = HttpHandler::new(None, DirectConnector::new(TokioResolver));
                serve(listen_addr, ProtocolHandler::new(socks, http)).await;
            }
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
