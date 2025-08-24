# Hyper-Util Legacy Pool vs Shadowcat Pool (GPT‑5 Analysis)

Date: 2025-08-24
Author: GPT‑5 code review and performance analysis
Scope: Compare Shadowcat’s generic pool (shadowcat/src/pool) with Hyper’s legacy client pool (hyper-util) and propose targeted improvements without sacrificing Shadowcat’s multi‑transport requirements (HTTP, stdio, WS).

## Executive Summary

Shadowcat’s pool is clean, protocol‑agnostic, and correctness‑oriented. It adds valuable features (hooks, graceful close, maintenance, deterministic shutdown) that a minimalist HTTP pool does not. The largest performance gap vs hyper’s legacy pool stems from two design choices:

- Asynchronous return path in `Drop` that always spawns a task before requeueing.
- Mandatory async health checks on both acquire and return paths.

These are fixable. With a synchronous fast path for typical returns, a recency‑based health policy, and a hotter idle reuse strategy (LIFO), Shadowcat can close most of the perf gap while preserving features and safety. Additional wins are available by improving wakeup/handoff semantics and reducing lock overhead on the idle list.

## What Hyper’s Legacy Pool Optimizes For

While we cannot import the source in this environment, Hyper’s legacy pool is well‑known to favor:
- LIFO reuse of idle connections to keep cache‑hot sockets in the hot path and reduce tail latency.
- Synchronous return to pool (no spawn in `Drop`) by avoiding awaits on the release path.
- Minimal bookkeeping in the hot path (no hooks/health by default, HTTP‑specific lifecycle ensures unhealthy sockets fail during IO rather than on release).
- Targeted keying (per authority or connector key) and direct waiter handoff to reduce rescheduling and improve fairness.

These choices trade feature richness for throughput and predictability in an HTTP‑only setting.

## What Shadowcat’s Pool Optimizes For Today

Observed from `shadowcat/src/pool`:
- Hooks: `after_create`, `before_acquire`, `after_release` (SQLx‑style) for lifecycle customization across protocols.
- Maintenance: periodic idle cleanup + health checks; weak‑backed task that doesn’t keep pool alive.
- Deterministic shutdown: `close()` with notify + event listener for racing acquires; best‑effort cleanup in `Drop`.
- Capacity enforcement: semaphore permit acquired before acquisition logic; only released after requeue completes on drop.
- Idle structure: `Mutex<VecDeque<…>>` with `pop_front`/`push_back` (FIFO reuse).
- Health checks: awaited in `pop_idle_healthy()` and again on return path prior to requeue.

These yield good safety and clarity across arbitrary resources, but introduce overheads absent in Hyper’s pool.

## Critical Findings

- Synchronous return path missing
  - `PoolConnection<T>::drop()` unconditionally spawns an async task that awaits `is_healthy()` and optionally a hook before pushing back to idle and only then drops the permit. Under churn, this creates many short‑lived tasks and adds scheduling latency to every release.
  - Releasing the permit only after requeue guarantees capacity invariants (good), but it also extends the critical path for the next waiter.

- Double async health checks on the hot path
  - Idle candidate is health‑checked again during acquire (`pop_idle_healthy`) and also on release. For protocols where health failures manifest during IO (HTTP request/response), this is wasted work for recently‑used connections.

- FIFO reuse of idle connections
  - Using `pop_front` + `push_back` biases reuse to the oldest idle resource. Hyper favors LIFO to maximize locality and reduce the chance of handing out a cold/stale connection.

- Idle lock choice and granularity
  - `tokio::sync::Mutex` protects a fast, non‑awaited push/pop. An async mutex is not strictly needed here; the code already avoids holding the lock across awaits. A non‑async mutex (e.g., `parking_lot::Mutex`) can reduce lock/unlock overhead and cross‑task wakeups.

- Return‑path fan‑out
  - Without the optional `bounded-return-executor` feature, returns spawn unbounded tasks. In high‑RPS scenarios with many short requests, this is a real source of allocator and scheduler pressure.

- Metrics available but not wired
  - `pool/metrics.rs` defines useful counters/gauges, but the pool doesn’t update them. This blocks data‑driven tuning (e.g., measuring reuse rate or async vs sync return ratio after optimizations).

## Targeted Improvements (Ranked)

