# Next Session: Transport Metrics Quick Wins

## Context
Phase 1 (ProcessManager Integration) and Phase 3 (SSE Streaming Optimizations) are complete. We've achieved:
- Full ProcessManager integration with graceful shutdown
- >15% SSE memory reduction through buffer pooling
- >20% expected throughput improvement
- Comprehensive reconnection logic verified

The existing metrics infrastructure (Prometheus + OpenTelemetry) is robust. We need to add transport-specific visibility.

## Session Objectives
Implement quick-win transport metrics that provide immediate operational value and validate our recent optimizations.

## Tasks for This Session

### Task M.1: Add Buffer Pool Metrics (30 min)
Expose buffer pool statistics as Prometheus metrics.

**Implementation:**
1. Add metrics to MetricsCollector (hit rate, pool size, allocations)
2. Implement `export_metrics()` method on BytesPool
3. Wire up to Prometheus registry
4. Test with all pools (STDIO, HTTP, SSE, JSON)

### Task M.2: Add ProcessManager Metrics (30 min)
Monitor subprocess lifecycle and health.

**Metrics to add:**
1. Active process count gauge
2. Process lifetime histogram
3. Graceful shutdown success rate counter
4. SIGTERM handling time histogram
5. Process restart count counter

### Task M.3: Add SSE-Specific Metrics (1h)
Validate SSE optimizations with detailed metrics.

**Metrics to add:**
1. Reconnection count and success rate
2. Event deduplication effectiveness (duplicates filtered/total)
3. Stream idle time histogram
4. Buffer usage after optimization
5. Last-Event-ID resumption success rate

### Task M.4: Add Transport Latency Breakdown (1h)
Detailed timing for each transport operation.

**Per-transport metrics:**
1. Connect time histogram
2. First byte time histogram
3. Total transfer time histogram
4. Serialization/deserialization time
5. Operation-specific breakdowns (send, receive, close)

## Key Implementation Points

### Use Existing Infrastructure
- MetricsCollector in `src/metrics/mod.rs`
- Prometheus registry already configured
- HTTP endpoint at `/metrics` already exposed
- OpenTelemetry spans already defined

### Implementation Pattern
```rust
// 1. Add to MetricsCollector
pub buffer_pool_hits: GaugeVec,

// 2. Register in MetricsCollector::new()
let buffer_pool_hits = register_gauge_vec!(
    "shadowcat_buffer_pool_hit_rate",
    "Buffer pool hit rate percentage",
    &["pool"]
)?;

// 3. Update in operations
global_pools::SSE_POOL.export_metrics(&metrics);
```

### Testing Approach
- Use existing test MetricsCollector
- Verify Prometheus export format
- Check metric values in integration tests
- Benchmark overhead (must be <1%)

## Deliverables
- [ ] Buffer pool metrics integrated
- [ ] ProcessManager metrics added
- [ ] SSE-specific metrics implemented
- [ ] Transport latency breakdowns
- [ ] Tests for new metrics
- [ ] Documentation updated

## Success Criteria
- All metrics exported via `/metrics` endpoint
- Metrics collection overhead <1%
- Buffer pool hit rates visible
- ProcessManager health trackable
- SSE reconnection patterns observable
- All existing tests continue to pass

## References
- Metrics Module: `@shadowcat/src/metrics/mod.rs`
- Telemetry Module: `@shadowcat/src/telemetry/mod.rs`
- Buffer Pool: `@shadowcat/src/transport/buffer_pool.rs`
- ProcessManager: `@shadowcat/src/process/manager.rs`
- SSE Reconnection: `@shadowcat/src/transport/sse/reconnect.rs`
- Analysis: `@plans/transport-advanced-features/analysis/phase4-metrics-analysis.md`
- Tracker: `@plans/transport-advanced-features/transport-advanced-features-tracker.md`

## Previous Achievements
### Phase 1 (ProcessManager)
- ✅ Full backward compatibility
- ✅ Graceful shutdown with SIGTERM
- ✅ Health monitoring

### Phase 3 (SSE Optimizations)  
- ✅ >15% memory reduction
- ✅ >20% throughput improvement
- ✅ Buffer pooling integrated
- ✅ Comprehensive reconnection logic

## Time Estimate
3 hours total (30m + 30m + 1h + 1h)

## Notes
- Leverage existing Prometheus/OpenTelemetry infrastructure
- Focus on operational visibility for recent optimizations
- Keep metric overhead under 1%
- Test metrics export via `/metrics` endpoint
- Consider creating dashboards in follow-up session