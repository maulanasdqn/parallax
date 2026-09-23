use async_trait::async_trait;
use parallax_core::{
    Command, ConnectionHandler, ProxyError, Reply, Result, TargetAddr, SOCKS_VERSION, write_reply,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::TcpStream;
use tracing::info;

use crate::{RouteResolver, UpstreamConnector, parse_username};

pub struct RoutingSocksHandler<R: RouteResolver, U: UpstreamConnector> {
    resolver: Arc<R>,
    connector: Arc<U>,
}

impl<R: RouteResolver, U: UpstreamConnector> RoutingSocksHandler<R, U> {
    pub fn new(resolver: Arc<R>, connector: Arc<U>) -> Self {
        Self { resolver, connector }
    }

    async fn authenticate(&self, stream: &mut TcpStream) -> Result<String> {
        if stream.read_u8().await? != SOCKS_VERSION {
            return Err(ProxyError::InvalidVersion);
        }
        let n = stream.read_u8().await? as usize;
        let mut methods = vec![0u8; n];
        stream.read_exact(&mut methods).await?;

        if !methods.contains(&0x02) {
            stream.write_all(&[SOCKS_VERSION, 0xFF]).await?;
            return Err(ProxyError::NoAcceptableMethod);
        }
        stream.write_all(&[SOCKS_VERSION, 0x02]).await?;

        if stream.read_u8().await? != 0x01 {
            return Err(ProxyError::InvalidVersion);
        }
        let ulen = stream.read_u8().await? as usize;
        let mut ubuf = vec![0u8; ulen];
        stream.read_exact(&mut ubuf).await?;
        let plen = stream.read_u8().await? as usize;
        let mut pbuf = vec![0u8; plen];
        stream.read_exact(&mut pbuf).await?;

        let username = String::from_utf8(ubuf).map_err(|_| ProxyError::AuthFailed)?;
        let password = String::from_utf8(pbuf).map_err(|_| ProxyError::AuthFailed)?;

        let (carrier, _) = parse_username(&username)?;
        self.resolver.resolve(carrier)?;
        self.resolver.verify_password(&password)?;

        stream.write_all(&[0x01, 0x00]).await?;
        Ok(username)
    }
}

#[async_trait]
impl<R: RouteResolver + 'static, U: UpstreamConnector + 'static> ConnectionHandler
    for RoutingSocksHandler<R, U>
{
    async fn handle(&self, mut stream: TcpStream) -> Result<()> {
        let username = match self.authenticate(&mut stream).await {
            Ok(u) => u,
            Err(e) => {
                let _ = stream.write_all(&[0x01, 0x01]).await;
                return Err(e);
            }
        };

        let (carrier, _) = parse_username(&username)?;
        let upstream = self.resolver.resolve(carrier)?;

        if stream.read_u8().await? != SOCKS_VERSION {
            return Err(ProxyError::InvalidVersion);
        }
        let cmd = Command::try_from(stream.read_u8().await?)?;
        let _rsv = stream.read_u8().await?;
        let addr = TargetAddr::read_from(&mut stream).await?;
        let mut port_buf = [0u8; 2];
        stream.read_exact(&mut port_buf).await?;
        let port = u16::from_be_bytes(port_buf);
        let fallback: SocketAddr = ([0, 0, 0, 0], 0).into();

        match cmd {
            Command::Connect => {
                info!(%carrier, %upstream, "chaining socks5");
                match self.connector.connect(upstream, addr, port).await {
                    Ok(mut target) => {
                        let bind = target.local_addr().unwrap_or(fallback);
                        write_reply(&mut stream, Reply::Succeeded, bind).await?;
                        copy_bidirectional(&mut stream, &mut target).await?;
                        Ok(())
                    }
                    Err(e) => {
                        let _ =
                            write_reply(&mut stream, Reply::ConnectionRefused, fallback).await;
                        Err(e)
                    }
                }
            }
            _ => {
                let _ = write_reply(&mut stream, Reply::CommandNotSupported, fallback).await;
                Err(ProxyError::UnsupportedCommand)
            }
        }
    }
}
