# Security Hardening Plan — Tracker

Scope: Turn analysis docs into actionable implementation tasks with clear deliverables and success criteria.

## Milestones
- M1: Header allowlist guards (reverse HTTP/SSE)
- M2: Trusted proxy and Origin/Host validation
- M3: Security tests (HTTP/SSE integration + unit)

## Tasks
- T1: Implement header allowlist in reverse HTTP upstream path
  - Files: `shadowcat/src/proxy/reverse.rs` (or equivalent), config schema
  - Behavior: Construct upstream headers from allowlist; drop sensitive inbound headers
  - Tests: Unit + integration
- T2: Implement header allowlist in reverse SSE upstream path
  - Files: reverse SSE proxy code
  - Behavior: Same as T1, with SSE specifics
  - Tests: Integration with mock SSE server
- T3: Add config for `trusted_proxies`, `allowed_hosts`, `require_origin`
  - Files: `shadowcat/src/config/*.rs`, CLI flags as needed
  - Behavior: Validate `X-Forwarded-*` only when remote addr is trusted; enforce Host/Origin rules
  - Tests: Unit validators + integration
- T4: Add HSTS/TLS doc and guardrails
  - Files: docs/config; runtime warnings if bound to `0.0.0.0` without secure fronting
  - Tests: N/A (logging assertions optional)
- T5: Security tests
  - Files: `shadowcat/tests/security_*.rs`
  - Behavior: Assert no sensitive forward; assert allowlist correctness

## Success Criteria
- All new tests pass; no token/cookie propagation
- Configurable trusted proxy handling; safe defaults
- Clear operator guidance for prod

## References
- `reviews/cursor/analysis/security/*`
- `reviews/cursor/analysis/tests/security-plan.md`
