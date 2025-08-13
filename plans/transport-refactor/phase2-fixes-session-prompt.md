# Transport Refactor Phase 2 Fixes - Priority 1 Stability Improvements

## Session Status Update
✅ **COMPLETED**: All Priority 0 (Critical) fixes have been successfully implemented:
- Drop implementations for resource cleanup (verified by rust-code-reviewer)
- Mutex patterns reviewed (no deadlocks found)
- Connection counting verified as thread-safe
- All 839 tests passing, zero clippy warnings

## Current Focus: Priority 1 (Stability Improvements)

### Tasks for This Session

#### 1. Add Buffer Size Limits
**Issue**: Unbounded buffers can cause OOM
**Files**: `shadowcat/src/transport/raw/sse.rs`, `shadowcat/src/transport/raw/stdio.rs`
- Enforce `RawTransportConfig::max_message_size` consistently
- Add backpressure using bounded channels (already partially implemented)
- Prevent unbounded buffer growth in SSE transport
- Check streamable_http.rs for similar issues

#### 2. Implement Timeout Handling
**Issue**: Missing timeouts in network operations
**Files**: All transport files in `shadowcat/src/transport/raw/`
- Wrap async operations with `tokio::time::timeout`
- Use existing timeout fields from `RawTransportConfig`:
  - `connect_timeout`
  - `read_timeout`
  - `write_timeout`
- Pattern to follow:
```rust
use tokio::time::timeout;

match timeout(self.config.read_timeout, operation).await {
    Ok(Ok(result)) => // handle success
    Ok(Err(e)) => // handle operation error  
    Err(_) => // handle timeout - return TransportError::Timeout
}
```

#### 3. Improve Error Context
**Issue**: Some errors lack context for debugging
**Files**: All transport files
- Replace bare `unwrap_or(None)` patterns
- Add context using `.context("descriptive message")?`
- Ensure all TransportError variants have meaningful messages
- Look for error paths that silently fail

#### 4. Add Integration Tests for Concurrent Scenarios
**Location**: `shadowcat/tests/`
- Test concurrent connections with resource limits
- Test Drop implementations under load
- Test timeout scenarios
- Test buffer overflow protection
- Test graceful degradation

## Key Documents
- **Fix Plan**: `@plans/transport-refactor/phase2-review-fix-plan.md` (updated with progress)
- **Review**: `@plans/transport-refactor/reviews/phase2-review.md` (Score: 7.5/10)
- **Tracker**: `@plans/transport-refactor/transport-refactor-tracker.md`

## Testing Commands
```bash
cd shadowcat
cargo clippy --all-targets -- -D warnings  # Must pass
cargo test                                  # All tests must pass
cargo test transport -- --nocapture         # Focus on transport tests
cargo test --test transport_regression_suite # Regression tests
```

## Definition of Done for Priority 1
- [ ] Buffer limits enforced in all transports (check with large message tests)
- [ ] Timeouts implemented for all async operations
- [ ] All errors have proper context (no silent failures)
- [ ] New integration tests for concurrent scenarios added
- [ ] All existing tests still pass
- [ ] No new clippy warnings
- [ ] Document any behavior changes

## Implementation Tips
1. **Buffer Limits**: The config already has `max_message_size` - ensure it's checked before accepting data
2. **Timeouts**: Create a helper method `with_timeout()` to reduce code duplication
3. **Error Context**: Use `tracing::error!` for unexpected errors before returning
4. **Tests**: Consider using `proptest` for property-based testing of limits

## Next Session (Priority 2 - Performance)
After completing Priority 1, we'll focus on:
- Buffer pooling using existing `buffer_pool.rs`
- Zero-copy optimizations with `bytes::BytesMut`
- Performance benchmarks
- Documentation updates

## Notes
- Keep changes focused and incremental
- Test after each change to catch regressions early
- Remember: This is a git submodule - commit in shadowcat first
- The goal is production-ready transport layer with proper resource management