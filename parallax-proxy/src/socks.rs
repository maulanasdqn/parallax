use async_trait::async_trait;
use parallax_core::{
    AddressResolver, AuthMethod, Authenticator, Command, ConnectionHandler, Reply, Result,
    TargetAddr, ProxyError, SOCKS_VERSION, write_reply,
};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::TcpStream;

pub struct NoAuthenticator;

#[async_trait]
impl Authenticator for NoAuthenticator {
    fn method(&self) -> AuthMethod {
        AuthMethod::NoAuth
    }

    async fn authenticate(&self, _stream: &mut TcpStream) -> Result<()> {
        Ok(())
    }
}

pub struct PasswordAuthenticator {
    username: String,
    password: String,
}

impl PasswordAuthenticator {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

#[async_trait]
impl Authenticator for PasswordAuthenticator {
    fn method(&self) -> AuthMethod {
        AuthMethod::UsernamePassword
    }

    async fn authenticate(&self, stream: &mut TcpStream) -> Result<()> {
        if stream.read_u8().await? != 0x01 {
            return Err(ProxyError::InvalidVersion);
        }

        let ulen = stream.read_u8().await? as usize;
        let mut ubuf = vec![0u8; ulen];
        stream.read_exact(&mut ubuf).await?;

        let plen = stream.read_u8().await? as usize;
        let mut pbuf = vec![0u8; plen];
        stream.read_exact(&mut pbuf).await?;

        let user = String::from_utf8(ubuf).map_err(|_| ProxyError::AuthFailed)?;
        let pass = String::from_utf8(pbuf).map_err(|_| ProxyError::AuthFailed)?;

        if user == self.username && pass == self.password {
            stream.write_all(&[0x01, 0x00]).await?;
            Ok(())
        } else {
            stream.write_all(&[0x01, 0x01]).await?;
            Err(ProxyError::AuthFailed)
        }
    }
}

pub struct TokioResolver;

#[async_trait]
impl AddressResolver for TokioResolver {
    async fn resolve(&self, addr: TargetAddr, port: u16) -> Result<SocketAddr> {
        match addr {
            TargetAddr::V4(ip) => Ok(SocketAddr::new(ip.into(), port)),
            TargetAddr::V6(ip) => Ok(SocketAddr::new(ip.into(), port)),
            TargetAddr::Domain(domain) => tokio::net::lookup_host(format!("{domain}:{port}"))
                .await?
                .next()
                .ok_or(ProxyError::InvalidAddress),
        }
    }
}

pub struct SocksHandler<A: Authenticator, R: AddressResolver> {
    authenticator: A,
    resolver: R,
}

impl<A: Authenticator, R: AddressResolver> SocksHandler<A, R> {
    pub fn new(authenticator: A, resolver: R) -> Self {
        Self {
            authenticator,
            resolver,
        }
    }

    async fn negotiate(&self, stream: &mut TcpStream) -> Result<()> {
        if stream.read_u8().await? != SOCKS_VERSION {
            return Err(ProxyError::InvalidVersion);
        }

        let n = stream.read_u8().await? as usize;
        let mut methods = vec![0u8; n];
        stream.read_exact(&mut methods).await?;

        let required = self.authenticator.method();
        if methods.contains(&(required as u8)) {
            stream
                .write_all(&[SOCKS_VERSION, required as u8])
                .await?;
            self.authenticator.authenticate(stream).await
        } else {
            stream
                .write_all(&[SOCKS_VERSION, AuthMethod::NoAcceptable as u8])
                .await?;
            Err(ProxyError::NoAcceptableMethod)
        }
    }

    async fn relay(&self, stream: &mut TcpStream) -> Result<()> {
        if stream.read_u8().await? != SOCKS_VERSION {
            return Err(ProxyError::InvalidVersion);
        }

        let cmd = Command::try_from(stream.read_u8().await?)?;
        let _reserved = stream.read_u8().await?;
        let addr = TargetAddr::read_from(stream).await?;
        let mut port_buf = [0u8; 2];
        stream.read_exact(&mut port_buf).await?;
        let port = u16::from_be_bytes(port_buf);
        let fallback: SocketAddr = ([0, 0, 0, 0], 0).into();

        match cmd {
            Command::Connect => self.connect(stream, addr, port).await,
            _ => {
                let _ = write_reply(stream, Reply::CommandNotSupported, fallback).await;
                Err(ProxyError::UnsupportedCommand)
            }
        }
    }

    async fn connect(
        &self,
        stream: &mut TcpStream,
        addr: TargetAddr,
        port: u16,
    ) -> Result<()> {
        let fallback: SocketAddr = ([0, 0, 0, 0], 0).into();

        let target_addr = match self.resolver.resolve(addr, port).await {
            Ok(a) => a,
            Err(e) => {
                let _ = write_reply(stream, Reply::GeneralFailure, fallback).await;
                return Err(e);
            }
        };

        match TcpStream::connect(target_addr).await {
            Ok(mut target) => {
                let bind_addr = target.local_addr().unwrap_or(fallback);
                write_reply(stream, Reply::Succeeded, bind_addr).await?;
                copy_bidirectional(stream, &mut target).await?;
                Ok(())
            }
            Err(e) => {
                let _ = write_reply(stream, Reply::ConnectionRefused, fallback).await;
                Err(ProxyError::Io(e))
            }
        }
    }
}

#[async_trait]
impl<A: Authenticator + 'static, R: AddressResolver + 'static> ConnectionHandler
    for SocksHandler<A, R>
{
    async fn handle(&self, mut stream: TcpStream) -> Result<()> {
        self.negotiate(&mut stream).await?;
        self.relay(&mut stream).await
    }
}
