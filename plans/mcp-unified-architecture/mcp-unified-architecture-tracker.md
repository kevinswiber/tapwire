# MCP Unified Architecture Tracker

## Overview

This tracker coordinates the integration of hyper patterns, session management, and interceptors into both MCP client and server implementations, achieving production-grade performance and correctness. Incorporates comprehensive feedback from Gemini's architectural review.

**Last Updated**: 2025-08-26  
**Total Estimated Duration**: 250-280 hours  
**Status**: Planning

## Goals

1. **Performance Optimization** - Reduce task spawns by 80% using hyper patterns
2. **Session Management Integration** - Unified session tracking for client/server with persistence
3. **Interceptor Chain** - Message processing pipeline for both directions
4. **Production Readiness** - Graceful shutdown, proper error handling, monitoring

## Architecture Vision

```
┌─────────────────────────────────────────────────────────────┐
│                     MCP Unified Architecture                  │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  Client Mode:                                                 │
│  ┌─────────┐    ┌─────────┐    ┌────────────┐    ┌────────┐ │
│  │ Client  │───►│ Session │───►│Interceptor │───►│Transport│ │
│  │  (1 spawn)   │ Manager │    │   Chain    │    │ (Hyper) │ │
│  └─────────┘    └─────────┘    └────────────┘    └────────┘ │
│                       │                                       │
│                  ┌─────────┐                                  │
│                  │ Storage │                                  │
│                  │(SQLite/ │                                  │
│                  │ Redis)  │                                  │
│                  └─────────┘                                  │
│                                                               │
│  Server Mode:                                                 │
│  ┌─────────┐    ┌─────────┐    ┌────────────┐    ┌────────┐ │
│  │Transport│───►│ Session │───►│Interceptor │───►│ Server │ │
│  │ (Hyper) │    │ Manager │    │   Chain    │    │(1 spawn)│ │
│  └─────────┘    └─────────┘    └────────────┘    └────────┘ │
│       │                                              │        │
│  ┌─────────┐                                    ┌─────────┐  │
│  │SSE/WS   │                                    │ Handler │  │
│  │Upgrade  │                                    │         │  │
│  └─────────┘                                    └─────────┘  │
└───────────────────────────────────────────────────────────────┘
```

## Work Phases

### Phase A: Foundation Analysis (Week 1)
Understand current state and design integration approach

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | **Inventory Session & Interceptor Code** | 4h | None | ⬜ Not Started | | [Details](tasks/A.0-inventory-existing-code.md) |
| A.1 | **Design Unified Session Architecture** | 6h | A.0 | ⬜ Not Started | | [Details](tasks/A.1-design-session-architecture.md) |
| A.2 | **Design Interceptor Integration** | 4h | A.0 | ⬜ Not Started | | [Details](tasks/A.2-design-interceptor-integration.md) |
| A.3 | **Create Migration Plan** | 4h | A.1, A.2 | ⬜ Not Started | | [Details](tasks/A.3-create-migration-plan.md) |

**Phase A Total**: 18 hours

### Phase B: Server Refactoring (Week 2)
Apply hyper patterns and async best practices to server

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.0 | **Fix Async Antipatterns** | 8h | A.3 | ⬜ Not Started | | [Details](tasks/B.0-fix-async-antipatterns.md) |
| B.1 | **Implement Hyper serve_connection** | 12h | B.0 | ⬜ Not Started | | [Details](tasks/B.1-hyper-serve-connection.md) |
| B.2 | **Add SSE Support** | 10h | B.1 | ⬜ Not Started | | [Details](tasks/B.2-add-sse-support.md) |
| B.3 | **WebSocket Integration** | 8h | B.1 | ⬜ Not Started | | [Details](tasks/B.3-websocket-integration.md) |
| B.4 | **Graceful Shutdown** | 6h | B.1 | ⬜ Not Started | | [Details](tasks/B.4-graceful-shutdown.md) |

**Phase B Total**: 44 hours

### Phase C: Client Optimization & Session Heartbeat (Week 3)
Apply same patterns to client implementation with proactive liveness detection

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.0 | **Analyze Client Architecture** | 4h | B.1 | ⬜ Not Started | | [Details](tasks/C.0-analyze-client.md) |
| C.1 | **Session Heartbeat Mechanism** | 8h | C.0 | ⬜ Not Started | | [Details](tasks/C.1-session-heartbeat.md) |
| C.2 | **Reduce Client Spawns** | 8h | C.0 | ⬜ Not Started | | [Details](tasks/C.2-reduce-client-spawns.md) |
| C.3 | **Client Connection Pool Integration** | 10h | C.2 | ⬜ Not Started | | [Details](tasks/C.3-client-pool-integration.md) |
| C.4 | **Client SSE Handling** | 6h | C.2 | ⬜ Not Started | | [Details](tasks/C.4-client-sse-handling.md) |

**Phase C Total**: 36 hours

