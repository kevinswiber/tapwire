# Findings

## Initial Implementation (2025-08-19)

- Implemented a fresh, transport-agnostic pool in `src/pool` on branch `refactor/pool` (worktree: `shadowcat-connection-pooling`).
- Public API: `PoolOptions`, `Pool<T>`, `PoolConnection<T>`, `PoolStats`, and `traits::PoolableResource`.
- Behavior:
  - Acquire uses semaphore + timeout; reuses healthy idle else creates via factory.
  - Weak-backed maintenance with first-tick absorption; periodic cleanup of idle/expired.
  - `close().await` marks closed, stops maintenance, drains and closes idle.
  - Drop (last ref) provides best‑effort idle cleanup (notify; await maintenance up to 5s; close idle).
  - Fair capacity release: permit is released only after resource is requeued to idle.
- Tests added cover reuse, close semantics, idle timeout cleanup, and fairness.

Open items / future enhancements:
- Close event helper implemented: `Pool::close_event()` and acquire now cancels
  promptly when close starts (sqlx-style behavior). Added unit test to verify
  pending acquires resolve with error after `close()` begins.
- Health hooks implemented (SQLx-style):
  - `after_create` for new resources, fail acquire on error.
  - `before_acquire` for idle resources, return false/Err to close-and-retry.
  - `after_release` on drop, false/Err closes instead of requeue.
  Includes `PoolConnectionMetadata { age, idle_for }` and examples.
- (Optional) Lock-free idle queue + atomic counters if profiling shows contention.

## Integration Complete (2025-08-20)

Pilot integration complete:
- ReverseProxyServer fully migrated to use `shadowcat::pool::Pool<OutgoingResource>`
- StdioUpstream successfully uses new pool with factory-based acquisition
- Pool creation mapped from ReverseUpstreamConfig pool settings
- Clean shutdown properly calls `pool.close().await` in multiple paths
- Integration tests added: `test_stdio_new_pool.rs`, `test_stdio_pool_reuse.rs`

## Completed Fixes (2025-08-20)

1. ✅ **Metadata.age fixed**: Now tracks `created_at` and `last_idle_at` properly
   - Added `ResourceMetadata` struct with both timestamps
   - `PoolConnectionMetadata.age` computed from `created_at.elapsed()`
   - `max_lifetime` now correctly enforced based on age, not idle time

2. ✅ **Old pool removed**: All references migrated to `shadowcat::pool`
   - Deleted `proxy::pool` module export
   - Migrated 5 test/bench files to new API
   - Removed 2 redundant test files

3. ✅ **Stress tests added**: Comprehensive concurrency testing
   - 100 concurrent acquires: p95 latency ~113ms (< 1s target ✓)
   - Pool exhaustion timeout verified
   - Metadata age/idle tracking validated
   - Max lifetime enforcement tested
   - Close cancels all pending acquires confirmed

4. ✅ **Performance validated**: Tests show excellent performance
   - Reuse working correctly (10 resources for 100 acquires)
   - No deadlocks or starvation under load
   - Fairness preserved through drop task design

## Architecture Notes
- Actual structure: `pool/mod.rs` + `pool/traits.rs` (not split files as planned)
- Forward proxy doesn't use pooling (only reverse proxy does)
- Migration policy: Breaking changes OK, no deprecation needed
