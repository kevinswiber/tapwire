# Cursor Comprehensive Rust Code Review — Analysis Workspace

This directory stores research and analysis artifacts for the comprehensive Shadowcat/Tapwire Rust code review driven by `CURSOR_RUST_CODE_REVIEWER.md`.

## Structure
- `findings/` — Deep‑dive notes per subsystem (transport, proxy, session, recorder, interceptor, auth, metrics). Includes `baseline.md` for fmt/clippy/tests.
- `perf/` — Microbench notes, flamegraphs, and performance hypothesis docs (see `perf/workloads.md`).
- `safety/` — Unsafe/FFI audits, invariants, and proofs (see `safety/unsafe-audit.md`).
- `async/` — Cancellation safety, locking analyses, task lifecycles, shutdown diagrams (see `async/cancellation.md`, `async/locking.md`).
- `api/` — Public API assessments, trait boundaries, and ergonomics. See:
  - `api/transport.md` — Transport defaults, timeouts, size limits, shutdown, header casing.
  - `api/proxy-session.md` — Forward/reverse lifecycle, interceptor effects, recording accuracy, metrics.
  - `api/errors.md` — Error taxonomy and HTTP/JSON‑RPC mappings with guidance for IO boundary context.
  - `api/docs.md` — Public API overview, examples, and guidance (current version v0.3).
 - `security/` — Security analyses and guidance.
- `tests/` — Coverage notes, gaps, and proposed test plans.

Create subfolders/files as needed. Keep artifacts incremental and reference concrete file paths.

## Process
- Do not perform code edits here. This area is for notes and results.
- Use code citations (`start:end:path`) to point directly to relevant code.
- Link back to tracker tasks in `../tracker.md`.
- Keep `NEW_SESSION_PROMPT_CURSOR.md` updated when major milestones are reached.

### Current Focus
- Wrap-up: See `../conclusion.md` for executive summary and next-step recommendations.

### Quickstart (Delta Audit Prep)
- Create a fresh worktree for Shadowcat main without touching this snapshot:
  - Ensure submodules: `git submodule update --init --recursive`
  - From Shadowcat: `git worktree add ../shadowcat-delta main`
  - Verify commit: `git -C ../shadowcat-delta rev-parse --short HEAD`
  - Baseline checks: `cargo -C ../shadowcat-delta test`, `cargo -C ../shadowcat-delta clippy --all-targets -- -D warnings`
  - Record path/hash in `../tracker.md` and add “Addendum (Delta)” stubs to relevant `api/*.md` docs.

## References
- Reviewer guide: `./../../CURSOR_RUST_CODE_REVIEWER.md`
- Project rules: `./../../CLAUDE.md`
- Plans template: `./../../plans/tracker-template.md`
