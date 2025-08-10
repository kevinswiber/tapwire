Session: Phase D kickoff — Performance, Recording, Interceptors

Repo/context
- Working dir: tapwire/
- Shadowcat snapshot (stable citations): shadowcat-cursor-review/ @ eec52c8 — analysis-only; DO NOT modify this snapshot
- Shadowcat delta worktree (latest main): shadowcat-delta/ @ b793fd1 — read-only for analysis; DO NOT commit code here
- Scope: Analysis-only. Update performance analysis artifacts under `reviews/cursor/**`. Preserve existing `eec52c8` citations; add `shadowcat-delta/` citations where useful. No source edits.

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
- Completed Delta Audit addenda for Phase C docs; added exact citations to `shadowcat-delta@b793fd1`
- Tracker updated with Delta Audit checklist marked complete

Tasks (Phase D)
- D.1 Hot-path allocation and logging audit
  - Identify high-frequency paths (transports, reverse/forward proxy loops) with unnecessary allocations/clones and verbose logging in tight loops.
  - Search patterns: `clone\(`, `to_string\(`, `format!\(`, `serde_json::to_string`, frequent `tracing::debug!`/`info!` inside loops.
  - Capture start:end:path citations and recommendations in `reviews/cursor/analysis/perf/hot-paths.md`.

- D.2 Recorder overhead and memory usage
  - Inspect `recorder/` modules and `SessionManager::record_frame` call sites for synchronous IO, large in-memory buffers, locking granularity, and duplication.
  - Note opportunities for streaming, batching, and reducing copies; add citations to `reviews/cursor/analysis/perf/recorder.md`.

- D.3 Interceptor chain performance
  - Analyze `interceptor/` (e.g., rules engine, file watcher) for per-message overhead, locking, and metrics emission cost.
  - Document hotspots and suggestions (e.g., precompiled predicates, bounded async work) in `reviews/cursor/analysis/perf/interceptors.md` with citations.

- Tracker update
  - In `reviews/cursor/tracker.md`, add a short Delta Audit section with a checklist for the above, and record any notable deviations from the Phase C taxonomy or guidance

Suggested commands (analysis only)
- Build/lint/tests
  - cargo test -q --manifest-path shadowcat-delta/Cargo.toml
  - cargo clippy --manifest-path shadowcat-delta/Cargo.toml --all-targets -- -D warnings
- Grep targets
  - rg "clone\(|to_string\(|format!\(" shadowcat-delta/src -n
  - rg "tracing::(debug|info)!\(" shadowcat-delta/src -n
  - rg "record_frame\(|TapeRecorder" shadowcat-delta/src -n
  - rg "Interceptor|RuleBasedInterceptor|engine" shadowcat-delta/src -n
- Optional (local profiling)
  - cargo bench --manifest-path shadowcat-delta/Cargo.toml
  - cargo flamegraph (see `CLAUDE.md` for usage)

Success criteria
- `reviews/cursor/analysis/perf/hot-paths.md` updated with top hotspots, citations, and recommendations
- `reviews/cursor/analysis/perf/recorder.md` updated with recorder/storage overhead analysis and proposals
- `reviews/cursor/analysis/perf/interceptors.md` updated with interceptor-chain performance findings
- `reviews/cursor/tracker.md` updated to reflect Phase D task statuses

Deliverables to update
- reviews/cursor/analysis/perf/hot-paths.md
- reviews/cursor/analysis/perf/recorder.md
- reviews/cursor/analysis/perf/interceptors.md
- reviews/cursor/tracker.md (Phase D checklist/status)

Notes
- Do not edit source code in either worktree during analysis
- Maintain citation stability: use shadowcat-cursor-review@eec52c8 for baseline and shadowcat-delta@b793fd1 for delta
- Prefer exact start:end:path citations; keep examples conceptual without code edits