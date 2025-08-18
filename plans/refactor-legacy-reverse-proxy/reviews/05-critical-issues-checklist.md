# Critical Issues Checklist

## 🔴 MUST FIX Before Merge

### Resource Leaks
- [ ] **Connection Pool Leak** (`upstream/pool.rs:56-60`)
  - Connections silently lost when return channel fails
  - Will cause pool exhaustion under load
  - **Fix**: Ensure cleanup in Drop impl with spawned task

- [ ] **Untracked Spawned Tasks** (`server.rs:106,160,504`)
  - 3 duplicate TapeRecorder init tasks
  - No JoinHandles stored for cleanup
  - **Fix**: Store handles and abort in Drop

- [ ] **Missing Drop Implementation** (`server.rs`)
  - No cleanup on server shutdown
  - Resources leak on restart
  - **Fix**: Implement Drop to cleanup all resources

### Performance Killers
- [ ] **Stdio Process Spawning** (`upstream/stdio.rs:87-106`)
  - New subprocess per request (!!!)
  - 10ms overhead per request
  - **Fix**: Implement real connection reuse

- [ ] **Duplicate State Creation** (`server.rs:121,174,248,318,476`)
  - AppState created 3+ times
  - 3x memory overhead
  - **Fix**: Single create_app_state() method

- [ ] **SSE Double Buffering** (`streaming/raw.rs`, `streaming/intercepted.rs`)
  - 2x memory per SSE connection
  - No buffer pooling
  - **Fix**: Restore buffer pools, single buffer chain

### Missing Critical Features
- [ ] **SSE Reconnection Not Implemented** (`streaming/intercepted.rs:292`)
  - Connections drop permanently on network issues
  - Breaks real-time features
  - **Fix**: Implement with exponential backoff

- [ ] **No Request Timeouts** (all upstream impls)
  - Requests can hang forever
  - Resource exhaustion risk
  - **Fix**: Add timeout configuration

## 🟡 High Priority Issues

### Functionality Gaps
- [ ] **Health Checks Always True** (`upstream/mod.rs`)
- [ ] **Load Balancing Incomplete** (`upstream/selector.rs`)
- [ ] **Metrics Not Collected** (`upstream/http/client.rs`)
- [ ] **Circuit Breaker Missing** (all upstreams)
- [ ] **Request Retries Not Implemented** (despite config field)

### Test Coverage Loss
- [ ] **Admin Endpoint Tests Removed** (~300 lines)
- [ ] **Rate Limiting Tests Removed** (~265 lines)
- [ ] **Integration Tests Simplified** (lost edge cases)

## Quick Validation Tests

```bash
# Test for connection pool leak
for i in {1..1000}; do curl http://localhost:8080/test & done
# Check: Memory should stay stable

# Test stdio spawning overhead  
time for i in {1..100}; do curl http://localhost:8080/stdio-endpoint; done
# Check: Should complete in <5 seconds (not 100+ seconds)

# Test SSE reconnection
# Start SSE connection, kill upstream, check if reconnects
curl -N http://localhost:8080/sse &
kill <upstream-pid>
# Check: Connection should reconnect, not drop

# Test resource cleanup
for i in {1..10}; do
  # Start server
  ./shadowcat reverse --config test.yaml &
  PID=$!
  sleep 2
  kill $PID
done
# Check: No file descriptor leaks, stable memory
```

## Performance Regression Limits

**DO NOT MERGE IF:**
- [ ] P95 latency regression > 5%
- [ ] Memory usage increase > 10%  
- [ ] Throughput reduction > 5%
- [ ] Stdio requests take > 20ms
- [ ] SSE connections drop without reconnection

## Files to Review Most Carefully

1. `src/proxy/reverse/upstream/pool.rs` - Connection leak
2. `src/proxy/reverse/upstream/stdio.rs` - Process spawning
3. `src/proxy/reverse/server.rs` - Resource management
4. `src/proxy/reverse/upstream/http/streaming/intercepted.rs` - SSE reconnection
5. `tests/e2e_complete_flow_test.rs` - Removed test coverage

## Sign-off Checklist

### Code Review
- [ ] All critical issues addressed
- [ ] Performance benchmarks pass
- [ ] No resource leaks detected
- [ ] Tests restored or justified

### Testing
- [ ] Load test: 1000 concurrent connections for 1 hour
- [ ] Memory leak test: valgrind clean
- [ ] Stdio performance: <20ms per request
- [ ] SSE stability: 100 connections for 1 hour

### Documentation
- [ ] Breaking changes documented
- [ ] Migration guide provided
- [ ] Admin endpoint alternatives explained
- [ ] Performance impacts noted

### Deployment
- [ ] Rollback plan in place
- [ ] Monitoring alerts configured
- [ ] Feature flag ready (if applicable)
- [ ] Gradual rollout planned

---

**DO NOT MERGE** until all 🔴 items are checked!