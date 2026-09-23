# Parallax

Indonesian mobile proxy platform with multi-protocol proxy server, upstream carrier routing, REST API, and SaaS dashboard. Built in Rust with trait-based clean architecture.

## Workspaces

| Workspace | Binary | Purpose |
|---|---|---|
| `parallax-core` | -- | Protocol types, error types, trait definitions |
| `parallax-proxy` | -- | Direct proxy handlers (SOCKS5, HTTP, auto-detect) |
| `parallax-router` | -- | Upstream chaining with carrier-based routing |
| `parallax-server` | `parallax-server` | Proxy server entry point (direct or routing mode) |
| `parallax-api` | `parallax-api` | REST API with Axum (hexagonal architecture) |
| `parallax-platform` | `parallax-platform` | SaaS dashboard with TopCoat + Tailwind |

## Architecture

```
                    parallax-platform (TopCoat dashboard)
                            |
                    parallax-api (Axum REST API)
                            |
                        PostgreSQL
                            
Clients --> parallax-server (proxy) --> Direct / Upstream chain
                |               |
          parallax-proxy   parallax-router
                |               |
              parallax-core (traits)
```

### Proxy traits (parallax-core)

| Trait | Purpose |
|---|---|
| `Authenticator` | SOCKS5 auth method negotiation |
| `AddressResolver` | Resolve `TargetAddr` to `SocketAddr` |
| `Connector` | Establish connection to target (direct or chained) |
| `ConnectionHandler` | Handle a full TCP connection lifecycle |

### Proxy implementations (parallax-proxy)

| Type | Implements |
|---|---|
| `NoAuthenticator` | `Authenticator` -- no auth |
| `PasswordAuthenticator` | `Authenticator` -- RFC 1929 username/password |
| `TokioResolver` | `AddressResolver` -- tokio DNS |
| `DirectConnector<R>` | `Connector` -- resolve + TCP connect |
| `SocksHandler<A, C>` | `ConnectionHandler` -- SOCKS5 flow |
| `HttpHandler<C>` | `ConnectionHandler` -- HTTP CONNECT + forward |
| `ProtocolHandler<S, H>` | `ConnectionHandler` -- auto-detect SOCKS5/HTTP |

### Routing traits (parallax-router)

| Trait | Purpose |
|---|---|
| `RouteResolver` | Map carrier name to upstream `SocketAddr` |
| `UpstreamConnector` | Chain connection through upstream SOCKS5 proxy |

| Type | Implements |
|---|---|
| `RouteTable` | `RouteResolver` -- carrier prefix to upstream mapping |
| `Socks5Chain` | `UpstreamConnector` -- SOCKS5 client for chaining |
| `RoutingSocksHandler<R, U>` | `ConnectionHandler` -- carrier-based SOCKS5 |
| `RoutingHttpHandler<R, U>` | `ConnectionHandler` -- carrier-based HTTP |

### API architecture (parallax-api, hexagonal)

```
adapter/http/ --> application/service.rs --> domain/port.rs <-- adapter/postgres.rs
   (inbound)        (use cases)              (traits only)       (outbound)
```

| Layer | Contents |
|---|---|
| `domain/entity.rs` | User, Plan, Carrier, Session + ZodSchema validation |
| `domain/port.rs` | Repository traits (UserRepository, PlanRepository, CarrierRepository, SessionRepository) |
| `application/service.rs` | AppService orchestrating use cases via port traits |
| `adapter/postgres.rs` | PostgreSQL implementations of all repository traits |
| `adapter/http/` | Axum route handlers (users CRUD, carriers, sessions, health) |

### Platform pages (parallax-platform)

| Route | Page |
|---|---|
| `/` | Dashboard -- stats, proxy endpoint status, quick connect |
| `/users` | User management -- create, suspend, delete, regen API key |
| `/proxies` | Proxy management -- endpoints, carriers, connection guide |
| `/api/*` | JSON API endpoints |

