# MCP Server Architecture Analysis & Refactoring Guide

## Overview

This directory contains the comprehensive analysis and refactoring plan for the MCP server implementation, focusing on achieving production-grade performance and correctness through hyper pattern adoption and async best practices.

## Key Findings

Our analysis revealed significant opportunities for optimization:
- **80% reduction in task spawns** (from 5 to 1 per connection)
- **Elimination of async antipatterns** (locks across await, polling loops, unbounded channels)
- **Proper notification support** via SSE/WebSocket for MCP's server-to-client requirements
- **Memory savings** of 8-32MB per 1000 connections

## Document Guide

### 1. [spawn-audit.md](spawn-audit.md)
**Current State Analysis**
- Comprehensive audit of all `tokio::spawn` usage
- Comparison with hyper's efficient patterns
- Identifies duplicate spawns and unnecessary tasks
- Quantifies performance impact and memory overhead

**Read this first** to understand the current problems and motivation for refactoring.

### 2. [server-architecture.md](server-architecture.md)
**Target Architecture Design**
- Complete production-ready server implementation
- Incorporates all hyper patterns and async best practices
- Addresses both spawn reduction and correctness issues
- Includes proper transport handling for notifications
- Week-by-week implementation roadmap

**Primary implementation reference** - this is the blueprint for refactoring.

### 3. [sse-implementation.md](sse-implementation.md)
**SSE Implementation Details**
- Specific guidance for Server-Sent Events with hyper v1
- StreamBody pattern for efficient streaming
- Connection configuration for long-lived streams
- Production considerations and browser compatibility

**Reference during Week 3** when implementing notification transport.

## Implementation Roadmap

### Phase 1: Critical Fixes (Week 1)
Start with correctness issues that could cause production failures:

1. **Fix Lock-Across-Await Issues** [server-architecture.md#lock-hygiene]
   - Never hold RwLock/Mutex across await points
   - Extract data before async operations

2. **Replace Polling with Select** [server-architecture.md#async-patterns]
   - Change `try_recv()` + sleep to `recv().await` + `select!`
   - Eliminate busy-wait antipattern

3. **Atomic Connection Limits** [server-architecture.md#connection-management]
   - Replace count check with Semaphore
   - Eliminate race conditions

### Phase 2: Hyper Integration (Week 2)
Adopt hyper's proven patterns:

1. **Single Spawn per Connection** [server-architecture.md#serve-pattern]
   - Use `serve_connection` with `service_fn`
   - Eliminate handler task spawning

2. **Remove Duplicate Spawns** [spawn-audit.md#duplicate-spawns]
   - Centralize connection driving in transport layer
   - Remove redundant connection layer spawns

3. **Consolidate HTTP Transport** [server-architecture.md#transport-layer]
   - Remove worker thread pattern
   - Use channels for coordination

### Phase 3: Notification Support (Week 3)
Implement server-to-client push:

1. **SSE Integration** [sse-implementation.md]
   - StreamBody for efficient event streaming
   - Proper headers and connection configuration

2. **WebSocket Support** [server-architecture.md#websocket-handling]
   - Upgrade detection and handoff
   - Full duplex when needed

3. **Transport Abstraction** [server-architecture.md#connection-transport]
   - Unified interface for HTTP/SSE/WebSocket
   - Seamless notification delivery

### Phase 4: Production Hardening (Week 4)

1. **Graceful Shutdown** [server-architecture.md#shutdown]
   - CancellationToken propagation
   - JoinSet for task tracking
   - Timeout-based cleanup

2. **Error Handling** [server-architecture.md#error-handling]
   - Proper JSON-RPC error codes
   - Connection resilience
   - Structured logging

3. **Performance Tuning** [sse-implementation.md#performance-tips]
   - TCP_NODELAY for low latency
   - Buffer pool optimization
   - HTTP/2 preference

## Quick Reference

### Hyper Patterns to Adopt
```rust
// Single spawn per connection
tokio::spawn(async move {
    http1::Builder::new()
        .serve_connection(io, service_fn(handler))
        .await
});
```

### Async Best Practices
```rust
// Never hold locks across await
let data = {
    let guard = lock.read().await;
    guard.clone()  // Extract data
}; // Lock released
data.async_operation().await;  // Safe to await
```

### SSE Streaming
```rust
// Efficient streaming with backpressure
let (tx, rx) = mpsc::channel(100);
let stream = ReceiverStream::new(rx);
let body = StreamBody::new(stream).boxed();
```

## Testing Strategy

### Unit Tests
- Mock transports for spawn counting
- Verify single task per connection
- Test graceful shutdown paths

### Integration Tests
```bash
# Test SSE notifications
cargo test --test sse_integration

# Test connection limits
cargo test test_max_clients_atomic

# Test graceful shutdown
cargo test test_shutdown_no_deadlock
```

### Performance Validation
```bash
# Benchmark before/after
cargo bench spawn_overhead

# Memory profiling
heaptrack cargo run --example load_test
```

## Success Metrics

- [ ] Task spawns reduced from 5 to 1 per connection
- [ ] No locks held across await points
- [ ] Zero race conditions in connection management
- [ ] SSE/WebSocket notifications working
- [ ] Graceful shutdown completes in <5s
- [ ] Memory usage <100KB per session
- [ ] Latency overhead <5% p95

## Migration Notes

### Breaking Changes
- Session structure changes for transport abstraction
- Handler trait requires async notification support
- Connection builder API simplified

### Compatibility
- Wire protocol unchanged (MCP 2025-11-05)
- Client connections work identically
- Existing interceptors remain compatible

## Questions?

Refer to the specific documents for detailed implementation guidance. The architecture is designed to be implemented incrementally - each week's work provides value even if later phases are delayed.

Priority order:
1. **Correctness** (Week 1) - Fix critical bugs
2. **Efficiency** (Week 2) - Reduce resource usage
3. **Features** (Week 3) - Add notification support
4. **Polish** (Week 4) - Production readiness

Start with `spawn-audit.md` to understand the problems, then use `server-architecture.md` as your implementation guide.