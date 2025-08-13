# Task C.5: Performance Optimization

## Overview
Optimize Shadowcat's performance to ensure production readiness with < 5% latency overhead vs direct connection.

## Duration
6 hours

## Dependencies
- Phase B complete (builders, shutdown, API)
- All tests passing
- No clippy warnings

## Objectives
1. Profile current implementation to identify hot paths
2. Reduce allocations in message processing
3. Optimize buffer sizes for transports
4. Implement connection pooling for HTTP
5. Achieve < 5% latency overhead target

## Deliverables

### 1. Performance Profiling Setup
- [ ] Install and configure flamegraph
- [ ] Create benchmark suite for baseline measurement
- [ ] Document profiling methodology

### 2. Baseline Benchmarks
- [ ] Measure direct connection latency
- [ ] Measure current proxy overhead
- [ ] Identify top 5 hot paths from flamegraph
- [ ] Memory allocation analysis

### 3. Transport Optimizations
- [ ] Optimize stdio buffer sizes (currently using default)
- [ ] Reduce allocations in message parsing
- [ ] Implement zero-copy where possible
- [ ] HTTP connection pooling (if not already present)

### 4. Message Processing Optimizations
- [ ] Reduce cloning in hot paths
- [ ] Use `Cow<str>` where appropriate
- [ ] Optimize JSON parsing/serialization
- [ ] Cache frequently accessed data

### 5. Memory Optimizations
- [ ] Use `Arc` instead of cloning large data
- [ ] Implement object pools for frequent allocations
- [ ] Reduce intermediate Vec allocations
- [ ] Optimize session storage

### 6. Performance Validation
- [ ] Re-run benchmarks after optimizations
- [ ] Verify < 5% overhead target achieved
- [ ] Document performance improvements
- [ ] Create performance regression tests

## Success Criteria
- [ ] < 5% latency overhead in p95 benchmarks
- [ ] Reduced memory allocations in hot paths (measured)
- [ ] Connection pooling implemented for HTTP transport
- [ ] All existing tests still passing
- [ ] No new clippy warnings
- [ ] Performance improvements documented with measurements

## Implementation Steps

### Step 1: Set Up Profiling Tools
```bash
# Install flamegraph
cargo install flamegraph

# Install cargo-bloat for binary size analysis
cargo install cargo-bloat

# Install criterion for benchmarking
# Add to Cargo.toml dev-dependencies if not present
```

### Step 2: Create Benchmark Suite
Create `benches/proxy_performance.rs`:
- Direct connection baseline
- Forward proxy overhead
- Reverse proxy overhead
- Message parsing performance
- Session management overhead

### Step 3: Profile Current Implementation
```bash
# Generate flamegraph
cargo flamegraph --bench proxy_performance

# Analyze binary size
cargo bloat --release

# Memory profiling with heaptrack or valgrind
```

### Step 4: Implement Optimizations
Based on profiling results:
- Focus on top 5 hot paths
- Reduce allocations
- Optimize buffer sizes
- Add connection pooling

### Step 5: Validate Improvements
- Re-run benchmarks
- Compare before/after metrics
- Ensure target met

## Notes
- Use `criterion` for reliable benchmarking
- Consider using `perf` on Linux for detailed CPU profiling
- HTTP connection pooling may overlap with C.9 task
- Keep backwards compatibility while optimizing
- Document any API changes needed for performance

## References
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Flamegraph Repository](https://github.com/flamegraph-rs/flamegraph)