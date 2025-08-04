# Shadowcat Architecture Plan

**Project:** Shadowcat (MCP Forward & Reverse Proxy)  
**Version:** v0.1  
**Date:** August 4, 2025  
**Status:** Draft

---

## 1. Executive Summary

Shadowcat is the core proxy component of Tapwire, implementing both forward and reverse proxy capabilities for Model Context Protocol (MCP) traffic. Built in Rust for performance and reliability, it provides transparent interception, recording, replay, and security enforcement for MCP communications across stdio and HTTP transports.

---

## 2. Architecture Overview

### 2.1 High-Level Components

```
┌─────────────────────────────────────────────────────────────────┐
│                         Shadowcat Core                           │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   Forward   │  │   Reverse   │  │  Recording  │            │
│  │    Proxy    │  │    Proxy    │  │   Engine    │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │ Interceptor │  │   Session   │  │    Auth     │            │
│  │   Engine    │  │   Manager   │  │   Gateway   │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
├─────────────────────────────────────────────────────────────────┤
│                     Transport Layer                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │    Stdio    │  │ Streamable  │  │   Legacy    │            │
│  │  Transport  │  │    HTTP     │  │  HTTP+SSE   │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

1. **Forward Proxy Mode**: Client → Shadowcat → MCP Server
   - Development/debugging tool - launches and manages MCP server process
   - Primarily uses stdio transport
   - No authentication needed (client controls both ends)

2. **Reverse Proxy Mode**: Client → Shadowcat (auth/policy) → MCP Server  
   - Production deployment pattern - clients connect TO shadowcat
   - HTTP server accepting client connections
   - OAuth 2.1 authentication gateway enforces security
   - Policy engine controls access to upstream MCP servers

3. **Recording**: All traffic → Storage (with timing/metadata)
4. **Replay**: Storage → Shadowcat → Client (deterministic playback)

---

## 3. Core Dependencies

### 3.1 Essential Crates

```toml
[dependencies]
# MCP Protocol Implementation
rmcp = { version = "0.2", features = ["server", "client"] }

# Async Runtime
tokio = { version = "1.43", features = ["full"] }

# HTTP/WebSocket Support
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }
hyper = "1.5"
tokio-tungstenite = "0.26"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging & Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Storage & Persistence
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "json"] }

# Security & Auth
jsonwebtoken = "9"
oauth2 = "4.4"
ring = "0.17"

# Configuration
config = "0.14"
clap = { version = "4.5", features = ["derive"] }

# Error Handling
thiserror = "2.0"
anyhow = "1.0"

# Testing
mockall = "0.13"
```

### 3.2 Optional/Future Dependencies

```toml
# Metrics & Observability
opentelemetry = "0.27"
opentelemetry-otlp = "0.27"
prometheus = "0.13"

# Rate Limiting
governor = "0.7"

# Policy Engine
opa-wasm = "0.8"
```

---

## 4. Module Structure

```
shadowcat/
├── src/
│   ├── main.rs                 # Entry point, CLI parsing
│   ├── lib.rs                  # Public API
│   ├── config/
│   │   ├── mod.rs              # Configuration management
│   │   └── schema.rs           # Config validation
│   ├── transport/
│   │   ├── mod.rs              # Transport abstraction
│   │   ├── stdio.rs            # Stdio transport impl
│   │   ├── http.rs             # HTTP/SHTTP transport
│   │   └── sse.rs              # Legacy SSE support
│   ├── proxy/
│   │   ├── mod.rs              # Proxy trait definitions
│   │   ├── forward.rs          # Forward proxy logic
│   │   └── reverse.rs          # Reverse proxy logic
│   ├── session/
│   │   ├── mod.rs              # Session management
│   │   ├── store.rs            # Session storage
│   │   └── state.rs            # Session state machine
│   ├── interceptor/
│   │   ├── mod.rs              # Interception engine
│   │   ├── rules.rs            # Rule matching/execution
│   │   ├── actions.rs          # Rewrite/mock/fault actions
│   │   └── ui.rs               # Interactive intercept UI
│   ├── recorder/
│   │   ├── mod.rs              # Recording engine
│   │   ├── tape.rs             # Tape format/storage
│   │   └── replay.rs           # Replay engine
│   ├── auth/
│   │   ├── mod.rs              # Auth gateway
│   │   ├── oauth.rs            # OAuth 2.1 implementation
│   │   └── policy.rs           # Access control
│   ├── metrics/
│   │   ├── mod.rs              # Metrics collection
│   │   └── export.rs           # OTLP/Prometheus export
│   └── error.rs                # Error types
├── tests/
│   ├── integration/
│   └── conformance/            # MCP protocol conformance
└── examples/
    ├── forward_stdio.rs
    ├── reverse_http.rs
    └── record_replay.rs
