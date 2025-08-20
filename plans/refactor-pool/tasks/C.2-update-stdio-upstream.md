# Task C.2: Update stdio upstream to use new pool

## Objective
Replace reverse proxy stdio upstream’s use of the legacy proxy pool with the new generic `shadowcat::pool::Pool<T>` for `PoolableOutgoingTransport`.

## Background
The new pool provides better correctness (cancel-aware close, SQLx-style hooks) and clarity. Reverse proxy currently depends on `src/proxy/pool.rs`. We will adapt the stdio path to `shadowcat::pool` with minimal churn.

## Key Questions to Answer
1. Integration path: use in-branch module (`shadowcat::pool`) with no feature flag — decided.
2. Options mapping: map reverse proxy pool settings to `PoolOptions` sensible defaults.
3. Shutdown sequencing: call `pool.close().await` on server shutdown.
4. Test strategy: keep tests deterministic (avoid flakiness with short timeouts).

## Step-by-Step Process

### 1. Analysis (20 min)
- Locate stdio upstream acquisition sites (reverse/upstream/stdio.rs) and pool construction (reverse/server.rs AppState).
- Confirm `PoolableOutgoingTransport` implements the required trait methods for the new pool.

```bash
# Commands to understand current state
cd {working_directory}
{command to explore codebase}
{command to find relevant patterns}
```

### 2. Design (20 min)
- Define `type OutgoingPool = shadowcat::pool::Pool<PoolableOutgoingTransport>`.
- Select PoolOptions: `max_connections`, `acquire_timeout`; keep idle/lifetime defaults initially.
- Hooks disabled for pilot (can add later if needed).

### 3. Implementation (60–90 min)
- Create `OutgoingPool` in AppState instead of legacy pool for stdio.
- Replace `ConnectionPool<PoolableOutgoingTransport>::acquire` with `pool::Pool<..>::acquire(factory)`.
- Factory: build `SubprocessOutgoing`, connect, wrap in `PoolableOutgoingTransport`.
- Ensure shutdown path calls `pool.close().await`.

### 4. Testing (45–60 min)
```bash
# Commands to test implementation
cargo test reverse -- --nocapture
cargo clippy --all-targets -- -D warnings
cargo fmt
```

Test cases to implement:
- [ ] Stdio upstream acquire/send/receive path works with new pool.
- [ ] Pool closes on server shutdown; no leaked processes.
- [ ] Acquire cancel behaves as expected when server shuts down.

### 5. Documentation (15 min)
- Update tracker status and add a brief note in reverse proxy docs if needed.

## Expected Deliverables

### New Files
- None expected.

### Modified Files
- `shadowcat/src/proxy/reverse/upstream/stdio.rs` — switch to new pool in acquire path.
- `shadowcat/src/proxy/reverse/server.rs` — construct and close new pool in AppState.

### Tests
- Adapt existing reverse stdio tests; add one deterministic case for acquire/return.
- All tests passing; no clippy warnings.

### Documentation
- Rustdoc comments for all public APIs
- Usage examples in module documentation
- Updated README if applicable

## Success Criteria Checklist

- [x] Stdio path uses `shadowcat::pool` in reverse proxy.
- [x] Tests pass; clippy clean.
- [x] Clean shutdown closes idle resources.
- [x] Tracker updated (C.2 complete).

## Risk Assessment

| Risk | Impact | Mitigation | 
|------|--------|------------|
| {Risk description} | {HIGH/MEDIUM/LOW} | {How to mitigate} |
| {Another risk} | {Impact level} | {Mitigation strategy} |

## Duration Estimate

**Total: 2–3 hours**
- Analysis: 20 min
- Design: 20 min
- Implementation: 60–90 min
- Testing: 45–60 min
- Documentation: 15 min

## Dependencies

- C.1 new pool present in repo (complete)

## Integration Points

- `reverse/upstream/stdio.rs` acquire path
- `reverse/server.rs` AppState construction + shutdown

## Performance Considerations

- Fairness preserved; hook overhead disabled initially.

## Notes

- No feature flag; work happens on refactor branch.

**Task Status**: ✅ Complete
**Last Modified**: 2025-08-20

## Commands Reference

```bash
# Quick reference of useful commands for this task
cd {working_directory}

# Development
{dev_command_1}
{dev_command_2}

# Testing
{test_command_1}
{test_command_2}

# Validation
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo test --quiet
```

## Example Implementation

```rust
// Optional: Provide a concrete example of the expected implementation
// This helps guide the implementation and serves as documentation
{example_code}
```

## Follow-up Tasks

After completing this task, consider:
- {Potential improvement}
- {Related task that could be done}
- {Optimization opportunity}

---

**Task Status**: ⬜ Not Started
**Created**: {Date}
**Last Modified**: {Date}
**Author**: {Author}