# Shadowcat Architecture Guide

## Overview

Shadowcat is a high-performance Model Context Protocol (MCP) proxy implementation in Rust, serving as the core engine of the Tapwire platform. It provides transparent interception, recording, replay, and security enforcement for MCP communications across multiple transport protocols.

## Architecture Principles

### Core Design Goals
- **Performance**: < 5% latency overhead, support for 10,000+ concurrent sessions
- **Modularity**: Clean separation between transport, protocol, and proxy layers
- **Extensibility**: Plugin-based architecture for interceptors and processors
- **Security**: OAuth 2.1 authentication, rate limiting, and policy enforcement
- **Observability**: Comprehensive telemetry and audit logging

### Technology Stack
- **Language**: Rust (stable toolchain)
- **Async Runtime**: Tokio
- **HTTP Framework**: Axum + Tower
- **Protocol**: MCP v2025-11-05, v2025-03-26, v2025-06-18
- **Storage**: SQLite (sessions), filesystem (tapes)
- **Serialization**: Serde JSON

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI Interface                            │
│         forward | reverse | record | replay | intercept          │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                         Proxy Core                               │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Forward    │  │   Reverse    │  │   Gateway    │         │
│  │    Proxy     │  │    Proxy     │  │    Proxy     │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│         │                  │                  │                  │
│  ┌──────────────────────────────────────────────────┐          │
│  │            Session Manager (SQLite)               │          │
│  └──────────────────────────────────────────────────┘          │
│         │                  │                  │                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │ Interceptor  │  │   Recorder   │  │   Replay     │         │
│  │   Engine     │  │              │  │   Engine     │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                      Transport Layer                             │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │    stdio     │  │  Streamable  │  │     SSE      │         │
│  │              │  │     HTTP     │  │              │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│  ┌────────────────────────────────────────────────────┐        │
│  │          MessageEnvelope (Protocol Abstraction)     │        │
│  └────────────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                       MCP Protocol Layer                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Version    │  │    Parser    │  │  Correlation │         │
│  │  Negotiation │  │              │  │              │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Transport Layer (`src/transport/`)

The transport layer provides a unified abstraction over different communication mechanisms:

#### Raw Transports
- **stdio**: Process-based communication via stdin/stdout
- **HTTP**: HTTP client/server with connection pooling
- **SSE**: Server-Sent Events for server→client notifications
- **Streamable HTTP**: Combined HTTP POST + SSE (MCP v2025-03-26+)

#### MessageEnvelope System
The `MessageEnvelope` provides protocol-agnostic message handling:
```rust
pub struct MessageEnvelope {
    pub content: Bytes,           // Raw message content
    pub metadata: EnvelopeMetadata, // Transport metadata
    pub session_id: Option<String>, // Session correlation
}
```

This separation allows the proxy to handle messages without parsing protocol details at the transport layer.

### 2. Proxy Modes (`src/proxy/`)

#### Forward Proxy
- **Purpose**: Development and debugging tool
- **Flow**: Client → Shadowcat → MCP Server
- **Features**:
  - Subprocess management
  - Transparent interception
  - Recording/replay capabilities
  - No authentication (trusted environment)

#### Reverse Proxy
- **Purpose**: Production deployment pattern
- **Flow**: Client → Shadowcat (auth/policy) → MCP Server
- **Features**:
  - OAuth 2.1 authentication gateway
  - Rate limiting and policy enforcement
  - Load balancing with health checks
  - Circuit breaker for resilience

#### Gateway Proxy
- **Purpose**: Multi-upstream routing
- **Features**:
  - Multiple upstream server support
  - Load balancing strategies
  - Failover and retry logic
  - Per-upstream configuration

### 3. Session Management (`src/session/`)

Thread-safe session lifecycle management with SQLite persistence:

```rust
pub struct Session {
    pub id: String,
    pub created_at: SystemTime,
    pub transport_type: TransportType,
    pub state: SessionState,
    pub metadata: HashMap<String, Value>,
}
```

Features:
- Automatic session expiry
- Transport-specific metadata storage
- SSE reconnection support with Last-Event-Id
- Concurrent session handling

### 4. Interceptor System (`src/interceptor/`)

Pluggable message processing pipeline:

```rust
pub trait Interceptor: Send + Sync {
    async fn process(&self, envelope: MessageEnvelope) 
        -> Result<InterceptorAction>;
}

pub enum InterceptorAction {
    Continue(MessageEnvelope),
    Pause { resume_token: String },
    Modify { envelope: MessageEnvelope },
    Block { reason: String },
}
```

Built-in interceptors:
- **MCPInterceptor**: Protocol-aware message handling
- **RulesInterceptor**: Pattern-based routing and filtering
- **HttpPolicyInterceptor**: HTTP-specific security policies

### 5. Recording & Replay (`src/recorder/`, `src/replay/`)

