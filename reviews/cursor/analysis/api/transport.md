# C.1 — Transport Trait and Implementations Review (Prep)

Scope: `shadowcat-cursor-review/` at `eec52c8`

- Trait summary
  - `Transport` defines async `connect/send/receive/close` with `session_id`, `transport_type`, and `is_connected` defaults.
  - Cite:
    ```112:131:shadowcat-cursor-review/src/transport/mod.rs
    pub trait Transport: Send + Sync { async fn connect(&mut self) -> _; async fn send(&mut self, _); async fn receive(&mut self) -> _; async fn close(&mut self) -> _; fn session_id(&self) -> &SessionId; fn transport_type(&self) -> TransportType { ... } fn is_connected(&self) -> bool { ... } }
    ```

- Initial observations
  - Default `transport_type` and `is_connected` return stdio/true; consider making them required methods to avoid incorrect defaults. Implementations already override these, but defaults are footguns.
  - `&mut self` async methods require external synchronization. Current code wraps transports in `Arc<RwLock<T>>` in forward proxy writers/readers, which is fine but verbose. Consider an API that splits send/recv halves or provides an internal concurrency guard to reduce external lock noise.
  - Lifecycle: some transports spawn background tasks (stdio, http SSE in the future, replay). Trait has no explicit shutdown token; consider optional `with_shutdown` on implementations and document cooperative shutdown expectations.

- Implementation highlights to review next
  - `stdio`, `http`, `http_mcp`, `replay`, `sse` modules.
  - Pay special attention to background tasks lifecycle and size limits enforcement consistency.

- Early proposals
  - Make `transport_type()` and `is_connected()` required (no defaults) to prevent accidental misuse in new implementations.
  - Add optional `shutdown(&mut self)` or standardized `with_shutdown(token)` in implementations that spawn tasks; document expectation that `close()` is idempotent and drains/terminates background work.
  - Provide guidance to avoid await-in-lock in `receive()` for wrapper transports (see replay), and prefer lock-free accumulation for metrics on hot paths.

- Notable impl notes and citations
  - Stdio transport: background IO tasks; Drop spawns kill; propose cooperative shutdown plumbing.
    ```451:459:shadowcat-cursor-review/src/transport/stdio.rs
    impl Drop for StdioTransport { /* spawns kill on child */ }
    ```
  - HTTP transport: has SSE placeholder; standard request/response with size checks and MCP headers; uses channel for receive in client mode.
    ```370:492:shadowcat-cursor-review/src/transport/http.rs
    impl Transport for HttpTransport { /* connect/send/receive/close */ }
    ```
  - Replay transport: await-in-lock on receiver; propose swap-out before await.
    ```368:381:shadowcat-cursor-review/src/transport/replay.rs
    let mut outbound_rx = self.outbound_rx.lock().await; let outbound_rx = outbound_rx.as_mut().ok_or(...)?; match outbound_rx.recv().await { ... }
    ```
  - HTTP MCP server transport: wraps channels for incoming/outgoing; envelope contexts use HTTP variants.
    ```203:264:shadowcat-cursor-review/src/transport/http_mcp.rs
    impl Transport for HttpMcpTransport { /* connect/send/receive/close */ }
    ```
  - SSE: session-aware manager, connection manager/client; ensure any future `Transport` impl for SSE respects cooperative shutdown.
    ```41:89:shadowcat-cursor-review/src/transport/sse/session.rs
    pub async fn start_monitoring(mut self) -> Self { tokio::spawn(... interval ...); }
    ```

## API refinements

- Header naming consistency
  - Ensure consistent casing for MCP headers across transports (currently mix of `MCP-Protocol-Version` and lower-case in reverse path). Standardize on canonical casing when constructing, accept case-insensitive for parsing.
  - Citations:
    ```818:827:shadowcat-cursor-review/src/proxy/forward.rs
    request = request.header("MCP-Protocol-Version", "2025-06-18");
    ```
    ```1188:1213:shadowcat-cursor-review/src/proxy/reverse.rs
    if let Some(protocol_version) = headers.get("mcp-protocol-version") { /* ... */ }
    ```

- Timeout semantics
  - HTTP and stdio use `TransportConfig.timeout_ms`; ensure consistent timeout usage for send/receive paths and document expected behavior on timeout (error type, retry guidance).
    ```351:357:shadowcat-cursor-review/src/transport/stdio.rs
    let line = timeout(recv_timeout, stdout_rx.recv()).await
    ```
    ```255:263:shadowcat-cursor-review/src/transport/http.rs
    let response = timeout(Duration::from_millis(self.config.timeout_ms), request.send())
    ```

- Trait ergonomics
  - Consider a thin adapter type `ConcurrentTransport<T: Transport>` that encapsulates `Arc<RwLock<T>>` and exposes `send/receive` without leaking lock details to callers (used by forward proxy), or split `Transport` into read/write halves to enable separate task ownership.

- Cooperative shutdown in trait docs
  - Document that `close()` should be idempotent and terminate background work; if implementations spawn tasks, provide `with_shutdown(token)` or similar in their API.
