# Task D.3: SSE Reconnection Testing & Polish

**Duration**: 2 hours  
**Dependencies**: D.0, D.1, D.2 complete  
**Status**: ⬜ Not Started

## Objective
Comprehensive testing of SSE reconnection features, performance validation, metrics addition, and documentation updates.

## Key Questions to Answer
1. Does reconnection work reliably under various failure modes?
2. What's the performance impact of event tracking?
3. Are the metrics sufficient for monitoring in production?
4. Is the behavior well-documented for users?

## Process

### 1. Integration Tests (1 hour)
```bash
# Create test file
touch tests/integration/reverse_proxy_sse_reconnect.rs
```

Test cases to implement:
- `test_upstream_disconnect_reconnect` - Upstream drops, auto-reconnects
- `test_client_reconnect_with_last_event_id` - Client resumes correctly
- `test_duplicate_event_filtering` - No duplicates after reconnect
- `test_exponential_backoff_timing` - Verify backoff algorithm
- `test_health_monitor_idle_detection` - Stalled connections detected
- `test_concurrent_reconnections` - Multiple clients reconnecting

### 2. Performance Testing (0.5 hours)
```bash
# Create benchmark
touch benches/sse_reconnection.rs
```

Benchmarks:
- Event tracking overhead (operations/sec)
- Memory usage with N tracked events
- Reconnection latency
- Throughput with/without resilience

Expected targets:
- < 1% overhead for event tracking
- < 100ms reconnection time (local)
- < 1MB memory per 1000 events

### 3. Metrics Implementation (0.25 hours)
```rust
// Metrics to add
sse_reconnection_attempts_total
sse_reconnection_success_total
sse_reconnection_failure_total
sse_events_deduplicated_total
sse_client_reconnections_total
sse_upstream_connection_duration_seconds
sse_last_event_id_cache_hit_ratio
```

### 4. Documentation Updates (0.25 hours)
Update documentation:
- `docs/sse-resilience.md` - How it works
- `docs/configuration.md` - New SSE options
- API changelog - Note the enhancement
- README - Mention SSE resilience feature

## Deliverables

### Test Files
1. **New**: `tests/integration/reverse_proxy_sse_reconnect.rs`
   - Comprehensive integration tests
   - Failure simulation helpers
   - Assertion utilities

2. **New**: `benches/sse_reconnection.rs`
   - Performance benchmarks
   - Memory profiling
   - Comparison with/without resilience

### Metrics
- Add to `src/metrics/mod.rs`
- Export via `/metrics` endpoint
- Include in health checks

### Documentation
- User-facing docs on SSE resilience
- Configuration reference
- Troubleshooting guide
- Architecture notes

## Success Criteria
- [ ] All integration tests pass
- [ ] Performance within acceptable bounds
- [ ] Metrics exported and visible
- [ ] Documentation complete and clear
- [ ] No regressions in existing tests
- [ ] Works with real MCP Inspector

## Commands to Run
```bash
# Run integration tests
cargo test --test reverse_proxy_sse_reconnect

# Run benchmarks
cargo bench sse_reconnection

# Check metrics endpoint
cargo run -- reverse --bind 127.0.0.1:8080 --upstream http://localhost:3000
curl http://localhost:8080/metrics | grep sse_

# Test with MCP Inspector
# Manual testing with connection drops
```

## Manual Testing Checklist

### Upstream Failure
1. Start proxy with SSE upstream
2. Start streaming SSE to client
3. Kill upstream server
4. Verify proxy attempts reconnection
5. Restart upstream
6. Verify stream resumes without duplicates

### Client Reconnection
1. Connect client to SSE endpoint
2. Note last event ID received
3. Disconnect client
4. Reconnect with Last-Event-Id header
5. Verify no duplicate events
6. Verify stream continues

### Performance Under Load
1. Connect 100 clients
2. Stream 1000 events/second
3. Randomly disconnect/reconnect clients
4. Monitor memory and CPU usage
5. Check for memory leaks

## Notes
- Use docker-compose for test environment
- Consider adding chaos testing
- Document failure scenarios and recovery
- Add dashboards for new metrics
- Consider load testing with k6 or similar