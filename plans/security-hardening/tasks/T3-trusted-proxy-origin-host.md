# T3: Trusted Proxy and Origin/Host Validation

Objective: Enforce safe use of `X-Forwarded-*` and validate Host/Origin.

## Implementation
- Config: `trusted_proxies`, `allowed_hosts`, `require_origin`
- Middleware: extract client IP with trust checks; validate Host/Origin for requests

## Target Areas
- `shadowcat/src/config/*.rs`
- `shadowcat/src/rate_limiting/key_extractors.rs` (or new module)
- `shadowcat/src/proxy/reverse.rs` (middleware wiring)

## Tests
- Unit: validators for IP trust, Host/Origin rules
- Integration: scenarios with/without trusted proxies

## Done When
- Tests passing; defaults safe; warnings for insecure binds
