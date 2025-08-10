# Origin/Host Validation and Trusted Proxy Handling

## Goals
- Block DNS rebinding and enforce correct Host/Origin where applicable.
- Avoid trusting spoofed `X-Forwarded-*` headers unless behind a configured trusted proxy.

## Proposal
- Default: ignore `X-Forwarded-*` unless `trusted_proxies` is non-empty.
- Validation:
  - Require Host header for HTTP endpoints; allowlist configured hosts.
  - Validate `Origin`/`Referer` for browser-facing endpoints (if any); reject cross-origin.
- Config:
  - `server.trusted_proxies: [CIDRs]`
  - `server.allowed_hosts: [hostnames]`
  - `server.require_origin: true|false` (for browser contexts)

### Citations (current behavior)
- Bind defaults to loopback (dev-safe):
```170:176:shadowcat-delta/src/proxy/reverse.rs
bind_address: "127.0.0.1:8080".parse().expect(...)
```
```236:249:shadowcat-delta/src/config/reverse_proxy.rs
ServerSettings { bind_address: "127.0.0.1:8080", cors_enabled: true, ... }
```
- Client IP extraction currently trusts `X-Forwarded-*` unconditionally (risk if no trusted proxy):
```119:152:shadowcat-delta/src/rate_limiting/key_extractors.rs
// reads x-forwarded-for, x-real-ip, cf-connecting-ip directly
```

## Dev vs Prod
- Dev: bind `127.0.0.1`, permissive defaults with warnings.
- Prod: require explicit `allowed_hosts` and `trusted_proxies` when binding non-loopback.

## Tests
- Unit: header validators for Host/Origin/XFF.
- Integration: run server with/without trusted proxies; assert behavior using mocked requests.

## References
- See `analysis/security/transport.md` for risks and context.
