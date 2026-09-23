use async_trait::async_trait;
use parallax_core::{ConnectionHandler, ProxyError, Result, TargetAddr};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::TcpStream;
use tracing::info;

use crate::{RouteResolver, UpstreamConnector, parse_username};

pub struct RoutingHttpHandler<R: RouteResolver, U: UpstreamConnector> {
    resolver: Arc<R>,
    connector: Arc<U>,
}

impl<R: RouteResolver, U: UpstreamConnector> RoutingHttpHandler<R, U> {
    pub fn new(resolver: Arc<R>, connector: Arc<U>) -> Self {
        Self { resolver, connector }
    }

    async fn read_headers(stream: &mut TcpStream) -> Result<String> {
        let mut buf = Vec::with_capacity(4096);
        loop {
            let byte = stream.read_u8().await?;
            buf.push(byte);
            if buf.len() >= 4 && buf[buf.len() - 4..] == *b"\r\n\r\n" {
                break;
            }
            if buf.len() > 8192 {
                return Err(ProxyError::InvalidRequest);
            }
        }
        String::from_utf8(buf).map_err(|_| ProxyError::InvalidRequest)
    }

    fn extract_credentials(headers: &str) -> Result<(String, String)> {
        for line in headers.lines() {
            if line.len() > 27
                && line[..27].eq_ignore_ascii_case("proxy-authorization: basic ")
            {
                let encoded = line[27..].trim();
                let decoded = base64_decode(encoded).ok_or(ProxyError::AuthFailed)?;
                let text = String::from_utf8(decoded).map_err(|_| ProxyError::AuthFailed)?;
                let (user, pass) = text.split_once(':').ok_or(ProxyError::AuthFailed)?;
                return Ok((user.to_string(), pass.to_string()));
            }
        }
        Err(ProxyError::AuthFailed)
    }

    async fn send_error(stream: &mut TcpStream, status: u16, reason: &str) {
        let resp = format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        let _ = stream.write_all(resp.as_bytes()).await;
    }

    fn strip_proxy_headers(raw: &str, method: &str, path: &str) -> String {
        let mut request = format!("{method} {path} HTTP/1.1\r\n");
        for line in raw.lines().skip(1) {
            if line.is_empty() {
                break;
            }
            let lower = line.to_lowercase();
            if lower.starts_with("proxy-authorization:")
                || lower.starts_with("proxy-connection:")
                || lower.starts_with("connection:")
            {
                continue;
            }
            request.push_str(line);
            request.push_str("\r\n");
        }
        request.push_str("Connection: close\r\n\r\n");
        request
    }
}

#[async_trait]
impl<R: RouteResolver + 'static, U: UpstreamConnector + 'static> ConnectionHandler
    for RoutingHttpHandler<R, U>
{
    async fn handle(&self, mut stream: TcpStream) -> Result<()> {
        let headers = Self::read_headers(&mut stream).await?;
        let request_line = headers.lines().next().ok_or(ProxyError::InvalidRequest)?;
        let parts: Vec<&str> = request_line.split_whitespace().collect();

        if parts.len() < 3 {
            Self::send_error(&mut stream, 400, "Bad Request").await;
            return Err(ProxyError::InvalidRequest);
        }
        let (method, url) = (parts[0], parts[1]);

        let (username, password) = match Self::extract_credentials(&headers) {
            Ok(c) => c,
            Err(e) => {
                Self::send_error(&mut stream, 407, "Proxy Authentication Required").await;
                return Err(e);
            }
        };

        let (carrier, _) = parse_username(&username)?;
        self.resolver.verify_password(&password)?;
        let upstream = self.resolver.resolve(carrier)?;

        if method.eq_ignore_ascii_case("CONNECT") {
            let (host, port) = parse_authority(url)?;
            info!(%carrier, %upstream, "chaining http connect");
            let mut chained = self.connector.connect(upstream, TargetAddr::Domain(host), port).await?;
            stream
                .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                .await?;
            copy_bidirectional(&mut stream, &mut chained).await?;
            Ok(())
        } else {
            let (host, port, path) = parse_url(url)?;
            info!(%carrier, %upstream, "chaining http forward");
            let mut chained = self
                .connector
                .connect(upstream, TargetAddr::Domain(host), port)
                .await?;
            let request = Self::strip_proxy_headers(&headers, method, &path);
            chained.write_all(request.as_bytes()).await?;
            copy_bidirectional(&mut stream, &mut chained).await?;
            Ok(())
        }
    }
}

fn parse_authority(authority: &str) -> Result<(String, u16)> {
    match authority.rsplit_once(':') {
        Some((host, port)) => {
            let port = port.parse().map_err(|_| ProxyError::InvalidAddress)?;
            Ok((host.to_string(), port))
        }
        None => Ok((authority.to_string(), 443)),
    }
}

fn parse_url(url: &str) -> Result<(String, u16, String)> {
    let url = url.strip_prefix("http://").unwrap_or(url);
    let (authority, path) = match url.find('/') {
        Some(i) => (&url[..i], &url[i..]),
        None => (url, "/"),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h, p.parse().map_err(|_| ProxyError::InvalidAddress)?),
        None => (authority, 80u16),
    };
    Ok((host.to_string(), port, path.to_string()))
}

fn base64_decode(data: &str) -> Option<Vec<u8>> {
    const TABLE: [i8; 128] = {
        let mut t = [-1i8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 {
            t[chars[i] as usize] = i as i8;
            i += 1;
        }
        t
    };
    let data = data.trim_end_matches('=');
    let mut out = Vec::with_capacity(data.len() * 3 / 4);
    for chunk in data.as_bytes().chunks(4) {
        let mut n = 0u32;
        for (i, &b) in chunk.iter().enumerate() {
            if b > 127 {
                return None;
            }
            let v = TABLE[b as usize];
            if v < 0 {
                return None;
            }
            n |= (v as u32) << (18 - 6 * i);
        }
        out.push((n >> 16) as u8);
        if chunk.len() > 2 {
            out.push((n >> 8) as u8);
        }
        if chunk.len() > 3 {
            out.push(n as u8);
        }
    }
    Some(out)
}
