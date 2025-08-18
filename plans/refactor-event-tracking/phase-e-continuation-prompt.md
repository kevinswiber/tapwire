# 🔴 CRITICAL: Phase E - Fix Event Tracking Performance Issues

## Context
I've completed Phases A-C of event tracking consolidation in Shadowcat. Architecture review revealed **SEVERE PERFORMANCE ISSUES** that make the system unsuitable for production:

### Critical Problems Found
1. **Task Explosion**: Spawning 1000+ tasks/second under load (`src/session/manager.rs:961-974`)
2. **Silent Failures**: All persistence errors ignored (data loss risk)
3. **Memory Inefficiency**: 20KB per session (should be < 5KB)
4. **No Backpressure**: Unbounded task growth possible

## Your Mission: Execute Phase E (6.5 hours)

**THIS MUST BE COMPLETED BEFORE ANY PRODUCTION DEPLOYMENT**

### Task E.1: Implement Worker Pattern (3 hours) - 🔴 CRITICAL
**Location**: `src/session/manager.rs:961-974`

Replace fire-and-forget task spawning with bounded worker:
1. Create PersistenceWorker with bounded channel (1000 capacity)
2. Single worker task per SessionManager
3. Batch persistence operations (50 events or 100ms flush)
4. Add proper error handling with exponential backoff retries
5. Implement comprehensive monitoring metrics

**Key Metrics to Add**:
- `persistence_queue_depth` gauge
- `persistence_batch_size` histogram  
- `persistence_latency_seconds` histogram
- `persistence_success_total` counter
- `persistence_failure_total` counter

Read full task: `plans/refactor-event-tracking/tasks/E.1-implement-worker-pattern.md`

### Task E.2: Fix Activity Tracking (1.5 hours) - 🟡 HIGH
**Location**: `src/transport/sse/session.rs:377-398`

Eliminate task spawning for activity updates:
1. Extend worker pattern from E.1
2. Batch activity updates (HashSet for deduplication)
3. Use channels instead of spawning tasks
4. Add activity-specific metrics

**Key Metrics to Add**:
- `activity_updates_batched` histogram
- `activity_queue_depth` gauge
- `activity_coalesce_ratio` gauge (efficiency metric)

Read full task: `plans/refactor-event-tracking/tasks/E.2-fix-activity-tracking.md`

### Task E.3: Optimize Memory Usage (2 hours) - 🟡 HIGH

Reduce memory overhead by 75%:
1. Switch from String to Arc<str> for event IDs
2. Implement string interning with LRU cache
3. Add LRU eviction for sessions
4. Optimize data structures

**Key Metrics to Add**:
- `memory_bytes_per_session` histogram
- `string_intern_hit_ratio` gauge
- `session_evictions_total` counter

Target: < 5KB per session (from 20KB)

Read full task: `plans/refactor-event-tracking/tasks/E.3-optimize-memory-usage.md`

## Testing Commands

```bash
# After E.1 - Test worker pattern
cargo test session::persistence_worker
cargo bench event_persistence

# After E.2 - Test activity tracking  
cargo test session::activity_batching

# After E.3 - Test memory optimization
cargo test session::memory_usage
cargo bench memory_overhead

# Load test (MUST PASS)
cargo test test_load_1000_events_per_second -- --nocapture
```

## Success Metrics

### Before Phase E (CURRENT - BROKEN)
- Task spawn rate: 1000+/second 🔴
- Memory per session: 20KB 🟡
- Error handling: NONE 🔴
- Backpressure: NONE 🔴

### After Phase E (TARGET)
- Task spawn rate: < 1/second ✅
- Memory per session: < 5KB ✅
- Error handling: Full retry logic ✅
- Backpressure: Bounded channels ✅
- Full metrics/monitoring ✅

## Files to Reference

- **Tracker**: `plans/refactor-event-tracking/refactor-event-tracking-tracker.md`
- **Critical Analysis**: `plans/refactor-event-tracking/analysis/critical-architecture-review.md`
- **Phase A-C Completion**: Already merged to main

## Production Readiness Checklist

**DO NOT DEPLOY UNTIL ALL ITEMS ARE COMPLETE:**

- [ ] Worker pattern implemented (E.1)
- [ ] Task spawn rate < 10/second
- [ ] Error handling with retries
- [ ] Bounded channels for backpressure
- [ ] Memory usage < 5KB/session
- [ ] Load test passes (1000 events/sec)
- [ ] All metrics exposed and integrated with telemetry
- [ ] OpenTelemetry spans for tracing
- [ ] Alerts configured for key metrics

## Important Notes

- Focus on E.1 first - it's the most critical fix
- All phases can share the worker infrastructure from E.1
- Consider using `tokio-metrics` to monitor task spawning in tests
- Phase D (integration testing) is deferred until after Phase E
- All metrics should integrate with existing telemetry infrastructure
- Use OpenTelemetry spans to trace batch processing

## Starting Point

1. Read the critical architecture review
2. Start with E.1 - implement the worker pattern
3. Run tests frequently to ensure no regressions
4. Monitor task spawn rate using `tokio-console` or similar

The system is currently **unsuitable for production** and will fail under moderate load without these fixes.