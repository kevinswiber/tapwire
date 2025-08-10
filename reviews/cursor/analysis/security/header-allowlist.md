# Header Allowlist at Proxy Boundaries

## Rationale
Prevent accidental leakage of client secrets by forwarding only the headers the proxy must send upstream.

## Allowlist (Upstream Requests)
- MCP protocol headers: `MCP-Session-Id`, `MCP-Protocol-Version`, optional `MCP-Client-Info`
- Content headers: `Content-Type: application/json`, `Accept` (for SSE: `text/event-stream`)
- SSE specific: optional `Last-Event-Id`
- Configured upstream credentials only (if set in config): `Authorization: Bearer ...` or `Authorization: Basic ...`

## Denylist (Never forward from client)
- `Authorization`, `Proxy-Authorization`, `Cookie`, `Set-Cookie`, `X-Api-Key`, `X-Access-Token`, `X-Forwarded-*`

## Implementation Notes
- Construct upstream requests from scratch with explicit headers (current approach). Do not pass through inbound HeaderMap.
- Add unit/integration tests asserting absence of sensitive headers in upstream requests.
- Log at debug when dropping sensitive headers (without values) for traceability.

### Citations (current behavior)
- Reverse SSE upstream request uses explicit headers (no pass-through):
```990:1006:shadowcat-delta/src/proxy/reverse.rs
// Accept, MCP-Session-Id, MCP-Protocol-Version [+ optional MCP-Client-Info]
```
- Reverse HTTP upstream request uses explicit headers (no pass-through):
```1419:1436:shadowcat-delta/src/proxy/reverse.rs
// Content-Type, MCP-Protocol-Version, MCP-Session-Id [+ optional MCP-Client-Info]
```

## Tests
- See `analysis/tests/security-plan.md`.
