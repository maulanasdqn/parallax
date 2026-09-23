use async_trait::async_trait;
use parallax_core::{ConnectionHandler, Connector, ProxyError, Result, TargetAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::TcpStream;

pub struct HttpHandler<C: Connector> {
    credentials: Option<(String, String)>,
    connector: C,
}

impl<C: Connector> HttpHandler<C> {
    pub fn new(credentials: Option<(String, String)>, connector: C) -> Self {
        Self { credentials, connector }
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

    fn verify_auth(&self, headers: &str) -> Result<()> {
        let (user, pass) = match &self.credentials {
            Some(c) => c,
            None => return Ok(()),
        };
        let expected = base64_encode(format!("{user}:{pass}").as_bytes());
        for line in headers.lines() {
            if line.len() > 27
                && line[..27].eq_ignore_ascii_case("proxy-authorization: basic ")
                && line[27..].trim() == expected
            {
                return Ok(());
            }
        }
        Err(ProxyError::AuthFailed)
    }

    async fn tunnel(&self, stream: &mut TcpStream, host: &str, port: u16) -> Result<()> {
        let addr = TargetAddr::Domain(host.to_string());
        let mut target = self.connector.connect(addr, port).await?;
        stream
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await?;
        copy_bidirectional(stream, &mut target).await?;
        Ok(())
    }

    async fn forward(
        &self,
        stream: &mut TcpStream,
        method: &str,
        path: &str,
        host: &str,
        port: u16,
        raw_headers: &str,
    ) -> Result<()> {
        let addr = TargetAddr::Domain(host.to_string());
        let mut target = self.connector.connect(addr, port).await?;
        let mut request = format!("{method} {path} HTTP/1.1\r\n");
        for line in raw_headers.lines().skip(1) {
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
        target.write_all(request.as_bytes()).await?;
        copy_bidirectional(stream, &mut target).await?;
        Ok(())
    }

    async fn send_error(stream: &mut TcpStream, status: u16, reason: &str) {
        let resp = format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        let _ = stream.write_all(resp.as_bytes()).await;
    }
}

#[async_trait]
impl<C: Connector + 'static> ConnectionHandler for HttpHandler<C> {
    async fn handle(&self, mut stream: TcpStream) -> Result<()> {
        let headers = Self::read_headers(&mut stream).await?;
        let request_line = headers.lines().next().ok_or(ProxyError::InvalidRequest)?;
        let parts: Vec<&str> = request_line.split_whitespace().collect();

        if parts.len() < 3 {
            Self::send_error(&mut stream, 400, "Bad Request").await;
            return Err(ProxyError::InvalidRequest);
        }

        let (method, url) = (parts[0], parts[1]);

        if let Err(e) = self.verify_auth(&headers) {
            Self::send_error(&mut stream, 407, "Proxy Authentication Required").await;
            return Err(e);
        }

        if method.eq_ignore_ascii_case("CONNECT") {
            let (host, port) = parse_authority(url)?;
            self.tunnel(&mut stream, &host, port).await
        } else {
            let (host, port, path) = parse_url(url)?;
            self.forward(&mut stream, method, &path, &host, port, &headers)
                .await
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

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(4 * (data.len() + 2) / 3);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