1) Add a synchronous fast‑path in `Drop` (highest impact)
- Goal: In the common case (pool open, recently used resource, no hooks), perform a synchronous push to idle and release the permit immediately, without spawning.
- Approach:
  - Introduce a non‑await health heuristic on the trait (opt‑in):
    - `fn is_likely_healthy(&self) -> bool { true }` with a conservative default.
    - Implement cheaply for resources that can tell (HTTP2 connected state, WS open flag, stdio child alive flag).
  - In `Drop`:
    - If `inner.hooks.is_none()` and `!inner.is_closed` and `res.is_likely_healthy()`, try a non‑await path:
      - Attempt `idle.try_lock()` (or guardless if using `parking_lot::Mutex`) and `push_back` synchronously.
      - Drop the permit immediately after successfully enqueuing.
      - Fall back to the existing spawned async path if any condition fails (hooks present, lock contended, health uncertain).
- Why safe: Maintenance and acquire‑time validation still protect against true stale resources. The fast path is an optimization, not a semantic change.

2) Make health checks recency‑aware (high impact)
- Goal: Avoid awaiting `is_healthy()` for recently‑used connections, where a failure is unlikely and will be caught by the next IO anyway.
- Approach:
  - In `pop_idle_healthy()`:
    - If `metadata.last_idle_at.elapsed() < RECENT` (e.g., 5s, configurable), skip `await res.is_healthy()` and return immediately.
    - Otherwise, perform the health check as today.
  - Optionally track `last_health_check_at` in metadata and only re‑check after a minimum interval.
- Why: Hyper avoids health checks entirely; this change captures most of that win without eliminating protection against stale idles.

3) Switch idle reuse to LIFO (high impact, low risk)
- Goal: Hand out the most recently returned resource first.
- Approach: Replace `pop_front()` with `pop_back()` while keeping `push_back()`.
- Why: Warmer caches, fewer old connections resurfacing, better p95.

4) Replace `tokio::Mutex` on idle with a non‑async mutex (medium impact)
- Goal: Reduce overhead of locking for short, non‑await critical sections.
- Approach:
  - Use `parking_lot::Mutex<VecDeque<…>>` or `std::sync::Mutex` for `idle` because all push/pop sites avoid `.await` inside the critical section.
  - Carefully audit to ensure no awaited calls while holding the lock (already satisfied).
- Why: Async mutexes incur extra scheduling cost even under light contention; this is an easy micro‑win.

5) Bound the return path (medium impact)
- Goal: Avoid unbounded spawn storms during heavy churn.
- Approach:
  - Enable the `bounded-return-executor` feature by default, or embed a small `Semaphore` in the pool to cap concurrent return tasks (e.g., 64).
  - Consider a small inline SPSC channel to queue return work to a single background worker instead of per‑drop spawns.
- Why: Reduces allocator/scheduler pressure and evens out tail latency.

6) Consider waiter handoff on return (medium impact, design change)
- Goal: When a return occurs while waiters are parked on the semaphore, preferentially hand the resource directly to a waiter (avoiding extra scheduling hops).
- Approach:
  - Maintain a small list of waiting wakers or a `tokio::sync::Notify` used only for idle arrivals to accelerate handoff.
  - Alternative: Split capacity control (permits) from idle availability notification so a waiter can wake immediately when an idle is pushed.
- Why: Hyper’s legacy pool reduces hops by co‑locating waiters with the pool key and waking one waiter when an idle arrives.

7) De‑duplicate health checks on both ends (low‑medium impact)
- Goal: Don’t re‑validate a resource both on return and on acquire.
- Approach:
  - If the return path validated health (async path), tag the metadata as “checked_at = now()”. Acquire can then skip the health check if within `RECENT`.
  - This piggybacks on item 2 without adding much complexity.

8) Wire up metrics (observability enabling)
- Goal: Measure effectiveness of changes before/after.
- Approach:
  - Increment counters in acquire/return paths: total acquires, reuse vs new, async vs sync returns, health‑check skipped/hit, rejections by hooks, idle queue length, etc.
  - Expose a snapshot API or integrate with existing telemetry.

## Subtleties and Trade‑offs

- Permit release timing
  - Today, the permit is released only after the resource is pushed to idle. This prevents a waiter from acquiring a permit and racing into creation while the returning resource hasn’t been enqueued yet. Keep this invariant for correctness and reuse quality. The synchronous fast path should also drop the permit only after the idle push succeeds.

- Hook semantics
  - Hooks are a powerful differentiator. A synchronous fast path should apply only when `hooks.is_none()` or when a hook guarantees a constant‑time, non‑await fast path (rare). Otherwise, keep the current async path to preserve hook behavior.

