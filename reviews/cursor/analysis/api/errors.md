## C.3 — Error handling and Result flows (Phase C)

Scope: `shadowcat-cursor-review@eec52c8`

### Summary
- Reverse proxy maps internal errors to HTTP and JSON-RPC codes; extend taxonomy and ensure consistent use across transports/CLI.
- Add guidance to use `anyhow::Context` at IO boundaries for actionable logs.
- Reserve JSON-RPC code ranges for upstream vs client input vs rate limiting/auth.

### Key citation
```1366:1392:shadowcat-cursor-review/src/proxy/reverse.rs
impl IntoResponse for ReverseProxyError {
    fn into_response(self) -> Response {
        let status = self.to_http_status();
        let error_code = match &self {
            ReverseProxyError::InvalidHeaders(_) => -32600,
            ReverseProxyError::ProtocolVersionMismatch { .. } => -32600,
            ReverseProxyError::SessionCreationFailed(_) => -32603,
            ReverseProxyError::UpstreamConnectionFailed(_) => -32603,
            _ => -32603,
        };
        // ... JSON body construction ...
        (status, body).into_response()
    }
}
```

### Proposed taxonomy and mappings
- Client input / protocol violations:
  - JSON-RPC code: -32600 (Invalid Request)
  - HTTP status: 400 Bad Request
  - Examples: invalid/missing MCP headers, malformed JSON-RPC, version downgrade attempts.

- Upstream and transport failures (server side):
  - JSON-RPC code: -32603 (Internal error)
  - HTTP status: 502 Bad Gateway (connection/send/receive), 504 Gateway Timeout (explicit timeouts)
  - Examples: stdio child not responding, HTTP upstream non-2xx, SSE invalid content type.

- Rate limiting:
  - JSON-RPC code: -32001 (custom range -32000..-32099)
  - HTTP status: 429 Too Many Requests
  - Include `retry-after` hint when available.

- Authentication/authorization:
  - JSON-RPC code: -32002
  - HTTP status: 401 Unauthorized (missing/invalid), 403 Forbidden (policy denial)

- Replay/recording specific errors:
  - JSON-RPC code: -32010
  - HTTP status: 400/409 as appropriate (e.g., invalid tape, conflict)

Document these codes in public API docs and keep them stable. Where applicable, include an `error.data` object with `type`, `status`, and optional `retry_after`.

### Transport/HTTP guidance
- Wrap external IO with context for diagnostics:
  - Example areas: stdio spawn/read/write, HTTP request/send/parse.
  - Use `anyhow::Context` or structured `TransportError` variants with cause chains so logs identify the boundary and operation.

### Action checklist (C.3)
- Extend `IntoResponse` mapping to use 502/504 and custom -3200x codes for rate limiting/auth in reverse proxy.
- Add error taxonomy section to developer docs; align CLI exit codes with HTTP statuses.
- Encourage `context("while sending HTTP request to {url}")` style around IO.
