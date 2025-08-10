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

Additional related mappings in reverse proxy code paths show where 400s are emitted and where upstream errors occur:
```680:688:shadowcat-cursor-review/src/proxy/reverse.rs
return Ok((StatusCode::BAD_REQUEST, Json(error_response)).into_response());
```
```1159:1176:shadowcat-cursor-review/src/proxy/reverse.rs
ReverseProxyError::UpstreamConnectionFailed("Failed to send HTTP request: {e}")
// ... parse/serialize failures mapped to UpstreamConnectionFailed
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

Citations of timeout surfacing in transports (for consistent mapping to 504 where appropriate):
```351:357:shadowcat-cursor-review/src/transport/stdio.rs
timeout(recv_timeout, stdout_rx.recv()).await.map_err(|_| TransportError::Timeout("Receive timeout".to_string()))?
```
```393:399:shadowcat-cursor-review/src/transport/http.rs
timeout(...).await.map_err(|_| TransportError::Timeout("Connection health check timed out".to_string()))?
```

### Action checklist (C.3)
- Extend `IntoResponse` mapping to use 502/504 and custom -3200x codes for rate limiting/auth in reverse proxy.
- Add error taxonomy section to developer docs; align CLI exit codes with HTTP statuses.
- Encourage `context("while sending HTTP request to {url}")` style around IO.
- Document examples of `error.data` with `type`, `status`, and optional `retry_after`.

### Addendum (Delta)
Delta findings against `shadowcat-delta@b793fd1` (preserving existing `eec52c8` citations):

- Error-to-HTTP and JSON-RPC mapping in reverse proxy remains centralized in `impl IntoResponse for ReverseProxyError` and `to_http_status()`:

```1648:1673:shadowcat-delta/src/proxy/reverse.rs
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

        let body = Json(serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": error_code,
                "message": self.to_string(),
                "data": {
                    "type": std::any::type_name_of_val(&self),
                    "status": status.as_u16(),
                }
            }
        }));

        (status, body).into_response()
    }
}
```

```275:289:shadowcat-delta/src/error.rs
impl ReverseProxyError {
    pub fn to_http_status(&self) -> StatusCode {
        match self {
            Self::InvalidHeaders(_) => StatusCode::BAD_REQUEST,
            Self::SessionCreationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ProtocolVersionMismatch { .. } => StatusCode::BAD_REQUEST,
            Self::ProtocolError(_) => StatusCode::BAD_REQUEST,
            Self::UpstreamConnectionFailed(_) => StatusCode::BAD_GATEWAY,
            Self::AuthenticationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::HttpError { status, .. } => StatusCode::from_u16(*status)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
```

- Divergences from Phase C taxonomy to note:
  - Upstream timeouts are not distinguished as 504; upstream failures funnel to 502 via `UpstreamConnectionFailed`. Example HTTP upstream path uses a 30s client timeout but maps errors to `UpstreamConnectionFailed` (502):

```1403:1441:shadowcat-delta/src/proxy/reverse.rs
let client = Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .pool_max_idle_per_host(10)
    .build()
    .map_err(|e| ReverseProxyError::UpstreamConnectionFailed(format!(
        "Failed to create HTTP client: {e}"
    )))?;
...
let response = request_builder.send().await.map_err(|e| {
    ReverseProxyError::UpstreamConnectionFailed(format!("Failed to send HTTP request: {e}"))
})?;
```

  - AuthN/Z mapping for `ReverseProxyError::AuthenticationFailed` returns 500 rather than 401/403.
  - Custom JSON-RPC codes from Phase C (e.g., -32001 rate limit, -32002 auth, -32010 recording/replay) are not surfaced via `ReverseProxyError`; rate limiting emits HTTP 429 through its own responder with standard headers:

```153:171:shadowcat-delta/src/rate_limiting/middleware.rs
fn create_rate_limit_error_response(error: &RateLimitError) -> Response {
    let (status, message, retry_after) = match error {
        RateLimitError::GlobalLimitExceeded => (
            StatusCode::TOO_MANY_REQUESTS,
            "Global rate limit exceeded".to_string(),
            Some(60),
        ),
        ...
    };
```

- Client input/header errors correctly map to 400 and JSON-RPC -32600 in both validation and response shaping.

```1467:1492:shadowcat-delta/src/proxy/reverse.rs
fn validate_mcp_response_headers(headers: &reqwest::header::HeaderMap) -> ReverseProxyResult<()> {
    if let Some(protocol_version) = headers.get("mcp-protocol-version") {
        let version_str = protocol_version.to_str().map_err(|_| {
            ReverseProxyError::InvalidHeaders("Invalid MCP-Protocol-Version header".to_string())
        })?;
        ...
    }
    if let Some(session_id) = headers.get("mcp-session-id") {
        let _session_str = session_id.to_str().map_err(|_| {
            ReverseProxyError::InvalidHeaders("Invalid MCP-Session-Id header".to_string())
        })?;
    }
    Ok(())
}
```

Implications:
- Consider adding a timeout-specific variant or mapping to emit 504 for explicit elapsed timers.
- Adjust `AuthenticationFailed` to 401/403 per context; keep 403 for policy denials.
- If emitting JSON-RPC error bodies for middleware-generated 429/401/403, align codes with Phase C (-32001/-32002) for consistency.
