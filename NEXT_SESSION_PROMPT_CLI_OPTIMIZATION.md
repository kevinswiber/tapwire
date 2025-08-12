# Next Session Prompt - Shadowcat CLI Optimization Continuation

## Context

You're continuing the Shadowcat CLI optimization project. The project is located in a git worktree at `/Users/kevin/src/tapwire/shadowcat-cli-refactor/`.

**Current Progress: 57% Complete** (47 of 73 hours completed)

## What Was Just Completed: Phase C.4 - Telemetry and Metrics

Successfully implemented comprehensive observability with zero overhead when disabled:

### Implemented Components
1. **OpenTelemetry Tracing** (`src/telemetry/mod.rs`)
   - OTLP export support for Jaeger/Tempo
   - Configurable sampling rate
   - Span builders for common operations
   - Graceful shutdown handling

2. **Prometheus Metrics** (`src/metrics/mod.rs`)
   - HTTP endpoint at configurable path
   - Comprehensive metrics: request_count, request_duration, active_sessions, errors_total
   - Circuit breaker states, connection pool sizes, rate limit hits
   - Duration timer helper for easy instrumentation

3. **Integration** 
   - Initialized in main.rs based on configuration
   - Instrumented proxy operations in `src/proxy/forward.rs`
   - Added tracing to session manager methods
   - Created `examples/telemetry_demo.rs` showing Jaeger integration

### Configuration Added
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

## Your Next Mission - Choose One

### Option 1: C.3 - Improve Error Messages (2 hours) [Quick Win]
Enhance user-facing error messages:
- Add context to transport errors
- Improve proxy error descriptions
- Add suggestions for common mistakes
- Include recovery hints

### Option 2: C.5 - Performance Optimization (6 hours) [High Impact]
Optimize for production loads:
- Profile with flamegraph
- Reduce allocations in hot paths
- Optimize buffer sizes
- Implement connection pooling
- Target: < 5% latency overhead

### Option 3: C.6 - Extensive Test Coverage (6 hours) [Quality]
Achieve 70%+ test coverage:
- Add unit tests for uncovered code
- Create property-based tests
- Add stress tests
- Implement fuzz testing for parsers

### Option 4: C.7 - CLI Shell Completions (2 hours) [UX]
Add shell completion support:
- Bash completions
- Zsh completions
- Fish completions
- PowerShell completions

## Project Status Summary

### Completed Phases
- **Phase A**: Critical Fixes (100% - 7 hours) ✅
- **Phase B**: Library Readiness (100% - 24 hours) ✅
- **Phase B.7**: Code Review Fixes (100% - 5 hours) ✅
- **Phase C**: Quality & Testing (30% - 11 of 37 hours)
  - C.1: Documentation ✅
  - C.2: Configuration Files ✅
  - C.4: Telemetry/Metrics ✅
  - C.8: Example Programs ✅

### Remaining Work (26 hours)
- C.3: Improve Error Messages (2h)
- C.5: Performance Optimization (6h)
- C.6: Extensive Test Coverage (6h)
- C.7: CLI Shell Completions (2h)
- C.9: Connection Pooling (3h)
- C.10: Load Testing (2h)
- C.11: Release Preparation (2h)

## Key Implementation Notes

### Dependencies Added Today
```toml
# OpenTelemetry
opentelemetry = { version = "0.27", features = ["trace", "metrics"] }
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio", "trace", "metrics"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic", "trace", "metrics"] }
tracing-opentelemetry = "0.28"

# Metrics
prometheus = "0.13"
axum-prometheus = "0.7"
```

### Code Style Reminders
1. **Always run before committing**: `cargo clippy --all-targets -- -D warnings`
2. **Format strings**: Use `{var}` not `{}", var`
3. **IO errors**: Use `std::io::Error::other(e)` not `::new(::Other, e)`
4. **Unused items**: Prefix with `_` or remove

### Testing with Telemetry
```bash
# Start Jaeger
docker run -d -p 16686:16686 -p 4317:4317 jaegertracing/all-in-one:latest

# Run the demo
cargo run --example telemetry_demo

# View traces at http://localhost:16686
# View metrics at http://localhost:9090/metrics
```

## Important Files to Reference

1. **Project Tracker**: `plans/cli-refactor-optimization/cli-optimization-tracker.md`
2. **Configuration Schema**: `src/config/schema.rs`
3. **Telemetry Module**: `src/telemetry/mod.rs`
4. **Metrics Module**: `src/metrics/mod.rs`
5. **Example Config**: `shadowcat.example.toml`

## Success Criteria for Next Task

Whichever task you choose:
1. All existing tests must continue passing
2. No new clippy warnings
3. Update relevant documentation
4. Add tests for new functionality
5. Update the tracker in `plans/cli-refactor-optimization/cli-optimization-tracker.md`

## Recommended Next Steps

1. **If you want quick impact**: Do C.3 (Error Messages) - 2 hours
2. **If you want performance**: Do C.5 (Performance Optimization) - 6 hours
3. **If you want quality**: Do C.6 (Test Coverage) - 6 hours
4. **If you want UX improvement**: Do C.7 (Shell Completions) - 2 hours

## Context Management

- Current context usage: ~60% (after telemetry implementation)
- If approaching 85%, create a new session
- Focus on one task at a time to manage context

## Commands to Run

```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Build and test
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test

# Run specific examples
cargo run --example telemetry_demo
cargo run --example simple_library_usage

# Check current metrics
curl http://localhost:9090/metrics  # After starting a server with metrics enabled
```

Start by choosing which Phase C task to tackle based on project priorities!