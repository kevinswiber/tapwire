Session: Phase C wrap-up (Public API docs) → prepare Delta Audit

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
- reviews/cursor/analysis/api/docs.md

What changed last session
- C.1–C.3 finalized with precise citations and concrete recommendations (timeouts/size limits, header casing, cooperative shutdown, interceptor effects, metrics, error taxonomy).
- C.4 upgraded: public API `docs.md` moved to v0.2 with context construction, shutdown token example, and interceptor behaviors.
- Tracker updated to v0.5: C.1–C.3 complete; C.4 in progress; delta audit next.

Tasks
- C.4 Public API docs/examples (finalize):
  - Update `reviews/cursor/analysis/api/docs.md` from v0.2 → v0.3 by:
    - Adding a compact error mapping table (-32600/400, -32603/502-504, -32001/429 with retry-after, -32002/401-403, -32010 domain).
    - Cross-linking to citations in `errors.md`, `transport.md`, `proxy-session.md` for each row.
    - Ensuring header casing guidance has both write/read citations and matches examples.
    - Keeping examples conceptual; no source edits.

- Prepare Delta Audit (do not modify this snapshot):
  - Create a fresh worktree of the Shadowcat repo at latest `main` in a sibling directory (example commands below). Do NOT rebase or alter `shadowcat-cursor-review/`.
  - Record the new worktree path and commit hash in `reviews/cursor/tracker.md` and add “Addendum” stubs to each relevant analysis doc noting that delta findings will be appended with separate citations.
  - Example commands (run in the Shadowcat repo, not this snapshot):
    - Ensure submodule is fetched: `git submodule update --init --recursive`
    - From the Shadowcat repo: `git worktree add ../shadowcat-delta main`
    - Verify: `git -C ../shadowcat-delta rev-parse --short HEAD`
    - Baseline checks: `cargo -C ../shadowcat-delta test`, `cargo -C ../shadowcat-delta clippy --all-targets -- -D warnings`
  - Do not commit code; only update analysis docs in this Tapwire repo.

After Phase C (next session or following):
- Run the delta audit against the new worktree and append “Addendum” sections to each C.* doc with new citations. Preserve existing citations to `eec52c8` for stability.

Success criteria
- `analysis/api/docs.md` updated to v0.3 with an error mapping table, cross-links, and casing notes.
- “Delta audit prep” instructions captured with verified worktree commands; tracker updated with delta worktree path and commit hash.
- No code changes; only documentation and planning artifacts updated.

Deliverables to update
- reviews/cursor/analysis/api/docs.md (v0.3)
- reviews/cursor/tracker.md (record delta worktree path/hash; move C.4 to complete)
- reviews/cursor/analysis/* (optional “Addendum (Delta)” stubs appended where relevant)

Notes
- Don’t change code; artifacts only.
- Cite files/lines and propose fixes without implementing.
- Maintain citation stability; do not rebase snapshot. A separate worktree will be created post-Phase C for delta analysis.
