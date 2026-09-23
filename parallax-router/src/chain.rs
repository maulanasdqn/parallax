use async_trait::async_trait;
use parallax_core::{ProxyError, Result, TargetAddr};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::UpstreamConnector;

pub struct Socks5Chain;

#[async_trait]
impl UpstreamConnector for Socks5Chain {
    async fn connect(
        &self,
        upstream: SocketAddr,
        target: TargetAddr,
        port: u16,
    ) -> Result<TcpStream> {
        let mut stream = TcpStream::connect(upstream)
            .await
            .map_err(ProxyError::Io)?;

        stream.write_all(&[0x05, 0x01, 0x00]).await?;
        let mut greeting = [0u8; 2];
        stream.read_exact(&mut greeting).await?;
        if greeting[0] != 0x05 || greeting[1] != 0x00 {
            return Err(ProxyError::UpstreamFailed);
        }

        stream.write_all(&[0x05, 0x01, 0x00]).await?;
        target.write_to(&mut stream).await?;
        stream.write_all(&port.to_be_bytes()).await?;

        let mut reply = [0u8; 4];
        stream.read_exact(&mut reply).await?;
        if reply[1] != 0x00 {
            return Err(ProxyError::UpstreamFailed);
        }

        skip_bind_addr(&mut stream, reply[3]).await?;
        Ok(stream)
    }
}

async fn skip_bind_addr(stream: &mut TcpStream, addr_type: u8) -> Result<()> {
    match addr_type {
        0x01 => {
            let mut buf = [0u8; 6];
            stream.read_exact(&mut buf).await?;
        }
        0x03 => {
            let len = stream.read_u8().await? as usize;
            let mut buf = vec![0u8; len + 2];
            stream.read_exact(&mut buf).await?;
        }
        0x04 => {
            let mut buf = [0u8; 18];
            stream.read_exact(&mut buf).await?;
        }
        _ => return Err(ProxyError::UnsupportedAddrType),
    }
    Ok(())
}