```

---

## 5. Key Components Design

### 5.1 Transport Layer

**Stdio Transport**
- Process management with tokio::process
- Bidirectional stream handling
- Clean stderr separation for logging

**HTTP Transport**
- Streamable HTTP (POST for requests, GET for streaming)
- Session header management (Mcp-Session-Id)
- Protocol version negotiation
- SSE fallback for legacy servers

### 5.2 Session Management

```rust
pub struct Session {
    pub id: SessionId,
    pub server_session_id: Option<String>,
    pub transport: TransportType,
    pub protocol_version: String,
    pub created_at: Instant,
    pub auth_state: AuthState,
    pub frames: Vec<Frame>,
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    persistence: Box<dyn SessionStore>,
}
```

### 5.3 Recording Engine

```rust
pub struct Tape {
    pub id: TapeId,
    pub session_id: SessionId,
    pub frames: Vec<RecordedFrame>,
    pub metadata: TapeMetadata,
}

pub struct RecordedFrame {
    pub timestamp: Instant,
    pub direction: Direction,
    pub transport_edge: TransportEdge,
    pub content: FrameContent,
    pub is_synthetic: bool,
}
```

### 5.4 Interceptor Engine

```rust
pub struct InterceptRule {
    pub id: RuleId,
    pub matcher: Matcher,
    pub action: InterceptAction,
    pub scope: RuleScope,
}

pub enum InterceptAction {
    Rewrite(RewriteSpec),
    Mock(MockResponse),
    Fault(FaultSpec),
    PassThrough,
}
```

### 5.5 Auth Gateway

```rust
pub struct AuthGateway {
    pub oauth_config: OAuth2Config,
    pub token_validator: TokenValidator,
    pub policy_engine: PolicyEngine,
}

