# T1: Header Allowlist — Reverse HTTP

Objective: Ensure reverse HTTP upstream requests only include allowed headers.

## Implementation
- Add explicit allowlist builder for upstream headers
- Drop sensitive inbound headers

## Target Areas
- `shadowcat/src/proxy/reverse.rs` — HTTP proxy request building

## Allowed Headers
- `Content-Type: application/json`
- `MCP-Session-Id`
- `MCP-Protocol-Version`
- Optional `MCP-Client-Info`
- Configured `Authorization` (from upstream config only)

## Tests
- Unit: header builder
- Integration: mock upstream echoes headers, client sends sensitive headers; assert not forwarded

## Done When
- Tests passing; docs updated
