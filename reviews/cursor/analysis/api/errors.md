# C.3 — Error Handling and Result Flows (Prep)

Scope: `shadowcat-cursor-review/` at `eec52c8`

- Libraries
  - Uses `thiserror` for typed errors in library code and `anyhow` in binaries/tests.
  - `tracing` for contextual logs; some places use `expect()` in non-critical tests only.

- Initial observations
  - Error propagation is largely consistent; consider adding `Context` from `anyhow` in hot operational paths for better diagnostics.
  - Map transport/proxy errors to user-facing messages with consistent codes in CLI.
  - Reverse proxy maps errors to JSON-RPC-shaped error bodies with HTTP status via IntoResponse; codes mostly -32600/-32603.
    ```1366:1392:shadowcat-cursor-review/src/proxy/reverse.rs
    impl IntoResponse for ReverseProxyError { /* maps to status + JSON error */ }
    ```

- Early proposals
  - Consolidate error-to-HTTP status mapping and to-CLI exit codes; ensure no sensitive details leak.
  - Add a small error taxonomy doc enumerating mapping of internal errors to JSON-RPC codes and HTTP statuses; ensure protocol violations use -32600 and upstream failures -32603.
  - Encourage `anyhow::Context` on external IO boundaries (HTTP/stdio) for actionable logs.
