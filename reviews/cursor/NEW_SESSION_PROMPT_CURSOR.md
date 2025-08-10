Session: Phase B (Safety/Async) — deepen B.1–B.3; prep Phase C

Repo/context
- Working dir: shadowcat-cursor-review/
- Branch: current HEAD (do not rebase)
- Commit: eec52c8 (perf(mcp): optimize event ID generator for high throughput)
- Scope: analysis-only; DO NOT modify source files. Write artifacts under reviews/cursor/**

Pinned references
- CURSOR_RUST_CODE_REVIEWER.md
- reviews/cursor/tracker.md
- reviews/cursor/analysis/README.md
- reviews/cursor/analysis/findings/baseline.md
- reviews/cursor/analysis/safety/unsafe-audit.md
- reviews/cursor/analysis/async/cancellation.md
- reviews/cursor/analysis/async/locking.md

Tasks
- B.1 Unsafe/FFI audit (continue):
  - Confirm no `unsafe`/FFI; expand notes on `Drop` behavior and metrics accumulation.
  - Update: reviews/cursor/analysis/safety/unsafe-audit.md
- B.2 Cancellation safety (continue):
  - Draft concrete shutdown/token patterns for forward proxy, health checker, stdio transport.
  - Update: reviews/cursor/analysis/async/cancellation.md
- B.3 Locking analysis (continue):
  - Propose lock-free/lock-minimizing alternatives for replay receive and metrics.
  - Update: reviews/cursor/analysis/async/locking.md
- Prep Phase C scope notes:
  - Create stubs for `analysis/api/transport.md`, `analysis/api/proxy-session.md`, `analysis/api/errors.md`.

Success criteria
- Unsafe/FFI, cancellation, and locking docs updated with actionable proposals
- API review stubs created for Phase C
- Tracker updated to reflect Phase B progress and Phase C prep

Deliverables to update
- reviews/cursor/analysis/safety/unsafe-audit.md
- reviews/cursor/analysis/async/cancellation.md
- reviews/cursor/analysis/async/locking.md
- reviews/cursor/analysis/api/{transport,proxy-session,errors}.md
- reviews/cursor/tracker.md (Phase B statuses)

Notes
- Don’t change code; artifacts only.
- Cite files/lines and propose fixes without implementing.
