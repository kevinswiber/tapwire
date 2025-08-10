# Token Handling and Header Scrubbing

This document analyzes how Shadowcat handles tokens and sensitive headers across transports and proxy paths, and proposes mitigation where needed.

## Findings

### SSE transport/header construction (client side)
```320:350:shadowcat-delta/src/transport/sse_transport.rs
// Build headers
let mut headers = HeaderMap::new();
headers.insert(
    "MCP-Session-Id",
    self.session_id.to_string().parse().unwrap(),
);
headers.insert(
    "MCP-Protocol-Version",
    self.config.protocol_version.as_str().parse().unwrap(),
);
headers.insert("X-Event-Id", event_id.parse().unwrap());
```

```317:348:shadowcat-delta/src/transport/sse/client.rs
fn build_headers(&self, session_id: Option<&str>, protocol_version: &str, last_event_id: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/event-stream"));
    headers.insert("MCP-Protocol-Version", HeaderValue::from_str(protocol_version) ...);
    if let Some(session_id) = session_id { headers.insert("Mcp-Session-Id", value); }
    if let Some(last_id) = last_event_id { headers.insert("Last-Event-Id", value); }
    headers
}
```

- No `Authorization` or `Cookie` headers are set in SSE construction. Good default.

### HTTP transport (client to upstream)
```236:253:shadowcat-delta/src/transport/http.rs
let mut headers = HashMap::new();
headers.insert("Content-Type", "application/json");
headers.insert("MCP-Protocol-Version", MCP_PROTOCOL_VERSION);
headers.insert("Mcp-Session-Id", self.session_id.to_string());
for (key, value) in headers { request = request.header(&key, &value); }
```

- Only MCP and content-type headers forwarded. No auth headers injected here. Good.

### Reverse proxy outbound requests
SSE upstream request:
```990:1006:shadowcat-delta/src/proxy/reverse.rs
let request = client
    .get(&sse_url)
    .header("Accept", "text/event-stream")
    .header("MCP-Session-Id", session_id.to_string())
    .header("MCP-Protocol-Version", &mcp_headers.protocol_version);
// optional MCP-Client-Info
```

HTTP upstream request:
```1419:1436:shadowcat-delta/src/proxy/reverse.rs
let mut request_builder = client
    .post(url)
    .header("Content-Type", "application/json")
    .header("MCP-Protocol-Version", session.version_state.get_active_version() ...)
    .header("MCP-Session-Id", session.id.to_string())
    .json(&json_body);
if let Some(client_info) = &session.client_info { request_builder = request_builder.header("MCP-Client-Info", client_info); }
```

- Reverse proxy constructs upstream headers explicitly; does not forward client `Authorization`/`Cookie`. Good.

### Upstream auth configuration (internal only)
```320:349:shadowcat-delta/src/proxy/load_balancer.rs
if let Some(auth) = &upstream.auth {
    match auth {
        UpstreamAuth::Bearer(token) => { default Authorization: Bearer <token> }
        UpstreamAuth::Basic { username, password } => { default Authorization: Basic <creds> }
    }
}
```

- Upstream credentials defined in config are applied at the client, not forwarded from end-user requests. Acceptable.

### Token parsing/validation
```228:244:shadowcat-delta/src/auth/token.rs
pub fn extract_bearer_token(headers: &axum::http::HeaderMap) -> AuthResult<String> { ... requires "Authorization: Bearer ..." }
```
```246:291:shadowcat-delta/src/auth/token.rs
pub async fn validate_token(&self, token: &str) -> Result<TokenClaims> { ... validate alg, iss, aud; fetch keys as needed }
```

## Risks
- No explicit denylist for sensitive inbound headers on reverse proxy (e.g., `Authorization`, `Cookie`) to prevent accidental pass-through if future code proxies arbitrary headers.
- SSE reconnect headers include `Last-Event-Id`, but no accidental auth headers observed. Continue to ensure allowlist.

## Recommendations
- Enforce allowlist on headers sent upstream in reverse proxy (HTTP and SSE). Current code already constructs headers explicitly; document and test that no other headers are forwarded.
- Add middleware to strip or ignore inbound sensitive headers for upstream calls: `Authorization`, `Proxy-Authorization`, `Cookie`, `Set-Cookie`, `X-Api-Key`, `X-Access-Token`.
- Add tests asserting absence of `Authorization`/`Cookie` in upstream requests for both HTTP and SSE paths.

## Summary
- No evidence of client token leakage upstream. Maintain explicit header construction and consider an allowlist guard for future-proofing.
