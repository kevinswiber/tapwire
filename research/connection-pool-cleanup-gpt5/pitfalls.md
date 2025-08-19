# Pitfalls & Gotchas (from SQLx and general practice)

- Async in Drop: Rust lacks async Drop; doing work in Drop that requires awaits leads to half-cleanups or aborts. Prefer explicit async shutdown and document it.
- Background tasks keeping pools alive: Spawn tasks with `Weak` refs and a close event; avoid strong refs in tasks which cause leaks.
- Channel backpressure on return: Always handle the case where the return path is full/closed; close connections explicitly to avoid leaks.
- Health mis-signaling: Ensure transports mark themselves disconnected when streams close; avoid reusing dead connections.
- Await while holding locks: Avoid; collect work items, release lock, then perform async checks/closures.
- Interval immediate tick: `tokio::time::interval` triggers an immediate tick; absorb the first tick to avoid biased select loops.
- Parent/child quotas: If ever needed, manage permits carefully and return them in Drop of the inner state only.
- Test harness realities: Simulate load, backpressure, shutdown races; rely on deterministic timers or timeouts in tests to avoid flakiness.

