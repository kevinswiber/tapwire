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
