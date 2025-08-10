# Security Test Plan (Phase F)

Objective: Assert that client tokens and cookies are not forwarded to upstreams and that security headers are handled per guidance.

## Scope
- Reverse proxy HTTP upstream requests
- Reverse proxy SSE upstream requests

## Assertions
- Upstream requests do NOT include `Authorization`, `Proxy-Authorization`, `Cookie`, `X-Api-Key`, `X-Access-Token` by default.
- MCP headers are present and correct: `MCP-Session-Id`, `MCP-Protocol-Version` (and `MCP-Client-Info` when available).
- SSE requests include `Accept: text/event-stream` and optional `Last-Event-Id` only.

## Rationale and citations
- Reverse SSE upstream request constructs headers explicitly (no auth/cookies):
```990:1006:shadowcat-delta/src/proxy/reverse.rs
let request = client
    .get(&sse_url)
    .header("Accept", "text/event-stream")
    .header("MCP-Session-Id", session_id.to_string())
    .header("MCP-Protocol-Version", &mcp_headers.protocol_version);
// optional MCP-Client-Info
```
- Reverse HTTP upstream request constructs headers explicitly (no auth/cookies):
```1419:1436:shadowcat-delta/src/proxy/reverse.rs
let mut request_builder = client
    .post(url)
    .header("Content-Type", "application/json")
    .header(
        "MCP-Protocol-Version",
        session
            .version_state
            .get_active_version()
            .unwrap_or(&crate::protocol::DEFAULT_PROTOCOL_VERSION.to_string()),
    )
    .header("MCP-Session-Id", session.id.to_string())
    .json(&json_body);
if let Some(client_info) = &session.client_info { request_builder = request_builder.header("MCP-Client-Info", client_info); }
```
- Upstream credentials (if configured) are injected internally, not forwarded from client:
```320:349:shadowcat-delta/src/proxy/load_balancer.rs
if let Some(auth) = &upstream.auth {
    match auth {
        UpstreamAuth::Bearer(token) => { /* default Authorization: Bearer <token> */ }
        UpstreamAuth::Basic { username, password } => { /* default Authorization: Basic <creds> */ }
    }
}
```

## Test Types
- Unit tests: construct requests via builders and inspect headers.
- Integration tests: mock upstream endpoint capturing headers; run reverse paths and assert absence/presence.

## Fixtures
- Mock SSE and HTTP servers that echo received headers.
- Configured upstream auth (Bearer/Basic) to validate only configured credentials are attached.

## Test matrix
- Reverse HTTP → upstream
  - No sensitive inbound headers: upstream sees only `Content-Type`, `MCP-Protocol-Version`, `MCP-Session-Id`, optional `MCP-Client-Info`.
  - With inbound `Authorization`/`Cookie`: upstream still must not see them.
  - With configured upstream Bearer/Basic: upstream sees configured `Authorization` only.

- Reverse SSE → upstream
  - No sensitive inbound headers: upstream sees `Accept: text/event-stream`, `MCP-Session-Id`, `MCP-Protocol-Version`, optional `MCP-Client-Info`.
  - With inbound `Authorization`/`Cookie`: upstream must not see them.
  - With `Last-Event-Id` in reconnect path: upstream sees it; no other sensitive headers.

## Example test scaffolding (pseudocode)
```rust
// Integration: HTTP upstream
#[tokio::test]
async fn reverse_http_does_not_forward_sensitive_headers() {
    // start mock upstream that records request headers
    let upstream = start_mock_http_server();

    // start reverse proxy pointing to mock upstream
    let proxy = start_reverse_proxy(upstream.url());

    // send client request to proxy with sensitive headers present
    let client = reqwest::Client::new();
    let res = client.post(proxy.url("/mcp"))
        .header("Authorization", "Bearer client-secret")
        .header("Cookie", "session=abc")
        .header("MCP-Session-Id", "test-session")
        .header("MCP-Protocol-Version", "2025-06-18")
        .json(&json!({"jsonrpc":"2.0","id":"1","method":"ping"}))
        .send().await.unwrap();
    assert!(res.status().is_success());

    // assert upstream did not receive sensitive headers
    let received = upstream.take_last_request();
    assert!(!received.headers.contains_key("authorization"));
    assert!(!received.headers.contains_key("cookie"));
    assert_eq!(received.headers.get("mcp-session-id"), Some(&"test-session"));
}

// Integration: SSE upstream
#[tokio::test]
async fn reverse_sse_does_not_forward_sensitive_headers() {
    let upstream = start_mock_sse_server();
    let proxy = start_reverse_proxy(upstream.url());

    let client = reqwest::Client::new();
    let res = client.get(proxy.url("/sse"))
        .header("Authorization", "Bearer client-secret")
        .header("Cookie", "session=abc")
        .header("MCP-Session-Id", "test-session")
        .header("MCP-Protocol-Version", "2025-06-18")
        .header("Accept", "text/event-stream")
        .send().await.unwrap();
    assert!(res.status().is_success());

    let received = upstream.take_last_request();
    assert_eq!(received.headers.get("accept"), Some(&"text/event-stream"));
    assert!(!received.headers.contains_key("authorization"));
    assert!(!received.headers.contains_key("cookie"));
}
```

## References
- See `analysis/security/tokens.md` for rationale and citations.
