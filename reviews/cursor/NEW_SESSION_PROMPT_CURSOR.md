Session: Phase B (Safety/Async) — start B.1 and B.2

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

Tasks
- B.1 Unsafe/FFI audit:
  - Search for any `unsafe` blocks or FFI usage; document with citations.
  - Deliverable: reviews/cursor/analysis/safety/unsafe-audit.md
- B.2 Cancellation safety review:
  - Examine `tokio::select!`, stream handling, and shutdown paths for cancellation hazards (await-in-lock, leaked tasks).
  - Deliverable: reviews/cursor/analysis/async/cancellation.md
  - Tip: pay attention to `session::manager`, `transport::sse::reconnect`, and proxy loops.

Success criteria
- Unsafe/FFI audit doc created with cited code locations
- Cancellation safety review doc created with concrete findings and proposals
- Tracker updated to reflect Phase B progress

Deliverables to update
- reviews/cursor/analysis/safety/unsafe-audit.md
- reviews/cursor/analysis/async/cancellation.md
- reviews/cursor/tracker.md (Phase B statuses)

Notes
- Don’t change code; artifacts only.
- Cite files/lines and propose fixes without implementing.