#### Recording Engine
- Captures all traffic with precise timing
- Supports multiple storage formats (binary, JSONL)
- Metadata preservation for session context

#### Replay Engine
- Deterministic playback with timing control
- Message transformation capabilities
- SSE event stream reconstruction

### 6. Authentication & Security (`src/auth/`)

#### OAuth 2.1 Gateway
- PKCE (Proof Key for Code Exchange) support
- JWT validation with JWKS fetching
- Token introspection and refresh

#### Rate Limiting
- Multi-tier rate limiting (global, per-user, per-endpoint)
- Token bucket algorithm
- Configurable rate policies

#### Policy Engine
- Resource-based access control
- Method-level restrictions
- Custom policy expressions

### 7. Telemetry & Observability (`src/telemetry/`, `src/audit/`)

- OpenTelemetry integration
- Structured logging with tracing
- Metrics collection (latency, throughput, errors)
- Audit trail for compliance

## Data Flow Patterns

### 1. Forward Proxy Flow
```
1. Client spawns shadowcat with MCP server command
2. Shadowcat launches server subprocess
3. Client ←→ Shadowcat ←→ Server (stdio transport)
4. Optional: Recording to tape storage
5. Optional: Interceptor processing
```

### 2. Reverse Proxy Flow
```
1. Client connects to shadowcat HTTP endpoint
2. Authentication via OAuth 2.1 flow
3. Session establishment with MCP handshake
4. Client ←→ Shadowcat ←→ Upstream (HTTP/SSE)
5. Policy enforcement at each message
6. Audit logging for compliance
```

### 3. Recording Flow
```
1. Messages flow through proxy
2. Envelope captured with metadata
3. Written to tape storage (JSONL format)
4. Session correlation maintained
5. Timing information preserved
```

### 4. Replay Flow
```
1. Tape loaded from storage
2. Session reconstruction
3. Timed message replay
4. Optional transformations applied
5. Client receives deterministic playback
```

## Configuration Architecture

### Configuration Hierarchy
```
1. CLI arguments (highest priority)
2. Environment variables
3. Configuration files (TOML/JSON)
4. Default values (lowest priority)
```

### Key Configuration Areas
- Transport settings (ports, timeouts)
- Authentication (OAuth providers, JWKS)
- Rate limiting (tiers, limits)
- Storage (database paths, tape directories)
- Telemetry (exporters, sampling)

## Performance Characteristics

### Benchmarks (v0.2.0)
- **Latency overhead**: < 3% p95 (target: < 5%)
- **Memory usage**: ~60KB per session (target: < 100KB)
- **Throughput**: 63,000+ sessions/sec (target: > 10,000)
- **Startup time**: < 50ms (target: < 100ms)

### Optimization Strategies
- Zero-copy message passing with `Bytes`
- Buffer pooling for transport operations
- Lazy parsing (only when needed)
- Connection pooling for HTTP
- Async I/O throughout

## Security Architecture

### Defense in Depth
1. **Transport Security**: TLS for HTTP, localhost binding by default
2. **Authentication**: OAuth 2.1 with PKCE, JWT validation
3. **Authorization**: Policy engine with resource-based access control
4. **Rate Limiting**: Multi-tier protection against abuse
5. **Audit Logging**: Complete trail for compliance

### Security Boundaries
- Client tokens never forwarded to upstream
- Session isolation in multi-tenant scenarios
- Input validation at every layer
- Secure defaults (deny by default)

## Extension Points

### Custom Interceptors
Implement the `Interceptor` trait for custom processing:
```rust
#[async_trait]
impl Interceptor for MyCustomInterceptor {
    async fn process(&self, envelope: MessageEnvelope) 
        -> Result<InterceptorAction> {
        // Custom logic here
    }
}
```

### Custom Storage Providers
Implement storage traits for alternative backends:
```rust
impl SessionStore for RedisStore {
    async fn create(&self, session: Session) -> Result<()>;
    async fn get(&self, id: &str) -> Result<Option<Session>>;
}
```

### Transport Extensions
Add new transport types by implementing core traits:
```rust
impl Transport for MyTransport {
    async fn send(&mut self, envelope: MessageEnvelope) -> Result<()>;
    async fn receive(&mut self) -> Result<MessageEnvelope>;
}
```

## Future Architecture Directions

### In Progress (Top Priority)
1. **Reverse Proxy Session Mapping**: Dual session ID tracking for SSE
2. **Multi-Session Forward Proxy**: Concurrent client support

### Planned Enhancements
1. **Redis Session Storage**: Distributed session management
2. **WebAssembly Modules**: Wassette integration for sandboxed extensions
3. **Full Batch Support**: Complete MCP batch message handling
4. **Smart CLI**: Auto-detection of transport types

See `plans/README.md` for detailed roadmap and implementation status.