### Phase D: Session Management & Error Handling (Week 4)
Integrate comprehensive session management with robust error handling

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.0 | **Port Session Manager** | 8h | C.2 | ⬜ Not Started | | [Details](tasks/D.0-port-session-manager.md) |
| D.1 | **Session Store Trait** | 6h | D.0 | ⬜ Not Started | | [Details](tasks/D.1-session-store-trait.md) |
| D.2 | **SQLite Implementation** | 8h | D.1 | ⬜ Not Started | | [Details](tasks/D.2-sqlite-store.md) |
| D.3 | **Interceptor Error Handling** | 7h | D.0 | ⬜ Not Started | | [Details](tasks/D.3-interceptor-error-handling.md) |
| D.4 | **Redis Implementation** | 10h | D.1 | ⬜ Not Started | | [Details](tasks/D.4-redis-store.md) |
| D.5 | **SSE Session Tracking** | 6h | D.0, B.2 | ⬜ Not Started | | [Details](tasks/D.5-sse-session-tracking.md) |

**Phase D Total**: 45 hours

### Phase E: API Design & Observability (Week 5)
Add builder pattern API and comprehensive observability

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| E.0 | **Builder Pattern API Design** | 6h | D.0 | ⬜ Not Started | | [Details](tasks/E.0-api-design.md) |
| E.1 | **Port Interceptor Engine** | 8h | E.0 | ⬜ Not Started | | [Details](tasks/E.1-port-interceptor-engine.md) |
| E.2 | **MCP Protocol Interceptor** | 6h | E.1 | ⬜ Not Started | | [Details](tasks/E.2-mcp-interceptor.md) |
| E.3 | **Observability & Metrics** | 6.5h | E.0 | ⬜ Not Started | | [Details](tasks/E.3-observability.md) |
| E.4 | **Rules Engine Integration** | 8h | E.1 | ⬜ Not Started | | [Details](tasks/E.4-rules-engine.md) |
| E.5 | **HTTP Policy Interceptor** | 6h | E.1 | ⬜ Not Started | | [Details](tasks/E.5-http-policy.md) |
| E.6 | **Client Interceptor Chain** | 6h | E.1, C.2 | ⬜ Not Started | | [Details](tasks/E.6-client-interceptors.md) |
| E.7 | **Server Interceptor Chain** | 6h | E.1, B.1 | ⬜ Not Started | | [Details](tasks/E.7-server-interceptors.md) |

**Phase E Total**: 52.5 hours

### Phase F: Testing & Hardening (Week 6)
Comprehensive testing and production readiness

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| F.0 | **Integration Test Suite** | 12h | E.7 | ⬜ Not Started | | [Details](tasks/F.0-integration-tests.md) |
| F.1 | **Performance Benchmarks** | 8h | F.0 | ⬜ Not Started | | [Details](tasks/F.1-performance-benchmarks.md) |
| F.2 | **Load Testing** | 6h | F.0 | ⬜ Not Started | | [Details](tasks/F.2-load-testing.md) |
| F.3 | **Monitoring & Metrics** | 8h | F.0, E.3 | ⬜ Not Started | | [Details](tasks/F.3-monitoring-metrics.md) |
| F.4 | **Documentation** | 8h | F.0 | ⬜ Not Started | | [Details](tasks/F.4-documentation.md) |

**Phase F Total**: 42 hours

### Phase G: Chaos & Security Testing (Week 7)
Chaos engineering, security hardening, and long-running validation

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| G.0 | **Fault Injection & Chaos Testing** | 8.5h | F.0 | ⬜ Not Started | | [Details](tasks/G.0-fault-injection.md) |
| G.1 | **Security Testing & Hardening** | 7h | F.0 | ⬜ Not Started | | [Details](tasks/G.1-security-testing.md) |
| G.2 | **Soak Testing (24-48 hrs)** | 6.5h | F.0 | ⬜ Not Started | | [Details](tasks/G.2-soak-testing.md) |

**Phase G Total**: 22 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (Foundation)
- [ ] A.0: Inventory Session & Interceptor Code
- [ ] A.1: Design Unified Session Architecture
- [ ] A.2: Design Interceptor Integration
- [ ] A.3: Create Migration Plan

### Week 2 (Server Refactoring)
- [ ] B.0: Fix Async Antipatterns
- [ ] B.1: Implement Hyper serve_connection
- [ ] B.2: Add SSE Support
- [ ] B.3: WebSocket Integration
- [ ] B.4: Graceful Shutdown

### Week 3 (Client & Session)
- [ ] C.0: Analyze Client Architecture
- [ ] C.1: Session Heartbeat Mechanism
- [ ] C.2: Reduce Client Spawns
- [ ] C.3: Client Connection Pool Integration
- [ ] C.4: Client SSE Handling

### Week 4 (Session & Error Handling)
- [ ] D.0: Port Session Manager
- [ ] D.1: Session Store Trait
- [ ] D.2: SQLite Implementation
- [ ] D.3: Interceptor Error Handling
- [ ] D.4: Redis Implementation
- [ ] D.5: SSE Session Tracking

