# Cursor Comprehensive Rust Code Review — Analysis Workspace

This directory stores research and analysis artifacts for the comprehensive Shadowcat/Tapwire Rust code review driven by `CURSOR_RUST_CODE_REVIEWER.md`.

## Structure
- `findings/` — Deep‑dive notes per subsystem (transport, proxy, session, recorder, interceptor, auth, metrics). Includes `baseline.md` for fmt/clippy/tests.
- `perf/` — Microbench notes, flamegraphs, and performance hypothesis docs (see `perf/workloads.md`).
- `safety/` — Unsafe/FFI audits, invariants, and proofs (see `safety/unsafe-audit.md`).
- `async/` — Cancellation safety, locking analyses, task lifecycles, shutdown diagrams (see `async/cancellation.md`, `async/locking.md`).
- `api/` — Public API assessments, trait boundaries, and ergonomics.
- `tests/` — Coverage notes, gaps, and proposed test plans.

Create subfolders/files as needed. Keep artifacts incremental and reference concrete file paths.

## Process
- Do not perform code edits here. This area is for notes and results.
- Use code citations (`start:end:path`) to point directly to relevant code.
- Link back to tracker tasks in `../tracker.md`.
- Keep `NEW_SESSION_PROMPT_CURSOR.md` updated when major milestones are reached.

## References
- Reviewer guide: `./../../CURSOR_RUST_CODE_REVIEWER.md`
- Project rules: `./../../CLAUDE.md`
- Plans template: `./../../plans/tracker-template.md`
