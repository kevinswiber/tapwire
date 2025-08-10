## Shadowcat public API overview and examples (Phase C, C.4)

Scope: `shadowcat-cursor-review@eec52c8` — conceptual docs, no code edits.

### Core traits and types
- Transport trait and config
```112:131:shadowcat-cursor-review/src/transport/mod.rs
#[async_trait]
pub trait Transport: Send + Sync { /* connect/send/receive/close; session_id */ }
```
- Message model and context
```158:206:shadowcat-cursor-review/src/transport/envelope.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContext { /* session_id, direction, transport, protocol_version, timestamp */ }
```
- Concrete transports
  - Stdio: `src/transport/stdio.rs`
  - HTTP client: `src/transport/http.rs`
  - HTTP MCP server-side: `src/transport/http_mcp.rs`
  - Replay: `src/transport/replay.rs`

- Proxy engines
  - Forward proxy: `src/proxy/forward.rs`
  - Reverse proxy (HTTP server): `src/proxy/reverse.rs`

- Sessions
  - `SessionManager`: create/update/record/cleanup with DoS protections

### Example: forward proxy (stdio ↔ http)
Conceptual example wiring a client transport to a server transport. This runs both directions and applies interception and recording when provided.

```rust
use shadowcat::proxy::ForwardProxy;
use shadowcat::transport::{StdioTransport, HttpTransport, Transport, MessageEnvelope, MessageDirection, TransportContext, MessageContext};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = StdioTransport::new(tokio::process::Command::new("npx").arg("@modelcontextprotocol/server-everything"));
    let mut server = HttpTransport::new(Url::parse("http://127.0.0.1:8080/mcp")?);

    let mut proxy = ForwardProxy::new();
    // optionally: proxy = proxy.with_session_manager(sm).with_tape_recorder(tr).with_interceptor_chain(chain);

    proxy.start(client, server).await?;
    // later: proxy.shutdown().await?;
    Ok(())
}
```

Guidance:
- Ensure transports are connected by `start()`; it will spawn reader/writer tasks.
- Provide a shutdown signal (see below) and prefer join-with-timeout before aborting tasks.

### Example: reverse proxy (HTTP server → stdio upstream)
Minimal reverse proxy server that accepts HTTP MCP requests and proxies to a stdio upstream.

```rust
use shadowcat::proxy::reverse::{ReverseProxyServer, ReverseProxyConfig, ReverseUpstreamConfig};
use shadowcat::session::SessionManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let session_manager = Arc::new(SessionManager::new());
    let mut config = ReverseProxyConfig::default();
    config.upstream_configs = vec![ReverseUpstreamConfig::stdio(
        "local-stdio",
        vec!["npx".into(), "@modelcontextprotocol/server-everything".into()],
    )];

    ReverseProxyServer::new(config, session_manager)
        .start()
        .await?;
    Ok(())
}
```

### Shutdown and lifecycle guidance
- Transports
  - `close()` must be idempotent and should stop background tasks, close channels, and release handles.
  - Use a shutdown token in implementations that spawn tasks; join tasks with a small timeout before force-killing.

- Forward proxy
  - Add a `with_shutdown(token)` style API; on shutdown: signal, drain, join, then force abort if needed.

- Reverse proxy
  - HTTP server shutdown should rely on server’s graceful shutdown primitive; ensure session cleanup via `SessionManager::shutdown()` when stopping the server.

### Error mapping guidance
- JSON-RPC and HTTP statuses should follow a stable taxonomy:
  - -32600 ↔ 400 for invalid input/headers
  - -32603 ↔ 502/504 for upstream/timeout
  - -32001 ↔ 429 for rate limiting (include `retry-after`)
  - -32002 ↔ 401/403 for auth

Cited reverse mapping:
```1366:1392:shadowcat-cursor-review/src/proxy/reverse.rs
impl IntoResponse for ReverseProxyError { /* maps error to status + JSON */ }
```

### Header casing guidance
- When writing: use canonical casing (`MCP-Protocol-Version`, `Mcp-Session-Id`).
- When reading: treat header names as case-insensitive.

Citations:
```818:827:shadowcat-cursor-review/src/proxy/forward.rs
proxy_req = proxy_req.header("MCP-Protocol-Version", "2025-06-18");
```
```73:98:shadowcat-cursor-review/src/transport/http_mcp.rs
headers.get("mcp-protocol-version").and_then(|v| v.to_str().ok())
```

### Timeouts and size limits
- `TransportConfig.timeout_ms` governs send/receive timeouts; use `tokio::time::timeout()` and return `TransportError::Timeout`.
- `TransportConfig.max_message_size` enforced for both outbound serialized payloads and inbound frames; return `TransportError::MessageTooLarge { size, limit }`.

Citations:
```312:321:shadowcat-cursor-review/src/transport/stdio.rs
if serialized.len() > self.config.max_message_size { return Err(TransportError::MessageTooLarge { .. }); }
```
```116:126:shadowcat-cursor-review/src/transport/stdio.rs
// inbound size check in stdout reader
```

### Recording accuracy
- Build `TransportContext` based on the reader’s known transport type rather than defaulting to stdio in session recording paths, to preserve accurate edge metadata for analytics and replay.

Citation:
```835:842:shadowcat-cursor-review/src/session/manager.rs
TransportContext::stdio(), // Default context - could be improved
```

---
Document version: 0.1 (created as part of Phase C)
