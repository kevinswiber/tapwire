## C.1 — Transport trait and implementations review (Phase C)

Scope: `shadowcat-cursor-review@eec52c8`

### Summary
- Default methods on `Transport` for `transport_type()`/`is_connected()` are footguns; require explicit impls.
- Timeout and size-limit behaviors differ across transports; document and align.
- Cooperative shutdown is implicit; add guidance and optional shutdown plumbing.
- Concurrency ergonomics can be improved without leaking locks to callers.
- Header casing must be canonicalized on write and case-insensitive on read.

### Key citations
```112:131:shadowcat-cursor-review/src/transport/mod.rs
#[async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&mut self) -> TransportResult<()>;
    async fn send(&mut self, envelope: MessageEnvelope) -> TransportResult<()>;
    async fn receive(&mut self) -> TransportResult<MessageEnvelope>;
    async fn close(&mut self) -> TransportResult<()>;

    fn session_id(&self) -> &SessionId;

    fn transport_type(&self) -> TransportType { TransportType::Stdio }
    fn is_connected(&self) -> bool { true }
}
```

```351:357:shadowcat-cursor-review/src/transport/stdio.rs
let line = timeout(recv_timeout, stdout_rx.recv())
    .await
    .map_err(|_| TransportError::Timeout("Receive timeout".to_string()))?
    .ok_or_else(|| TransportError::ReceiveFailed("Channel closed".to_string()))?;
```

```255:263:shadowcat-cursor-review/src/transport/http.rs
let response = timeout(
    Duration::from_millis(self.config.timeout_ms),
    request.send(),
)
.await
.map_err(|_| TransportError::Timeout("HTTP request timed out".to_string()))?
.map_err(|e| TransportError::SendFailed(format!("HTTP send failed: {e}")))?;
```

```818:827:shadowcat-cursor-review/src/proxy/forward.rs
// HTTP forwarder sets header using canonical casing
proxy_req = proxy_req.header("MCP-Protocol-Version", "2025-06-18");
```

```73:98:shadowcat-cursor-review/src/transport/http_mcp.rs
// Server-side extraction uses lower-case header names
let protocol_version = headers
    .get("mcp-protocol-version")
    .and_then(|v| v.to_str().ok())
    .unwrap_or(HTTP_DEFAULT_VERSION);
```

### Findings and proposals
- Require explicit connection state methods
  - Make `transport_type()` and `is_connected()` required methods (remove defaults) to prevent accidental stdio/true defaults from leaking into metrics or logs when new transports are added.
  - Document that `is_connected()` should be cheap and accurate.

- Timeouts: align semantics and error mapping
  - For `send()` and `receive()` across transports, use `TransportError::Timeout` for elapsed timers, and ensure callers can distinguish retryable timeouts from terminal failures.
  - Apply identical timeout windows to request send and response read when applicable; reference `TransportConfig.timeout_ms` as the single source of truth.
  - Add doc guidance that timeouts should be implemented via `tokio::time::timeout()` with informative error messages including operation and duration.

- Size limits: enforce consistently
  - Stdio checks outbound serialized length and inbound line size; HTTP checks serialized JSON length. Ensure both sides enforce `max_message_size` and surface `TransportError::MessageTooLarge { size, limit }` for callers.

- Cooperative shutdown and idempotency
  - Document `close()` as idempotent and responsible for terminating background tasks, draining channels, and releasing OS handles.
  - For transports that spawn tasks (e.g., stdio reader/writer), prefer using a shutdown signal and join-with-timeout before force-kill on drop. Avoid spawning work in `Drop` if a runtime may be unavailable.

- Concurrency ergonomics
  - Current proxies wrap transports with `Arc<RwLock<T>>` and take write locks for each call. Provide an optional adapter type for library users:

    ```rust
    pub struct ConcurrentTransport<T: Transport>(Arc<tokio::sync::RwLock<T>>);
    impl<T: Transport> ConcurrentTransport<T> {
        pub async fn send(&self, env: MessageEnvelope) -> TransportResult<()> { self.0.write().await.send(env).await }
        pub async fn receive(&self) -> TransportResult<MessageEnvelope> { self.0.write().await.receive().await }
        pub async fn close(&self) -> TransportResult<()> { self.0.write().await.close().await }
    }
    ```
  - Alternately, consider splitting the trait into send/receive halves for future transports that can be owned by separate tasks.

- Header casing guidance
  - When writing headers, standardize on canonical casing:
    - Request: `MCP-Protocol-Version`, `Mcp-Session-Id`
    - Response: `mcp-protocol-version`, `mcp-server`
  - When reading headers, treat names as case-insensitive. Document this explicitly in transport/server APIs.

- SSE integration
  - The SSE stack (`sse/*`) is not a `Transport` yet. If/when implemented, ensure it adopts the same config knobs (timeouts, max sizes) and shutdown semantics described here. Leverage `SessionAwareSseManager` for session binding and lifecycle hooks.

### Additional citations
```451:459:shadowcat-cursor-review/src/transport/stdio.rs
impl Drop for StdioTransport { /* spawns kill on child */ }
```

```368:381:shadowcat-cursor-review/src/transport/replay.rs
let mut outbound_rx = self.outbound_rx.lock().await; // await-in-lock pattern
let outbound_rx = outbound_rx.as_mut().ok_or(TransportError::Closed)?;
match outbound_rx.recv().await { /* ... */ }
```

### Action checklist (C.1)
- Update docs to require explicit `transport_type()`/`is_connected()`.
- Add doc section with timeout and size-limit expectations and error taxonomy hooks.
- Recommend adapter or split-trait pattern for concurrency ergonomics in docs.
- Document canonical header casing and case-insensitive reads.
- Note replay receive await-outside-lock improvement for future edits.
