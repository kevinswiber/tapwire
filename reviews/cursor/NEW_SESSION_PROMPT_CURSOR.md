Session: Phase E kickoff — Security & Compliance

Repo/context
- Working dir: tapwire/
- Shadowcat snapshot (stable citations): shadowcat-cursor-review/ @ eec52c8 — analysis-only; DO NOT modify this snapshot
- Shadowcat delta worktree (latest main): shadowcat-delta/ @ b793fd1 — read-only for analysis; DO NOT commit code here
- Scope: Analysis-only. Update security analysis artifacts under `reviews/cursor/**`. Preserve existing `eec52c8` citations; add `shadowcat-delta/` citations where useful. No source edits.

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

Tasks (Phase E)
- E.1 Token handling and header scrubbing
  - Verify client tokens are never forwarded upstream; ensure Authorization and related headers are scrubbed or regenerated at proxy boundaries.
  - Inspect reverse/forward proxy HTTP paths and SSE HTTP client for header propagation. Confirm MCP headers are correct and no client secrets leak.
  - Grep targets: `Authorization`, `Bearer`, `token`, `header`, `reqwest`, `request`, `proxy` in `shadowcat-delta/src/**`.
  - Capture start:end:path citations and guidance in `reviews/cursor/analysis/security/tokens.md`.

- E.2 OAuth 2.1 and transport security checks
  - Review auth gateway components for OAuth 2.1 basics: PKCE, audience validation, token storage, JWT verification hygiene.
  - Check transport security: origin validation, DNS rebinding protection, localhost defaults for dev, TLS expectations for prod.
  - Grep targets: `oauth2`, `PKCE`, `jsonwebtoken`, `origin`, `Host`, `bind`, `Tls`, `https`, `axum`, `headers`.
  - Document findings with exact citations in `reviews/cursor/analysis/security/transport.md`.

- Tracker update
  - In `reviews/cursor/tracker.md`, add Phase E checklist/status and note any security deviations from prior guidance.

Suggested commands (analysis only)
- Build/lint/tests
  - cargo test -q --manifest-path shadowcat-delta/Cargo.toml
  - cargo clippy --manifest-path shadowcat-delta/Cargo.toml --all-targets -- -D warnings
- Grep targets
  - rg "Authorization|Bearer|token|reqwest|header|set_header|http::Header|MCP-Protocol-Version|Mcp-Session-Id" shadowcat-delta/src -n
  - rg "oauth2|pkce|jsonwebtoken|origin|Host|bind|tls|https|Dns|rebind" shadowcat-delta/src -n -i
- Optional
  - Review reverse proxy request/response header flows and SSE client header usage

Success criteria
- `reviews/cursor/analysis/security/tokens.md` updated with token/header handling analysis and mitigation guidance
- `reviews/cursor/analysis/security/transport.md` updated with OAuth 2.1 and transport security findings
- `reviews/cursor/tracker.md` updated to reflect Phase E task statuses

Deliverables to update
- reviews/cursor/analysis/security/tokens.md
- reviews/cursor/analysis/security/transport.md
- reviews/cursor/tracker.md (Phase E checklist/status)

Notes
- Do not edit source code in either worktree during analysis
- Maintain citation stability: use shadowcat-cursor-review@eec52c8 for baseline and shadowcat-delta@b793fd1 for delta
- Prefer exact start:end:path citations; keep examples conceptual without code edits