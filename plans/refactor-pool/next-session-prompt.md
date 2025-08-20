# Next Session: Testing & Cleanup

## Project Context

Complete the pool refactor by removing the old implementation and ensuring comprehensive testing.

**Project**: Refactor Pool to shadowcat::pool  
**Tracker**: `plans/refactor-pool/refactor-pool-tracker.md`  
**Status**: Core implementation complete; ready for cleanup and testing

## Current Status

### What Has Been Completed
- ✅ Generic pool fully implemented in `shadowcat::pool`
- ✅ ReverseProxyServer fully migrated to new pool
- ✅ StdioUpstream using new pool successfully
- ✅ Basic integration tests exist
- ✅ C.3 (Type aliases) marked as not needed - no backward compatibility required

### What Needs Cleanup
The old `proxy::pool` module still exists and is referenced by:
- `tests/test_pool_reuse_integration.rs`
- `tests/test_stdio_pool_reuse.rs` 
- `tests/test_subprocess_health.rs`
- `examples/test_pool_shutdown.rs`
- `benches/reverse_proxy_latency.rs`

## Your Mission

Complete the refactor by removing old code and ensuring comprehensive testing.

### Priority Tasks (4-5h total)

1. **Fix Metadata.age Bug** (30min)
   - Currently always returns Duration::from_secs(0)
   - Either: Add created_at timestamp to resources and compute real age
   - Or: Remove age field from PoolConnectionMetadata and docs

2. **Cleanup Old Pool Code** (1h)
   - Remove `src/proxy/pool.rs` module entirely
   - Update `src/proxy/mod.rs` to remove the pool module export
   - Migrate 5 files: `grep -r "proxy::pool" --include="*.rs"`
     - tests/test_stdio_pool_reuse.rs
     - tests/test_pool_reuse_integration.rs  
     - tests/test_subprocess_health.rs
     - examples/test_pool_shutdown.rs
     - benches/reverse_proxy_latency.rs

3. **E.1: Expand Unit Tests** (1.5h)
   - Add targeted stress tests:
     - 100-500 concurrent acquires/drops (no deadlock/starvation)
     - Pool exhaustion with/without close() 
     - Timeout error surfaces correctly under maxed semaphore
     - Hook failure cases: after_create and before_acquire returning Err
     - Verify spawn task overhead in drop path

4. **E.2: Integration Tests** (1.5h)
   - Ensure all integration tests work with new pool
   - Add production scenario tests:
     - Measure p95 acquire latency at various concurrency levels
     - Verify < 5% overhead for stdio echo round-trip
     - Test graceful degradation when pool exhausted
     - Validate close() cancels all pending acquires

5. **E.3: Performance Validation** (1h)
   - Update benches/reverse_proxy_latency.rs to new pool API
   - Add simple stdio "echo" baseline (no pool) for comparison
   - Measure and document:
     - p95 acquire overhead vs no-pool baseline (target: < 5%)
     - Memory per idle connection (target: < 1KB)
     - Throughput degradation under load

### Optional Enhancements (if time permits)
- **D.2**: RAII capacity guard (2h) - if fairness issues found in testing
- **D.3**: Lock-free optimizations (3h) - if contention found in benchmarks

## Essential Commands

```bash
# Find remaining old pool usage
grep -r "proxy::pool" --include="*.rs"

# Run all pool-related tests
cargo test pool

# Run benchmarks
cargo bench pool

# Quality checks
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Full test suite
cargo test --release
```

## Success Criteria

By end of session:
- [ ] Metadata.age bug fixed or field removed
- [ ] Old proxy::pool module completely removed  
- [ ] All 5 files migrated to use shadowcat::pool
- [ ] Stress tests: 100+ concurrent acquires without deadlock
- [ ] Performance: p95 acquire < 1ms, < 5% overhead vs baseline
- [ ] No clippy warnings, all tests green

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
# Main codebase with new pool at src/pool/
# Old pool to remove at src/proxy/pool.rs
```

## Notes

- Since backward compatibility isn't needed, we can make clean breaking changes
- Focus on production readiness through thorough testing
- Document any performance findings for future reference
- Consider if D.2/D.3 enhancements are needed based on test results

---

**Session Focus**: Cleanup & Testing  
**Estimated Duration**: 4-5 hours  
**Last Updated**: 2025-08-20