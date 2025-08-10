# C.1 — Transport Trait and Implementations Review (Prep)

Scope: `shadowcat-cursor-review/` at `eec52c8`

- Trait summary
  - `Transport` defines async `connect/send/receive/close` with `session_id`, `transport_type`, and `is_connected` defaults.
  - Cite:
    ```112:131:shadowcat-cursor-review/src/transport/mod.rs
    pub trait Transport: Send + Sync { async fn connect(&mut self) -> _; async fn send(&mut self, _); async fn receive(&mut self) -> _; async fn close(&mut self) -> _; fn session_id(&self) -> &SessionId; fn transport_type(&self) -> TransportType { ... } fn is_connected(&self) -> bool { ... } }
    ```

- Initial observations
  - Default `transport_type` and `is_connected` return stdio/true; consider making them required methods to avoid incorrect defaults.
  - `&mut self` on async methods implies exclusive mutability; pool/pipeline code wraps in `RwLock` to share. Consider interior mutability pattern or splitting read/write halves.

- Implementation highlights to review next
  - `stdio`, `http`, `http_mcp`, `replay`, `sse` modules.
  - Pay special attention to background tasks lifecycle and size limits enforcement consistency.

- Early proposals
  - Make `transport_type()` and `is_connected()` required (no defaults) to prevent accidental misuse in new implementations.
  - Add optional `shutdown(&mut self)` with cooperative semantics for implementations that spawn tasks.