impl AuthGateway {
    pub async fn validate_request(&self, req: &Request) -> Result<AuthContext, AuthError> {
        // 1. Extract token
        // 2. Validate audience
        // 3. Check resource server metadata
        // 4. NEVER forward client tokens upstream
        // 5. Exchange for resource-specific token if needed
    }
}
```

---

## 6. Implementation Phases

### Phase 1: Core Infrastructure (Weeks 1-2)
- [ ] Basic project structure and dependency setup
- [ ] Transport abstraction layer
- [ ] Stdio transport implementation
- [ ] Basic forward proxy for stdio
- [ ] Session management foundation
- [ ] Logging and error handling

### Phase 2: HTTP Support (Weeks 3-4)
- [ ] Streamable HTTP transport
- [ ] Session header handling
- [ ] Protocol version negotiation
- [ ] Basic reverse proxy
- [ ] Legacy SSE compatibility

### Phase 3: Recording & Replay (Weeks 5-6)
- [ ] Tape format design
- [ ] Recording engine
- [ ] Storage layer (SQLite)
- [ ] Basic replay functionality
- [ ] Deterministic replay guarantees

### Phase 4: Interception (Weeks 7-8)
- [ ] Manual intercept (pause/edit/resume)
- [ ] Rule engine foundation
- [ ] Basic rewrite actions
- [ ] Mock responses
- [ ] Interactive UI (TUI or web)

### Phase 5: Reverse Proxy & Security (Weeks 9-10)
- [ ] Reverse proxy HTTP server implementation
- [ ] OAuth 2.1 authentication gateway integration
- [ ] Token validation and audience checking
- [ ] Policy engine for authorization decisions
- [ ] Security audit logging
- [ ] No client token passthrough enforcement (MCP requirement)

### Phase 6: Observability (Weeks 11-12)
- [ ] Metrics collection
- [ ] Latency tracking
- [ ] Error rate monitoring
- [ ] OTLP export
- [ ] Dashboard templates

---

## 7. Technical Decisions

### 7.1 Why Rust?
- Performance: Near-zero overhead for proxy operations
- Memory safety: Critical for security-sensitive proxy
- Async ecosystem: Excellent with tokio
- Type safety: Prevents protocol violations at compile time

### 7.2 Storage Choice (SQLite)
- Embedded, no external dependencies
- Excellent for local development
- JSON support for flexible schema
- Easy migration path to PostgreSQL

### 7.3 Async Runtime (Tokio)
- De facto standard in Rust ecosystem
- rmcp requires tokio
- Excellent performance characteristics
- Rich ecosystem of compatible crates

---

## 8. Security Considerations

### 8.1 Default Security Posture
- Bind to localhost only by default
- Require explicit flag for 0.0.0.0 binding
- DNS rebinding protection
- Origin validation for HTTP transport

### 8.2 Auth Security
- No token passthrough (enforced)
- Audience validation mandatory
- OAuth 2.1 compliance
- Resource server metadata discovery

### 8.3 Recording Security
- Sensitive data masking in tapes
- Encrypted storage option
- Access control for replay

---

## 9. Performance Targets

- **Latency overhead**: < 5% p95 for typical tool calls
- **Memory usage**: < 100MB for 1000 concurrent sessions
- **Throughput**: > 10,000 requests/second
- **Startup time**: < 100ms
- **Recording overhead**: < 10% additional latency

---

## 10. Testing Strategy

### 10.1 Unit Tests
**Transport Layer**
- Mock stdio process lifecycle
- HTTP request/response handling
- SSE stream parsing and resumption
- Error propagation and recovery

**Session Management**
- Concurrent session creation/deletion
- Session timeout handling
- State transitions
- Frame ordering guarantees

**Interceptor Engine**
- Rule matching logic
- Action execution order
- Pause/resume mechanics
- Context preservation

**Auth Module**
- Token validation
- Audience verification
- OAuth flow compliance
- Token exchange logic

### 10.2 Integration Tests
**Proxy Flows**
- Full request/response cycle
- Multi-turn conversations
- Streaming responses
- Session handover between transports

**Recording/Replay**
- Deterministic playback
- Timing preservation
- State reconstruction
- Synthetic frame injection

**Security Scenarios**
- Invalid token rejection
- Token passthrough prevention
- Origin validation
- DNS rebinding protection

### 10.3 Conformance Tests
**Protocol Compliance**
- JSON-RPC 2.0 formatting
- Method name validation
- Parameter schema checking
- Error code compliance

**Transport Behavior**
- Session header propagation
- Protocol version negotiation
- Connection lifecycle
- Graceful degradation

### 10.4 Performance Tests
**Baseline Metrics**
- Single request latency: < 0.5ms overhead
- Throughput: > 10k req/s on 4-core machine
- Memory per session: < 100KB
- Startup time: < 50ms

**Stress Testing**
- 10k concurrent sessions
- 1M requests sustained load
- Memory leak detection
- File descriptor limits

---

## 11. CLI Interface

```bash
# Forward proxy (stdio)
shadowcat forward stdio -- npx @modelcontextprotocol/server-everything

# Forward proxy (HTTP)
shadowcat forward http --port 8080 --target http://localhost:3000

# Reverse proxy
shadowcat reverse --port 8080 --upstream http://mcp-server.example.com \
  --auth-config auth.yaml

# Recording
shadowcat record --output session.tape -- npm run mcp-server

# Replay
shadowcat replay session.tape --port 8080

# Interactive mode
shadowcat --interactive
```

---

## 12. Configuration Schema

```yaml
# shadowcat.yaml
proxy:
  forward:
    enabled: true
    transports: [stdio, http]
  reverse:
    enabled: true
    bind: "127.0.0.1:8080"
    
