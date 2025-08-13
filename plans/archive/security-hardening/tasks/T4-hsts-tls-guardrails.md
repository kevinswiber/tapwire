# T4: HSTS/TLS Guardrails

Objective: Provide operator guidance and runtime warnings for secure deployment.

## Implementation
- Docs: production TLS termination guidance; HSTS recommendations
- Runtime: warn when binding non-loopback without reverse proxy/TLS hints

## Target Areas
- `shadowcat/src/cli/reverse.rs` (warnings)
- Project docs/README

## Tests
- Optional: assert warning logs for insecure binds in dev mode

## Done When
- Docs merged; warnings present
