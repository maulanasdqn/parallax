# Parallax

A lightweight, multi-protocol proxy server written in Rust. Drop-in replacement for dante-server with both SOCKS5 and HTTP proxy support on a single port.

## Features

- **SOCKS5** (RFC 1928) with TCP CONNECT
- **HTTP CONNECT** tunneling (HTTPS passthrough)
- **HTTP forward proxy** (plain HTTP requests)
- **Auto-detection** -- protocol is identified per-connection by peeking the first byte, so SOCKS5 and HTTP clients share one port
- **Authentication** -- no-auth or username/password (SOCKS5 via RFC 1929, HTTP via `Proxy-Authorization: Basic`)
- **Server-side DNS** -- domain names are resolved by the proxy (SOCKS5h behavior)
- **Graceful shutdown** on SIGINT/SIGTERM
- **Structured logging** via `tracing`

## Architecture

Multi-workspace Rust project with trait-based clean architecture.

```
parallax/
  Cargo.toml              workspace root
  parallax-core/          protocol types, error types, trait definitions
  parallax-proxy/         trait implementations, protocol handlers
    src/lib.rs            ProtocolHandler (auto-detect + delegate)
    src/socks.rs          SocksHandler, authenticators, resolver
    src/http.rs           HttpHandler (CONNECT + forward)
  parallax-server/        binary entry point, config, accept loop
```

### Traits

| Trait | Defined in | Purpose |
|---|---|---|
| `Authenticator` | `parallax-core` | SOCKS5 auth method negotiation |
| `AddressResolver` | `parallax-core` | Resolve `TargetAddr` (IPv4/IPv6/Domain) to `SocketAddr` |
| `ConnectionHandler` | `parallax-core` | Handle a full TCP connection lifecycle |

### Concrete implementations

| Type | Crate | Implements |
|---|---|---|
| `NoAuthenticator` | `parallax-proxy` | `Authenticator` -- method 0x00, no sub-negotiation |
| `PasswordAuthenticator` | `parallax-proxy` | `Authenticator` -- method 0x02, RFC 1929 |
| `TokioResolver` | `parallax-proxy` | `AddressResolver` -- tokio DNS lookup |
| `SocksHandler<A, R>` | `parallax-proxy` | `ConnectionHandler` -- full SOCKS5 flow |
| `HttpHandler<R>` | `parallax-proxy` | `ConnectionHandler` -- HTTP CONNECT + forward |
| `ProtocolHandler<S, H>` | `parallax-proxy` | `ConnectionHandler` -- peeks first byte, delegates to S or H |

### Dependencies

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime, TCP, `copy_bidirectional` |
| `async-trait` | Async methods in traits |
| `tracing` | Structured logging |
| `tracing-subscriber` | Log output formatting |

No HTTP framework. SOCKS5 is parsed from raw TCP. HTTP headers are parsed manually. Base64 encoding for HTTP Basic auth is implemented inline.

## Build

```sh
cargo build --release
```

Binary is at `target/release/parallax-server`.

## Usage

```sh
# Start with defaults (0.0.0.0:1080, no auth)
cargo run -p parallax-server

# Custom listen address
PROXY_LISTEN=127.0.0.1:9090 cargo run -p parallax-server

# With username/password auth
PROXY_USER=admin PROXY_PASS=secret cargo run -p parallax-server
```

## Configuration

All configuration is via environment variables.

| Variable | Default | Description |
|---|---|---|
| `PROXY_LISTEN` | `0.0.0.0:1080` | Address and port to bind |
| `PROXY_USER` | *(unset)* | Username for auth (both must be set to enable) |
| `PROXY_PASS` | *(unset)* | Password for auth (both must be set to enable) |

When `PROXY_USER` and `PROXY_PASS` are both set, authentication is required for all protocols:
- SOCKS5 clients must negotiate username/password auth (RFC 1929)
- HTTP clients must send `Proxy-Authorization: Basic <base64(user:pass)>` header

When either is unset, the proxy runs in no-auth mode.

## Testing

### SOCKS5

```sh
# HTTP through SOCKS5 (proxy resolves DNS)
curl -x socks5h://localhost:1080 http://httpbin.org/ip

# HTTPS through SOCKS5
curl -x socks5h://localhost:1080 https://httpbin.org/ip

# With auth
curl -x socks5h://admin:secret@localhost:1080 http://httpbin.org/ip
```

### HTTP proxy

```sh
# HTTPS via CONNECT tunnel
curl -x http://localhost:1080 https://httpbin.org/ip

# Plain HTTP forwarding
curl -x http://localhost:1080 http://httpbin.org/ip

# With auth
curl -x http://admin:secret@localhost:1080 https://httpbin.org/ip
```

### Browser configuration

Set your browser's proxy to `localhost:1080` for both SOCKS and HTTP proxy fields. Firefox supports per-protocol proxy configuration under Settings > Network Settings > Manual proxy configuration.

### systemd

```ini
[Unit]
Description=Parallax Proxy Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/parallax-server
Environment=PROXY_LISTEN=0.0.0.0:1080
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

## Protocol details

### SOCKS5 (RFC 1928)

```
Client -> Server:  [0x05, n_methods, methods...]
Server -> Client:  [0x05, chosen_method]
  (if username/password -- RFC 1929):
  Client -> Server:  [0x01, ulen, username, plen, password]
  Server -> Client:  [0x01, status]
Client -> Server:  [0x05, 0x01(CONNECT), 0x00, addr_type, dst_addr, dst_port]
Server -> Client:  [0x05, reply, 0x00, addr_type, bind_addr, bind_port]
                    <bidirectional TCP relay>
```

Supported address types: IPv4 (0x01), Domain (0x03), IPv6 (0x04).

Only the CONNECT command (0x01) is implemented. BIND and UDP ASSOCIATE return `CommandNotSupported`.

### HTTP proxy

**CONNECT** (HTTPS tunneling): The proxy reads the `CONNECT host:port` request, connects to the target, responds with `200 Connection Established`, then relays bytes bidirectionally. TLS passes through opaquely.

**Forward** (plain HTTP): The proxy reads the full request with absolute URL (`GET http://host/path`), rewrites it to a relative path, strips `Proxy-Authorization` and `Proxy-Connection` headers, adds `Connection: close`, and forwards to the target. Response is relayed back to the client.

### Auto-detection

The first byte of each connection determines the protocol:
- `0x05` -- SOCKS5 (version byte)
- Anything else -- HTTP (ASCII method character)

This happens per-connection, so SOCKS5 and HTTP clients can connect to the same port simultaneously.

## License

MIT