### Week 5 (API & Observability)
- [ ] E.0: Builder Pattern API Design
- [ ] E.1: Port Interceptor Engine
- [ ] E.2: MCP Protocol Interceptor
- [ ] E.3: Observability & Metrics
- [ ] E.4: Rules Engine Integration
- [ ] E.5: HTTP Policy Interceptor
- [ ] E.6: Client Interceptor Chain
- [ ] E.7: Server Interceptor Chain

### Week 6 (Testing)
- [ ] F.0: Integration Test Suite
- [ ] F.1: Performance Benchmarks
- [ ] F.2: Load Testing
- [ ] F.3: Monitoring & Metrics
- [ ] F.4: Documentation

### Week 7 (Chaos & Security)
- [ ] G.0: Fault Injection & Chaos Testing
- [ ] G.1: Security Testing & Hardening
- [ ] G.2: Soak Testing (24-48 hrs)

### Completed Tasks
<!-- Tasks will be moved here as completed -->

## Success Criteria

### Functional Requirements
- ✅ Client and server using hyper patterns
- ✅ Session management with persistence
- ✅ Interceptor chain for message processing
- ✅ SSE/WebSocket for server notifications
- ✅ Graceful shutdown without data loss
- ✅ Connection pooling for efficiency

### Performance Requirements
- ✅ < 5% latency overhead (p95)
- ✅ < 100KB memory per session
- ✅ 1 spawn per connection (down from 5)
- ✅ Support 10,000+ concurrent sessions
- ✅ < 50ms startup time

### Quality Requirements
- ✅ 90% test coverage
- ✅ No clippy warnings
- ✅ Full API documentation
- ✅ Integration tests for all modes
- ✅ Performance benchmarks established

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Breaking API changes | HIGH | Maintain compatibility layer during migration | Planned |
| Session state loss | HIGH | Implement persistence before switching | Planned |
| Performance regression | MEDIUM | Benchmark at each phase | Active |
| Interceptor overhead | MEDIUM | Profile and optimize hot paths | Planned |
| Connection pool complexity | MEDIUM | Start with simple implementation | Planned |

## Session Planning Guidelines

### Next Session Prompt
Each phase should focus on 2-3 related tasks that can be completed in a single session. The `next-session-prompt.md` will be maintained with current focus.

### Optimal Session Structure
1. **Review** (10 min): Check tracker and analysis documents
2. **Implementation** (3-4 hours): Complete phase tasks
3. **Testing** (30 min): Run tests, fix issues
4. **Documentation** (20 min): Update tracker, commit
5. **Handoff** (10 min): Update next-session-prompt.md

### Context Window Management
- Focus on one phase at a time
- Reference analysis documents only when needed
- Keep test runs selective (use `--lib` during development)
- Create checkpoint commits frequently

### Task Completion Criteria
- [ ] All deliverables implemented
- [ ] Tests passing
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Tracker status updated

## Critical Implementation Guidelines

### Spawn Reduction Pattern
**ALWAYS follow the hyper pattern:**
```rust
// ❌ OLD: Multiple spawns
tokio::spawn(handler);
tokio::spawn(connection_driver);
tokio::spawn(sse_processor);

// ✅ NEW: Single spawn with serve_connection
tokio::spawn(async move {
    http1::Builder::new()
        .serve_connection(io, service_fn(handler))
        .await
});
```

### Lock Hygiene
**NEVER hold locks across await:**
```rust
// ❌ BAD
let guard = lock.write().await;
something.await; // Lock still held!

// ✅ GOOD
let data = {
    let guard = lock.write().await;
    guard.clone()
}; // Lock released
data.process().await;
```

### Session Consistency
**ALWAYS maintain session state in both modes:**
- Forward proxy: Client session tracking
- Reverse proxy: Dual session (client + upstream)
- Both modes: Interceptor context preservation

## Related Documents

### Primary References
- [Server Refactoring Analysis](analysis/)
- [Session Management Design](analysis/session-architecture.md)
- [Interceptor Architecture](analysis/interceptor-design.md)

### Implementation Guides
- [Spawn Audit Report](analysis/spawn-audit.md)
- [Server Architecture](analysis/server-architecture.md)
- [SSE Implementation](analysis/sse-implementation.md)

### Specifications
- [MCP Protocol v2025-11-05](https://spec.modelcontextprotocol.io)
- [Hyper v1 Documentation](https://docs.rs/hyper/latest)

## Next Actions

1. **Start Phase A analysis tasks**
2. **Review existing session/interceptor code**
3. **Create detailed migration plan**

## Notes

- Session management must support both SQLite and Redis backends
- Interceptors must be configurable and chainable
- All changes must maintain backward compatibility
- Focus on incremental improvements that can be tested

---

**Document Version**: 1.0  
**Created**: 2025-08-26  
**Last Modified**: 2025-08-26  
**Author**: Claude + Human

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-26 | 1.0 | Initial tracker creation | Claude |
| 2025-08-26 | 1.1 | Added Gemini feedback tasks (C.1, D.3, E.0, E.3, G.0-G.2) | Claude |