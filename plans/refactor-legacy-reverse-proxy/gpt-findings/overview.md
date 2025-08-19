# GPT Findings: Phase H Plan Review

Date: 2025-08-19

## Verdict (Second Pass)

The Phase H plan remains directionally correct and maps well to the review’s priorities. After reading all task and review documents and re-checking current code, two critical surgical items are still not explicit in the plan and not yet implemented in code: (1) moving the pool return `mpsc::Receiver` ownership into the maintenance task, and (2) updating `Subprocess` health semantics so `is_connected()` becomes false on stdout EOF/process exit. Without these, stdio reuse will likely continue to fail even if other pool improvements land.

## Key Observations

- The pool fixes (H.0/H.1) should explicitly include the receiver ownership change in the maintenance loop and subprocess health semantics; otherwise, reuse may still fail. Current code still guards the receiver with `Arc<Mutex<_>>` and keeps `connected = true` on EOF.
- SSE reconnection (H.4) needs concrete reconnection policy (backoff, jitter, caps) and tests to avoid regressions.
- Success criteria include valgrind-based leak testing, which is impractical in typical Rust/Tokio workflows; propose more actionable alternatives.
- The plan calls out performance targets but lacks a lightweight, repeatable benchmark script and clear acceptance metrics per task.
