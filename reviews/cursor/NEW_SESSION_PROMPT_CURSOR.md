Session: Phase F kickoff — Security hardening & test plans

Repo/context
- Working dir: tapwire/
- Shadowcat snapshot (stable citations): shadowcat-cursor-review/ @ eec52c8 — analysis-only; DO NOT modify this snapshot
- Shadowcat delta worktree (latest main): shadowcat-delta/ @ b793fd1 — read-only for analysis; DO NOT commit code here
- Scope: Analysis-only. Update security analysis artifacts under `reviews/cursor/**`. Preserve existing `eec52c8` citations; add `shadowcat-delta/` citations where useful. No source edits.

Timebox: 60–90 minutes focused analysis

Pinned references
- CURSOR_RUST_CODE_REVIEWER.md
- reviews/cursor/tracker.md
- reviews/cursor/analysis/README.md
- reviews/cursor/analysis/api/docs.md (v0.3)
- reviews/cursor/analysis/api/errors.md
- reviews/cursor/analysis/api/transport.md
- reviews/cursor/analysis/api/proxy-session.md
- reviews/cursor/analysis/perf/hot-paths.md
- reviews/cursor/analysis/perf/recorder.md
- reviews/cursor/analysis/perf/interceptors.md

What changed last session
- Completed Phase D perf analyses with delta citations: `perf/hot-paths.md`, `perf/recorder.md`, `perf/interceptors.md`
- Updated `reviews/cursor/tracker.md` with Phase D marked complete and added a Phase D delta checklist
- Created Phase E security analyses: `analysis/security/tokens.md` and `analysis/security/transport.md`
- Updated `reviews/cursor/tracker.md` to reflect Phase E progress

Verification (Phase E)
- Confirm `reviews/cursor/analysis/security/tokens.md` includes allowlist guidance, SSE specifics, and exact citations.
- Confirm `reviews/cursor/analysis/security/transport.md` captures PKCE/state, JWT/JWKS validation, bind/origin/DNS-rebinding guidance with citations.
- Optional spot-check: re-run grep scans below to ensure no `Authorization` or `Cookie` headers are propagated to upstream in reverse HTTP/SSE paths.

Tasks (Phase F)
- F.1 Security test plan and assertions
  - Design tests to assert absence of `Authorization`/`Cookie` in upstream reverse HTTP and SSE requests; include positive/negative cases.
  - Specify unit vs integration coverage and fixtures.
  - Deliverable to create: `reviews/cursor/analysis/tests/security-plan.md`.

- F.2 Origin/Host validation and trusted proxy design
  - Propose validation rules for `Host`/`Origin` and trusted proxy handling for `X-Forwarded-*`.
  - Define config toggles and defaults (dev vs prod).
  - Deliverable to create: `reviews/cursor/analysis/security/origin-trusted-proxy.md`.

- F.3 Header allowlist at proxy boundaries
  - Draft allowlist/denylist for headers forwarded upstream; document rationale and migration plan.
  - Deliverable to create: `reviews/cursor/analysis/security/header-allowlist.md`.

- Tracker update
  - In `reviews/cursor/tracker.md`, add Phase E checklist/status and note any security deviations from prior guidance.

Suggested commands (analysis only)
- Build/lint/tests
  - cargo test -q --manifest-path shadowcat-delta/Cargo.toml
  - cargo clippy --manifest-path shadowcat-delta/Cargo.toml --all-targets -- -D warnings
- Grep targets
  - rg -n -i "Authorization|Proxy-Authorization|Bearer|token|cookie|set-cookie|x-api-key|x-access-token|reqwest|header|HeaderMap|TypedHeader|http::Header|MCP-Protocol-Version|Mcp-Session-Id|sse|event-stream|last-event-id" shadowcat-delta/src
  - rg -n -i "oauth2|pkce|jsonwebtoken|jwks|aud|iss|kid|origin|referer|Host|bind|tls|https|Dns|rebind|hsts|axum|headers" shadowcat-delta/src
- Optional
  - Review reverse proxy request/response header flows and SSE client header usage

Citations format reminder
- Use exact start:end:path blocks for all findings. Example:
```start:end:shadowcat-delta/src/proxy/http.rs
// code excerpt showing header handling
```

Success criteria
- Phase E verification complete (docs reviewed and grep spot-checks as needed)
- Drafted plans for Phase F deliverables (docs created as listed above)

Safety checklist
- No edits to `shadowcat-cursor-review/` or `shadowcat-delta/` source code
- Preserve `eec52c8` citations; add `b793fd1` delta citations where relevant
- Do not include secrets, tokens, or real endpoints in examples

Deliverables to update
- reviews/cursor/analysis/tests/security-plan.md (new)
- reviews/cursor/analysis/security/origin-trusted-proxy.md (new)
- reviews/cursor/analysis/security/header-allowlist.md (new)

Notes
- Do not edit source code in either worktree during analysis
- Maintain citation stability: use shadowcat-cursor-review@eec52c8 for baseline and shadowcat-delta@b793fd1 for delta
- Prefer exact start:end:path citations; keep examples conceptual without code edits