# Next Session: SSE Streaming Optimizations

## Context
Phase 1 (ProcessManager Integration) is complete with full backward compatibility, graceful shutdown, and comprehensive testing. The transport layer now has robust subprocess lifecycle management. We're ready to optimize SSE streaming performance.

## Session Objectives
Profile and optimize SSE streaming performance, focusing on buffer management, memory usage, and reconnection reliability.

## Tasks for This Session

### Task S.1: Profile SSE Performance Bottlenecks (1h)
Identify optimization targets in the SSE transport implementation.

**Sub-tasks:**
1. Profile memory usage during SSE streaming
2. Analyze buffer allocation patterns
3. Identify hot paths in message parsing
4. Measure reconnection overhead

### Task S.2: Implement SSE Buffering Improvements (2h)
Optimize buffer usage and reduce memory overhead.

**Sub-tasks:**
1. Implement buffer pooling for SSE events
2. Optimize string allocations in parser
3. Add lazy parsing for large messages
4. Reduce intermediate allocations

### Task S.3: Add SSE Reconnection Logic (1h)
Improve SSE connection reliability and recovery.

**Sub-tasks:**
1. Implement exponential backoff for reconnects
2. Add connection health monitoring
3. Handle partial message recovery
4. Add metrics for connection drops

## Key Implementation Points

### Performance Targets
- Reduce SSE memory usage by >15%
- Improve throughput by >20%
- Minimize allocation overhead
- Maintain message ordering guarantees

### Buffer Optimization
- Use global buffer pools (already available)
- Implement zero-copy where possible
- Reuse allocations across messages
- Consider streaming parser improvements

### Reconnection Strategy
- Exponential backoff with jitter
- Track last event ID for resumption
- Configurable retry limits
- Health check intervals

## Deliverables
- [ ] Performance profile report
- [ ] Optimized SSE buffer management
- [ ] Enhanced reconnection logic
- [ ] Benchmark comparisons
- [ ] Unit and integration tests
- [ ] Documentation of improvements

## Success Criteria
- SSE memory usage reduced by >15%
- Streaming performance improved by >20%
- No message loss during reconnects
- All existing tests continue to pass
- Benchmarks show measurable improvements

## References
- SSE Implementation: `@shadowcat/src/transport/sse/`
- Buffer Pool: `@shadowcat/src/transport/buffer_pool.rs`
- Raw SSE Transport: `@shadowcat/src/transport/raw/sse.rs`
- Tracker: `@plans/transport-advanced-features/transport-advanced-features-tracker.md`

## Phase 1 Achievements (Completed)
- ✅ ProcessManager integration with full backward compatibility
- ✅ Graceful shutdown with SIGTERM support (5s timeout)
- ✅ Optional ProcessManager injection via `with_process_manager()`
- ✅ Health monitoring with `process_status()` and `is_healthy()`
- ✅ Comprehensive test coverage
- ✅ All tests passing (223 transport tests)
- ✅ No clippy warnings

## Time Estimate
4 hours total (1h profiling + 2h optimization + 1h reconnection)

## Alternative: Phase 2 (Batch Support)
If SSE optimization is not priority, can work on:
- Batch Message Support (Tasks B.1-B.4)
- Full JSON-RPC batch request handling
- Protocol layer batch support

## Notes
- Profile first, optimize based on data
- Consider impact on existing SSE users
- Maintain backward compatibility
- Document performance improvements clearly