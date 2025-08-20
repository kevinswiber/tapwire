# Next Session: Phase B/C — Pilot Integration

## Project Context

Extract the generic connection pool into `shadowcat::pool`, align with sqlx patterns, and keep migration churn low.

**Project**: Refactor Pool to shadowcat::pool  
**Tracker**: `plans/refactor-pool/refactor-pool-tracker.md`  
**Status**: Phase D enhancements complete; ready for pilot integration

## Current Status

### What Has Been Completed
- Generic pool implemented in `shadowcat-connection-pooling/src/pool` with:
  - Cancel-aware close event; `acquire()` races shutdown
  - SQLx-style hooks: `after_create`, `before_acquire`, `after_release` (+ metadata)
  - Idle/lifetime cleanup; fairness (release after requeue)
  - Module docs and unit tests (reuse, cleanup, fairness, close-cancel, hooks)

### What's Next (Pilot)
- Decide integration path:
  - Option 1: Add pool crate as dependency in `shadowcat` and introduce a feature-gated stdio pilot path (no behavior change by default).
  - Option 2: Move finalized pool module into `shadowcat/src/pool` and adapt stdio upstream incrementally.

- Implement stdio adapter (pilot):
  - Define `PoolableOutgoingResource` that implements `pool::traits::PoolableResource` wrapping `Box<dyn OutgoingTransport>`.
  - Factory connects `SubprocessOutgoing` and returns wrapped resource.
  - Use `pool::Pool<PoolableOutgoingResource>::acquire()` in a pilot function/path.

- Validation tasks:
  - Add a minimal integration test (behind a feature) verifying acquire/send/receive/return.
  - Observe hook behavior (e.g., `before_acquire` idle gating) with a simple counter or ping.

## Your Mission

Focus on analysis deliverables to de-risk design/implementation.

### Priority 1: Implement stdio pilot (2–3h)
- Use `shadowcat::pool::Pool<PoolableOutgoingTransport>` in reverse/stdio upstream.
- Construct pool in AppState; call `pool.close().await` on shutdown.

### Priority 2: Tests and docs (1h)
- Add/adapt a deterministic integration test for stdio with new pool; brief docs.

## Essential Context Files to Read

1. **Primary Tracker**: `plans/refactor-pool/refactor-pool-tracker.md`
2. **Findings**: `plans/refactor-pool/analysis/findings.md`
3. **Design Decisions**: `plans/refactor-pool/analysis/design-decisions.md`
4. **Pool Implementation**: `shadowcat-connection-pooling/src/pool/*`

## Working Directory

```bash
cd /Users/kevin/src/tapwire
# Worktree for code lives in shadowcat-connection-pooling (branch: refactor/pool)
# Plan docs live in plans/refactor-pool
```
