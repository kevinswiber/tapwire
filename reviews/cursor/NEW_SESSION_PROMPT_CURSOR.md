Session: Phase C (API, Errors, Boundaries) — continue C.1–C.3; start C.4

Repo/context
- Working dir: shadowcat-cursor-review/
- Branch: current HEAD (do not rebase this snapshot)
- Commit: eec52c8 (perf(mcp): optimize event ID generator for high throughput)
- Scope: analysis-only; DO NOT modify source files. Write artifacts under reviews/cursor/**
- Important: This review operates on a static snapshot for citation stability. Do not rebase. A separate delta audit against latest main will be done after Phase C.

Pinned references
- CURSOR_RUST_CODE_REVIEWER.md
- reviews/cursor/tracker.md
- reviews/cursor/analysis/README.md
- reviews/cursor/analysis/findings/baseline.md
- reviews/cursor/analysis/safety/unsafe-audit.md
- reviews/cursor/analysis/async/cancellation.md
- reviews/cursor/analysis/async/locking.md
- reviews/cursor/analysis/async/proposals.md
- reviews/cursor/analysis/api/transport.md
- reviews/cursor/analysis/api/proxy-session.md
- reviews/cursor/analysis/api/errors.md

Tasks
- C.1 Transport trait and implementations (continue):
  - Review `src/transport/{mod.rs,stdio.rs,http.rs,http_mcp.rs,replay.rs,sse/**}` for API clarity, header casing consistency, timeout semantics, and cooperative shutdown guidance. Update: `reviews/cursor/analysis/api/transport.md`.
- C.2 Proxy engine and session lifecycle (continue):
  - Review forward/reverse proxy APIs for shutdown lifecycles and interceptor effects mapping; ensure `SessionManager` recording uses correct `TransportContext`. Update: `reviews/cursor/analysis/api/proxy-session.md`.
- C.3 Error handling and Result flows (continue):
  - Draft error taxonomy aligning internal errors to JSON-RPC codes and HTTP statuses; verify `IntoResponse` mapping. Update: `reviews/cursor/analysis/api/errors.md`.
- C.4 Public API docs/examples (start):
  - Create `analysis/api/docs.md` summarizing public traits, key structs, and example flows (forward stdio, reverse HTTP), with guidance on shutdown and error mapping.
  - Do not change code; examples are conceptual.

Success criteria
- C.1–C.3 docs enriched with actionable proposals and precise citations
- C.4 docs.md created with examples and guidance
- Tracker updated to reflect Phase C progress and post-Phase-C delta audit plan

Deliverables to update
- reviews/cursor/analysis/api/{transport,proxy-session,errors}.md
- reviews/cursor/analysis/api/docs.md (new)
- reviews/cursor/tracker.md (Phase C statuses + delta audit note)

Notes
- Don’t change code; artifacts only.
- Cite files/lines and propose fixes without implementing.
 - Maintain citation stability; do not rebase snapshot. A separate worktree will be created post-Phase C for delta analysis.
