use async_trait::async_trait;
use std::fmt;
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;

pub const SOCKS_VERSION: u8 = 0x05;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum AuthMethod {
    NoAuth = 0x00,
    UsernamePassword = 0x02,
    NoAcceptable = 0xFF,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Command {
    Connect = 0x01,
    Bind = 0x02,
    UdpAssociate = 0x03,
}

impl TryFrom<u8> for Command {
    type Error = ProxyError;

    fn try_from(v: u8) -> Result<Self> {
        match v {
            0x01 => Ok(Self::Connect),
            0x02 => Ok(Self::Bind),
            0x03 => Ok(Self::UdpAssociate),
            _ => Err(ProxyError::UnsupportedCommand),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Reply {
    Succeeded = 0x00,
    GeneralFailure = 0x01,
    ConnectionRefused = 0x05,
    CommandNotSupported = 0x07,
}

pub enum TargetAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
    Domain(String),
}

impl TargetAddr {
    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        match reader.read_u8().await? {
            0x01 => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf).await?;
                Ok(Self::V4(Ipv4Addr::from(buf)))
            }
            0x03 => {
                let len = reader.read_u8().await? as usize;
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf).await?;
                String::from_utf8(buf)
                    .map(Self::Domain)
                    .map_err(|_| ProxyError::InvalidAddress)
            }
            0x04 => {
                let mut buf = [0u8; 16];
                reader.read_exact(&mut buf).await?;
                Ok(Self::V6(Ipv6Addr::from(buf)))
            }
            _ => Err(ProxyError::UnsupportedAddrType),
        }
    }

    pub async fn write_to<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        match self {
            Self::V4(ip) => {
                writer.write_u8(0x01).await?;
                writer.write_all(&ip.octets()).await?;
            }
            Self::Domain(d) => {
                writer.write_u8(0x03).await?;
                writer.write_u8(d.len() as u8).await?;
                writer.write_all(d.as_bytes()).await?;
            }
            Self::V6(ip) => {
                writer.write_u8(0x04).await?;
                writer.write_all(&ip.octets()).await?;
            }
        }
        Ok(())
    }
}

pub async fn write_reply<W: AsyncWrite + Unpin>(
    writer: &mut W,
    reply: Reply,
    bind_addr: SocketAddr,
) -> Result<()> {
    let mut buf = vec![SOCKS_VERSION, reply as u8, 0x00];
    match bind_addr {
        SocketAddr::V4(a) => {
            buf.push(0x01);
            buf.extend_from_slice(&a.ip().octets());
        }
        SocketAddr::V6(a) => {
            buf.push(0x04);
            buf.extend_from_slice(&a.ip().octets());
        }
    }
    buf.extend_from_slice(&bind_addr.port().to_be_bytes());
    writer.write_all(&buf).await?;
    Ok(())
}

#[derive(Debug)]
pub enum ProxyError {
    Io(io::Error),
    InvalidVersion,
    InvalidRequest,
    NoAcceptableMethod,
    UnsupportedCommand,
    UnsupportedAddrType,
    InvalidAddress,
    AuthFailed,
    UpstreamFailed,
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io: {e}"),
            Self::InvalidVersion => write!(f, "invalid protocol version"),
            Self::InvalidRequest => write!(f, "invalid request"),
            Self::NoAcceptableMethod => write!(f, "no acceptable auth method"),
            Self::UnsupportedCommand => write!(f, "unsupported command"),
            Self::UnsupportedAddrType => write!(f, "unsupported address type"),
            Self::InvalidAddress => write!(f, "invalid address"),
            Self::AuthFailed => write!(f, "authentication failed"),
            Self::UpstreamFailed => write!(f, "upstream proxy failed"),
        }
    }
}

impl From<io::Error> for ProxyError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, ProxyError>;

pub struct ProxyConfig {
    pub listen_addr: SocketAddr,
    pub credentials: Option<(String, String)>,
}

#[async_trait]
pub trait Authenticator: Send + Sync {
    fn method(&self) -> AuthMethod;
    async fn authenticate(&self, stream: &mut TcpStream) -> Result<()>;
}

#[async_trait]
pub trait AddressResolver: Send + Sync {
    async fn resolve(&self, addr: TargetAddr, port: u16) -> Result<SocketAddr>;
}

#[async_trait]
pub trait Connector: Send + Sync {
    async fn connect(&self, addr: TargetAddr, port: u16) -> Result<TcpStream>;
}

#[async_trait]
pub trait ConnectionHandler: Send + Sync {
    async fn handle(&self, stream: TcpStream) -> Result<()>;
}