## Build

```sh
cargo build --release
```

## Proxy server

### Direct mode (exit node)

```sh
cargo run -p parallax-server

PROXY_LISTEN=0.0.0.0:1080 cargo run -p parallax-server
PROXY_USER=admin PROXY_PASS=secret cargo run -p parallax-server
```

### Routing mode (entry node)

```sh
ROUTES=telkomsel=10.0.0.2:1081,indosat=10.0.0.2:1082,xl=10.0.0.2:1083 \
PROXY_PASS=secret \
cargo run -p parallax-server
```

Clients connect with `{carrier}-{username}:{password}` credentials. The carrier prefix selects which upstream exit node to chain through.

### Configuration

| Variable | Default | Description |
|---|---|---|
| `PROXY_LISTEN` | `0.0.0.0:1080` | Listen address |
| `PROXY_USER` | *(unset)* | Single-user username (direct mode) |
| `PROXY_PASS` | *(unset)* | Password (direct and routing mode) |
| `ROUTES` | *(unset)* | Carrier routing table (enables routing mode) |

### Testing

```sh
# SOCKS5
curl -x socks5h://localhost:1080 http://httpbin.org/ip

# HTTP CONNECT
curl -x http://localhost:1080 https://httpbin.org/ip

# Routing mode
curl -x socks5h://telkomsel-admin:secret@localhost:1080 http://httpbin.org/ip
```

### Supported protocols

- **SOCKS5** (RFC 1928) with TCP CONNECT and username/password auth (RFC 1929)
- **HTTP CONNECT** tunneling for HTTPS passthrough
- **HTTP forward proxy** for plain HTTP requests
- Auto-detection per-connection: first byte `0x05` is SOCKS5, anything else is HTTP

## REST API

```sh
DATABASE_URL=postgres://user:pass@localhost/parallax cargo run -p parallax-api
```

| Method | Endpoint | Description |
|---|---|---|
| GET | `/api/health` | Health check |
| GET | `/api/users` | List users |
| POST | `/api/users` | Create user (validated with zod-rs) |
| DELETE | `/api/users/{id}` | Delete user |
| PATCH | `/api/users/{id}/toggle` | Suspend/activate user |
| POST | `/api/users/{id}/regenerate` | Regenerate API key |
| GET | `/api/carriers` | List carriers |
| GET | `/api/sessions` | List sessions |

### Configuration

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | *(required)* | PostgreSQL connection string |
| `API_LISTEN` | `0.0.0.0:3001` | Listen address |

## Dashboard

```sh
DATABASE_URL=postgres://user:pass@localhost/parallax topcoat dev
```

Requires the TopCoat CLI: `cargo install topcoat-cli@0.6.2`

## Infrastructure

Currently deployed on Hostinger VPS (`parallax.stynx.app:1080`) via NixOS with Clan. The proxy runs as a hardened `DynamicUser` systemd service.

### Mobile proxy roadmap

The platform is designed for carrier-based routing through Indonesian mobile networks. When USB 4G modems are connected to a Raspberry Pi:

```
Client --> parallax.stynx.app:1080 (Hostinger, entry)
           |
           WireGuard tunnel
           |
           Raspberry Pi + USB modems
           |-- Telkomsel (port 1081)
           |-- Indosat   (port 1082)
           |-- XL Axiata (port 1083)
```

Each modem gets its own exit proxy instance. The entry server routes by carrier prefix in the username.

## Dependencies

| Crate | Used by | Purpose |
|---|---|---|
| `tokio` | all | Async runtime |
| `async-trait` | all | Async methods in traits |
| `tracing` | server, api | Structured logging |
| `axum` | api | HTTP framework |
| `sqlx` | api, platform | PostgreSQL driver |
| `zod-rs` | api, platform | Request validation (ZodSchema derive) |
| `paginator-rs` | api, platform | Pagination |
| `topcoat` | platform | Full-stack web framework |

## License

MIT
