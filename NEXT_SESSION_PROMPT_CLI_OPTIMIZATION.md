# Next Session: Phase C.4 - Telemetry and Metrics Implementation

## Context
You are continuing the Shadowcat CLI optimization refactor. We have successfully completed:
- **Phase A**: All critical fixes (100% complete)
- **Phase B**: Library readiness (100% complete) 
- **Phase B.7**: Code review fixes (100% complete)
- **Phase C Progress**: 
  - C.1: Documentation (✅ Complete)
  - C.2: Configuration File Support (✅ Complete)
  - C.8: Example Programs (✅ Complete)

## Current Task: C.4 - Add Telemetry/Metrics (4 hours)

### Objective
Implement OpenTelemetry tracing and Prometheus metrics to provide production-grade observability for Shadowcat.

### Key Files to Reference
1. **Tracker**: `plans/cli-refactor-optimization/cli-optimization-tracker.md`
2. **Task Details**: `plans/cli-refactor-optimization/tasks/C.4-telemetry.md`
3. **Configuration Schema**: `src/config/schema.rs` (already has telemetry/metrics config sections)
4. **Example Config**: `shadowcat.example.toml` (has telemetry/metrics examples)

### Implementation Requirements

#### 1. OpenTelemetry Tracing
- Add `opentelemetry`, `opentelemetry-otlp`, and `tracing-opentelemetry` dependencies
- Create `src/telemetry/mod.rs` with initialization logic
- Instrument key functions with `#[tracing::instrument]`
- Support OTLP exporter configuration from config file
- Key spans to add:
  - Proxy request/response lifecycle
  - Session creation/destruction
  - Transport operations
  - Interceptor chain processing

#### 2. Prometheus Metrics
- Add `prometheus` and `axum-prometheus` dependencies (or `metrics` crate)
- Create `src/metrics/mod.rs` with metric definitions
- Add `/metrics` endpoint to reverse proxy
- Key metrics to track:
  - Request count and latency histograms
  - Active sessions gauge
  - Transport errors counter
  - Rate limiting rejections counter
  - Connection pool statistics

#### 3. Integration Points
- Initialize telemetry in `main.rs` based on configuration
- Add telemetry context to `Shadowcat` struct
- Pass spans through transport layers
- Export metrics from session manager, rate limiter, etc.

#### 4. Configuration
The configuration schema is already in place:
```toml
[telemetry]
enabled = false
otlp_endpoint = "http://localhost:4317"
service_name = "shadowcat"
sampling_rate = 0.1

[metrics]
enabled = false
bind = "127.0.0.1:9090"
path = "/metrics"
```

### Success Criteria
- [ ] OpenTelemetry tracing working with Jaeger/Tempo
- [ ] Prometheus metrics exposed and scrapeable
- [ ] Configuration from file working
- [ ] No performance impact when disabled
- [ ] Example demonstrating telemetry usage
- [ ] All tests passing, no clippy warnings

### Implementation Order
1. Add dependencies to Cargo.toml
2. Create telemetry module with initialization
3. Create metrics module with definitions
4. Integrate with main.rs and api.rs
5. Add instrumentation to key functions
6. Test with local Jaeger and Prometheus
7. Create example showing telemetry usage
8. Update documentation

### Testing
- Unit tests for metric collection
- Integration test with mock OTLP collector
- Manual testing with Jaeger UI
- Performance benchmark with/without telemetry

### Notes from Previous Session
- Configuration system is fully implemented and tested
- All configuration structs have proper validation
- SQLite database directory creation is handled in SessionManagerBuilder
- Code is clippy-clean with all tests passing

### Commands to Run
```bash
# Navigate to the refactor branch
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Run tests
cargo test

# Check for clippy warnings
cargo clippy --all-targets -- -D warnings

# Test with example config
cargo run -- --config shadowcat.example.toml forward stdio -- echo
```

### Potential Challenges
1. **Async context propagation** - Ensure spans are properly propagated across async boundaries
2. **Performance overhead** - Minimize impact when telemetry is disabled
3. **Cardinality explosion** - Be careful with label values in metrics
4. **Graceful degradation** - Handle OTLP endpoint unavailability

### Next Steps After C.4
After completing telemetry/metrics, the next priority tasks are:
- C.3: Improve Error Messages (2h) - User-friendly formatting
- C.9: Connection Pooling (3h) - HTTP transport optimization
- C.5: Performance Optimization (6h) - Profile and optimize hot paths

## Summary
Implement comprehensive observability for Shadowcat using OpenTelemetry for distributed tracing and Prometheus for metrics. The configuration infrastructure is already in place, so focus on the implementation and integration aspects.
EOF < /dev/null