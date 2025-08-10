# C.3 — Error Handling and Result Flows (Prep)

Scope: `shadowcat-cursor-review/` at `eec52c8`

- Libraries
  - Uses `thiserror` for typed errors in library code and `anyhow` in binaries/tests.
  - `tracing` for contextual logs; some places use `expect()` in non-critical tests only.

- Initial observations
  - Error propagation is largely consistent; consider adding `Context` from `anyhow` in hot operational paths for better diagnostics.
  - Map transport/proxy errors to user-facing messages with consistent codes in CLI.

- Early proposals
  - Consolidate error-to-HTTP status mapping and to-CLI exit codes; ensure no sensitive details leak.