- Protocol differences
  - HTTP/1.1: failures usually show on next read/write; recency skip is reasonable.
  - HTTP/2: health can be inferred from the state machine without IO; `is_likely_healthy()` can be strong.
  - WebSockets: long‑lived; pooling value is limited. Consider dedicated sessions with no pool, or a max‑idle of 0.
  - stdio: checking the child/process is a cheap health signal (`try_wait()`, pipe state). Implement `is_likely_healthy()` using those signals.

## Concrete Code Pointers (Shadowcat)

- `Drop` spawn on return: `shadowcat/src/pool/mod.rs`, `impl<T> Drop for PoolConnection<T>`
- Acquire path health check: `pop_idle_healthy()`; always awaits `is_healthy()` and uses FIFO.
- Idle lock type: `idle: Mutex<VecDeque<…>>` despite zero awaits in critical sections.
- Metrics: `shadowcat/src/pool/metrics.rs` exists but isn’t referenced from the pool module.

## Suggested Minimal Patches (sketches)

- LIFO reuse and recency skip (acquire side):

```rust
// in pop_idle_healthy()
let maybe = {
    let mut idle = inner.idle.lock().await;
    idle.pop_back() // LIFO instead of pop_front
};
let (mut res, metadata) = maybe?;

// Recency-based fast path
const RECENT: Duration = Duration::from_secs(5);
if metadata.last_idle_at.elapsed() < RECENT {
    return Some((res, metadata)); // skip await is_healthy()
}

// Otherwise check as today
if res.is_healthy().await { return Some((res, metadata)); }
let _ = res.close().await; // discard stale
```

- Synchronous fast path (return side):

```rust
impl<T: PoolableResource + 'static> Drop for PoolConnection<T> {
    fn drop(&mut self) {
        if let (Some(mut res), Some(permit)) = (self.resource.take(), self.permit.take()) {
            let pool = self.pool.clone();
            let created_at = self.created_at;

            // Fast path: only when pool open, no hooks, and resource likely healthy
            if !pool.inner.is_closed.load(Ordering::Acquire)
                && pool.inner.hooks.is_none()
                && res.is_likely_healthy() // new optional trait method with default=true
            {
                if let Ok(mut idle) = pool.inner.idle.try_lock() { // or parking_lot::Mutex no-async
                    idle.push_back((res, ResourceMetadata { created_at, last_idle_at: Instant::now() }));
                    drop(permit); // release capacity only after enqueue
                    return;
                }
            }

            // Fallback to existing async path
            tokio::spawn(async move { /* current logic */ });
        }
    }
}
```

- Idle lock type change:

```rust
// in PoolInner<T>
idle: parking_lot::Mutex<VecDeque<(T, ResourceMetadata)>>,
```

- Trait extension for cheap health heuristic:

```rust
#[async_trait]
pub trait PoolableResource: Send + Sync {
    async fn is_healthy(&self) -> bool;
    async fn close(&mut self) -> Result<()>;
    fn resource_id(&self) -> String;
    fn is_likely_healthy(&self) -> bool { true } // default
}
```

## Validation Plan

- Benchmarks in `shadowcat/benches/*` already exist. Augment with:
  - p50/p95 for hot reuse with and without hooks.
  - Stress: N concurrent acquires/releases with short tasks to magnify return‑path overhead.
  - Toggle `RECENT` threshold and record impact; instrument “health_check_skipped” vs “performed”.
  - Measure scheduler and task counts to verify fewer spawned tasks post‑change.

- Add metrics wiring to observe:
  - `release_sync_count` vs `release_async_count` (fast path hit rate).
  - `health_check_skipped` count.
  - Idle queue length dynamics under load.

## Risks and Mitigations

- Returning a bad connection via the fast path
  - Mitigation: restrict fast path to recent idles and to resources reporting `is_likely_healthy()`. Maintenance and subsequent IO still gate correctness.

- Lock contention on idle with LIFO
  - Push/pop are O(1) either way. LIFO often reduces contention by reducing time a connection spends in idle.

- Hook users regressions
  - Fast path bypasses only when hooks are absent. When hooks are present, behavior remains unchanged.

## Bottom Line

Shadowcat can keep its multi‑protocol, safety‑first semantics and still capture most of Hyper’s pool performance by:
- Reusing the hottest connections (LIFO),
- Avoiding awaits in the common release path (synchronous fast path), and
- Making health checks adaptive to recency.

Add bounded return execution and wire metrics to validate. These are focused, low‑risk changes with outsized payoffs for p95 and throughput.

