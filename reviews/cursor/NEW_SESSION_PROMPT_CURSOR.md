Session: Delta Audit kickoff (Shadowcat main) — addenda to Phase C docs

Repo/context
- Working dir: tapwire/
- Shadowcat snapshot (stable citations): shadowcat-cursor-review/ @ eec52c8 — analysis-only; DO NOT modify this snapshot
- Shadowcat delta worktree (latest main): shadowcat-delta/ @ b793fd1 — read-only for analysis; DO NOT commit code here
- Scope: Update analysis artifacts under reviews/cursor/** only. Preserve existing eec52c8 citations; append new “Addendum (Delta)” sections citing shadowcat-delta/ paths

Pinned references
- CURSOR_RUST_CODE_REVIEWER.md
- reviews/cursor/tracker.md
- reviews/cursor/analysis/README.md
- reviews/cursor/analysis/api/docs.md (v0.3)
- reviews/cursor/analysis/api/errors.md
- reviews/cursor/analysis/api/transport.md
- reviews/cursor/analysis/api/proxy-session.md

What changed last session
- Phase C C.4 finalized: public API docs updated to v0.3 (error mapping table, cross-links, casing guidance)
- Delta worktree created at shadowcat-delta/ commit b793fd1; tests and clippy passed
- “Addendum (Delta)” stubs appended to API analysis docs; tracker bumped to 0.6 and C.4 marked complete

Tasks
- Populate Addendum (Delta) sections with shadowcat-delta citations (keep existing eec52c8 content intact):
  1) Error mapping (reverse proxy)
     - Validate status and JSON-RPC code mapping in shadowcat-delta/src/proxy/reverse.rs:
       - -32600 → 400 for client input/header errors
       - -32603 → 502/504 for upstream failures vs explicit timeouts
       - -32001 → 429 with Retry-After when available
       - -32002 → 401/403 for authN/Z
       - -32010 → 400/409 for replay/recording domain
     - Capture exact IntoResponse mapping and any helper methods (to_http_status, error body construction) with start:end:path citations
     - Update `reviews/cursor/analysis/api/errors.md` Addendum with findings and note any divergences

  2) Header casing (write/read)
     - Writers: confirm canonical casing in shadowcat-delta/src/transport/http.rs and any SSE header builders
       - Expect: "MCP-Protocol-Version", "Mcp-Session-Id", etc.
     - Readers: confirm case-insensitive extraction in shadowcat-delta/src/transport/http_mcp.rs and reverse proxy request paths
       - Expect: lower-case lookups like "mcp-protocol-version"
     - Update `reviews/cursor/analysis/api/transport.md` Addendum with both write/read citations and reconcile examples in `api/docs.md`

  3) Timeouts and size limits parity
     - Confirm `TransportConfig.timeout_ms` and `max_message_size` enforced in stdio and http transports
       - Look for tokio::time::timeout usage and MessageTooLarge branches in shadowcat-delta/src/transport/stdio.rs and http.rs
     - Add citations and note any changed behaviors or error variants in `api/transport.md` Addendum; reflect in `api/docs.md` if needed

  4) Recording and context accuracy
     - Verify construction of `MessageContext`/`TransportContext` in shadowcat-delta paths:
       - reverse proxy: correct TransportContext::http for endpoints (SSE/HTTP)
       - session recording sites: ensure no default TransportContext::stdio() leaks when HTTP edge is known
     - Update `api/proxy-session.md` Addendum with precise before/after if behavior changed; cite exact lines

  5) Interceptor behavior and shutdown sequencing
     - Note any changes to interceptor effects or forward proxy shutdown (token/joins vs aborts)
     - Add citations in `api/proxy-session.md` Addendum

- Tracker update
  - In `reviews/cursor/tracker.md`, add a short Delta Audit section with a checklist for the above, and record any notable deviations from the Phase C taxonomy or guidance

Suggested commands (do not modify code; analysis only)
- Verify delta worktree state
  - git -C shadowcat-delta rev-parse --short HEAD
  - cargo test -q --manifest-path shadowcat-delta/Cargo.toml
  - cargo clippy --manifest-path shadowcat-delta/Cargo.toml --all-targets -- -D warnings

- Find relevant code quickly (use ripgrep)
  - rg "impl IntoResponse for ReverseProxyError" shadowcat-delta/src -n
  - rg "MCP-Protocol-Version|Mcp-Session-Id" shadowcat-delta/src -n
  - rg "mcp-protocol-version" shadowcat-delta/src -n
  - rg "timeout\(|MessageTooLarge" shadowcat-delta/src -n
  - rg "TransportContext::stdio\(|TransportContext::http\(" shadowcat-delta/src -n

Success criteria
- Addendum sections in `api/docs.md`, `api/errors.md`, `api/transport.md`, `api/proxy-session.md` updated with shadowcat-delta citations and concise findings
- Existing eec52c8 content preserved; new examples stay conceptual and aligned with taxonomy
- `reviews/cursor/tracker.md` updated with a Delta Audit checklist/status

Deliverables to update
- reviews/cursor/analysis/api/docs.md (Addendum section content)
- reviews/cursor/analysis/api/errors.md (Addendum content)
- reviews/cursor/analysis/api/transport.md (Addendum content)
- reviews/cursor/analysis/api/proxy-session.md (Addendum content)
- reviews/cursor/tracker.md (Delta section + checklist)

Notes
- Do not edit source code in either worktree during analysis
- Maintain citation stability: use shadowcat-cursor-review@eec52c8 for baseline and shadowcat-delta@b793fd1 for delta
- Prefer exact start:end:path citations; keep examples conceptual without code edits