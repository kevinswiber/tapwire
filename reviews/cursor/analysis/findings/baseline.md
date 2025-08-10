# Baseline Findings — Build/Lint/Test and Workloads

- Context: `shadowcat-cursor-review/` at commit `eec52c8` (perf(mcp): optimize event ID generator for high throughput)
- Formatting: `cargo fmt --all --check` — PASSED
- Lints: `cargo clippy --all-targets -- -D warnings` — PASSED
- Tests: `cargo test` — PASSED
  - Unit tests: 613 passed
  - Integration tests: 124 passed (25 + 26 + 24 + 22 + 4 + 4 + 4 + 5 + 10)
  - Doc-tests: 1 passed, 4 ignored
  - Overall: 738 passed; 0 failed
- Notes:
  - Event ID prefix handling previously flagged is fixed upstream (uses `strip_prefix`); not re-flagged.
  - Perf workload classes and E2E scenarios defined; see `../perf/workloads.md`.
- Suggested next focus (Phase B):
  - Unsafe/FFI audit, cancellation safety, and locking analysis (see session prompt).