session:
  timeout: 3600
  max_sessions: 1000
  
recording:
  enabled: true
  storage: "./recordings"
  compression: true
  
interceptor:
  enabled: true
  rules_file: "./rules.yaml"
  
auth:
  oauth:
    issuer: "https://auth.example.com"
    audience: "mcp-servers"
    discovery: true
    
metrics:
  enabled: true
  export:
    otlp:
      endpoint: "localhost:4317"
```

---

## 13. Integration Points

### 13.1 VS Code Integration
- Helper extension for .vscode/mcp.json
- Automatic proxy configuration
- Session viewer integration

### 13.2 MCP Inspector
- Export format compatibility
- Launch Inspector with recorded sessions
- Shared session format

### 13.3 CI/CD Integration
- GitHub Actions for replay
- Jenkins plugin
- GitLab CI templates

---

## 14. Open Questions

1. **Should we support WebSocket transport?** (Not in MCP spec yet)
2. **Real-time streaming to web UI?** (Via WebSocket or SSE)
3. **Plugin architecture for custom interceptors?**
4. **Multi-tenant support in v1?**
5. **WASM module support for policy engine?**

---

## 15. Success Criteria

- [ ] Successfully proxy stdio MCP servers with < 5% overhead
- [ ] Record and replay complete sessions deterministically
- [ ] Intercept and modify requests without breaking protocol
- [ ] Enforce OAuth security without token passthrough
- [ ] Pass all MCP conformance tests
- [ ] Handle 1000+ concurrent sessions

---

## 16. Next Steps

### Immediate Actions (Day 1)
1. **Project Setup**
   ```bash
   cd shadowcat
   # Update Cargo.toml with core dependencies
   cargo add rmcp tokio --features tokio/full
   cargo add axum tower tower-http
   cargo add serde serde_json
   cargo add tracing tracing-subscriber
   cargo add clap --features derive
   cargo add thiserror anyhow
   ```

2. **Create Module Structure**
   ```bash
   mkdir -p src/{transport,proxy,session,interceptor,recorder,auth,metrics,config}
   touch src/{error.rs,lib.rs}
   touch src/transport/{mod.rs,stdio.rs,http.rs}
   touch src/proxy/{mod.rs,forward.rs,reverse.rs}
   ```

3. **Implement Core Abstractions**
   - Start with `Transport` trait in `src/transport/mod.rs`
   - Implement `TransportMessage` enum
   - Create basic error types in `src/error.rs`

### Week 1 Milestone
- [ ] Working stdio transport that can spawn a process
- [ ] Basic message serialization/deserialization
- [ ] Simple forward proxy that passes messages through
- [ ] Unit tests for transport layer
- [ ] Basic CLI that can proxy a hello-world MCP server

### Proof of Concept Target
```bash
# This should work by end of Week 1:
cargo run -- forward stdio -- echo '{"jsonrpc":"2.0","method":"initialize","id":1}'
# Should see the message proxied and response returned
```

---

## Appendix A: Tape Format Specification (Draft)

```json
{
  "version": "1.0",
  "tape_id": "550e8400-e29b-41d4-a716-446655440000",
  "session_id": "session-123",
  "metadata": {
    "created_at": "2025-08-04T10:00:00Z",
    "duration_ms": 5432,
    "transport": "stdio",
    "protocol_version": "2025-11-05"
  },
  "frames": [
    {
      "timestamp_ms": 0,
      "direction": "client_to_server",
      "edge": "transport_in",
      "content": {
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {},
        "id": 1
      }
    }
  ]
}
```

---

## Appendix B: Rule Format Specification (Draft)

```yaml
rules:
  - id: "delay-tool-calls"
    description: "Add 500ms delay to all tool calls"
    matcher:
      method: "tools/call"
    action:
      type: "fault"
      fault:
        delay_ms: 500
        
  - id: "mock-weather"
    description: "Return mock weather data"
    matcher:
      method: "tools/call"
      params:
        name: "get_weather"
    action:
      type: "mock"
      response:
        content:
          - type: "text"
            text: "Sunny, 72°F"
```