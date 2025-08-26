# MCP Unified Architecture Plan

## 🎯 IMPORTANT: We Use Two Trackers!

We maintain **two trackers** for maximum flexibility (Option C from migration decision):

1. **[v1 Comprehensive Tracker](mcp-unified-architecture-tracker.md)** - Full 250-hour plan with all Gemini feedback
2. **[v2 Critical Path Tracker](mcp-tracker-v2-critical-path.md)** - Optimized 200-hour execution guide ⭐ **USE THIS**

**Strategy**: Execute from v2 (Sprint-based), reference v1 for detailed requirements.

## Overview

This plan coordinates the integration of three major architectural improvements into the MCP crate:

1. **Hyper Pattern Adoption** - 80% reduction in task spawns
2. **Session Management** - Comprehensive session tracking with persistence  
3. **Interceptor Chain** - Message processing pipeline for both client and server

## Current Status

**Sprint**: 1 - Core Foundation  
**Status**: Ready to Start  
**Next Task**: 1.0 - Fix Async Antipatterns (8h)  
**Session Guide**: [next-session-prompt.md](next-session-prompt.md) ⭐ **START HERE**

## Quick Start for New Session

```bash
# 1. Read the session prompt (THIS IS YOUR GUIDE)
cat plans/mcp-unified-architecture/next-session-prompt.md

# 2. Check which sprint/task we're on
grep "Sprint" plans/mcp-unified-architecture/mcp-tracker-v2-critical-path.md | head -5

# 3. Go to the MCP crate
cd crates/mcp

# 4. Start implementing!
```

## Architecture Goals

### Performance
- Reduce spawns from 5 to 1 per connection
- Memory savings of 8-32MB per 1000 connections
- Support 10,000+ concurrent sessions
- < 5% latency overhead

### Features
- Unified session management (SQLite/Redis)
- Configurable interceptor chains
- SSE/WebSocket for server notifications
- Graceful shutdown with data preservation

### Quality
- Production-ready error handling
- Comprehensive test coverage
- Performance benchmarks
- Full documentation

## Implementation Approach: Critical Path Sprints

We follow the **v2 Critical Path** approach for execution:

| Sprint | Focus | Duration | Delivers |
|--------|-------|----------|----------|
| 1 | Core Foundation | 38h | Working proxy with metrics |
| 2 | Persistence & SSE | 32h | Persistent sessions, SSE support |
| 3 | Production Essentials | 32h | Interceptors, error handling |
| 4 | Advanced Features | 38h | Pooling, Redis, WebSocket |
| 5 | Testing & Hardening | 42h | Battle-tested system |

**Total**: ~200 hours (vs 250 in comprehensive plan)

### Why Two Trackers?

- **v1**: Shows everything we might want (comprehensive)
- **v2**: Shows critical path to get there (execution)
- **Result**: No analysis paralysis, clear next steps

## Key Documents

### 📍 Navigation (Start Here)
- **[next-session-prompt.md](next-session-prompt.md)** ⭐ - What to do right now
- **[mcp-tracker-v2-critical-path.md](mcp-tracker-v2-critical-path.md)** - Execution tracker (use this)
- **[mcp-unified-architecture-tracker.md](mcp-unified-architecture-tracker.md)** - Reference tracker (consult for details)
- **[TRACKER-MIGRATION-DECISION.md](TRACKER-MIGRATION-DECISION.md)** - Why we have two trackers

### Analysis (Completed)
- [Spawn Audit](analysis/spawn-audit.md) - Current inefficiencies
- [Server Architecture](analysis/server-architecture.md) - Target design
- [Plan Optimization Review](analysis/plan-optimization-review.md) - How we optimized the plan
- [Gemini Feedback Incorporation](analysis/gemini-feedback-incorporation.md) - External review addressed

### Implementation
- [Task Files](tasks/) - Detailed work breakdown (referenced by both trackers)
- [Sprint Tasks](mcp-tracker-v2-critical-path.md#critical-path-sprints) - Organized by value delivery

## Critical Decisions Made

1. **Single Spawn Pattern**: Use hyper's `serve_connection` exclusively
2. **Lock Hygiene**: Never hold locks across await points
3. **Semaphore Limits**: Atomic connection limit enforcement
4. **SSE for Notifications**: Server-sent events for push messages
5. **Unified Session Store**: Trait-based storage abstraction

## Key Technical Components

### From Shadowcat (To Port)
```
src/session/
├── manager.rs         # Core session manager
├── store.rs          # Storage trait
├── memory.rs         # In-memory store
├── persistence_worker.rs # Background persistence
└── sse_integration.rs # SSE session tracking

src/interceptor/
├── engine.rs         # Interceptor chain engine
├── mcp_interceptor.rs # MCP protocol interceptor
├── rules_engine.rs   # Rules-based processing
└── http_policy.rs    # HTTP policy enforcement
```

### In MCP Crate (To Enhance)
```
crates/mcp/src/
├── server.rs         # Apply hyper patterns
├── client.rs         # Reduce spawns
├── interceptor.rs    # Expand functionality
└── pool/            # Already optimized ✓
```

## Success Metrics

### Performance
- [ ] Task spawns: 5 → 1 per connection
- [ ] Memory: < 100KB per session
- [ ] Latency: < 5% p95 overhead
- [ ] Throughput: > 10,000 req/sec

### Functionality
- [ ] Sessions persist across restarts
- [ ] Interceptors process all messages
- [ ] SSE delivers notifications reliably
- [ ] Graceful shutdown preserves state

### Quality
- [ ] 90% test coverage
- [ ] Zero clippy warnings
- [ ] Benchmarks established
- [ ] Documentation complete

## Development Workflow

### 🚀 Session Start Checklist
```bash
# 1. Where are we?
cat plans/mcp-unified-architecture/next-session-prompt.md

# 2. What's the current sprint?
grep -A 10 "Current Status" plans/mcp-unified-architecture/README.md

# 3. What task are we on?
grep "✅\|🔄" plans/mcp-unified-architecture/mcp-tracker-v2-critical-path.md || echo "Starting fresh!"
```

### During Development
1. **Follow v2 Sprint** from critical path tracker
2. **Reference v1 Task** for detailed requirements
3. **Test Incrementally** - don't wait until end
4. **Update Both Trackers** when completing tasks

### Session End Checklist
- [ ] Update task status in v2 tracker
- [ ] Mark corresponding items in v1 tracker
- [ ] Update Current Status in this README
- [ ] Update next-session-prompt.md with next steps
- [ ] Commit with clear message about what was done

## Testing Strategy

```bash
# During development (fast feedback)
cargo test --lib                    # Unit tests only
cargo test transport::              # Specific module

# Before commits (must pass)
cargo fmt                           # Format code
cargo clippy --all-targets -- -D warnings  # Lint
cargo test --lib --bins            # Core tests

# Before phase completion
cargo test                         # Full suite
cargo bench                        # Performance check
```

## Risk Management

| Risk | Mitigation |
|------|------------|
| API Breaking Changes | Compatibility layer during migration |
| Performance Regression | Benchmark at each phase |
| Session State Loss | Implement persistence first |
| Complexity Growth | Incremental, tested changes |

## Contact Points

- **Plan Location**: `~/src/tapwire/plans/mcp-unified-architecture/`
- **Implementation**: `~/src/tapwire/shadowcat-mcp-compliance/crates/mcp/`
- **Source Material**: `~/src/tapwire/shadowcat-mcp-compliance/src/`

## Sprint 1 Immediate Actions

### Task 1.0: Fix Async Antipatterns (8h) ⭐ CRITICAL
**What**: Remove block_on, fix locks across await, reduce spawns by 50%+  
**Where**: `/crates/mcp/src/server/` and `/crates/mcp/src/client/`  
**Reference**: v1 Task B.0 in `tasks/B.0-fix-async-antipatterns.md`  

### Task 1.1: Basic Observability (6h) ⭐ CRITICAL
**What**: OpenTelemetry + Prometheus, metrics endpoint  
**Reference**: v1 Task E.3 in `tasks/E.3-observability.md`  

### Task 1.2: Basic Hyper Server (6h) ⭐ CRITICAL
**What**: Single spawn per connection pattern  
**Reference**: v1 Task B.1 (partial)  

---

## ⚡ Quick Reference

**Lost?** → Read [next-session-prompt.md](next-session-prompt.md)  
**Confused about trackers?** → Read [TRACKER-MIGRATION-DECISION.md](TRACKER-MIGRATION-DECISION.md)  
**Need the plan?** → Use [v2 Critical Path](mcp-tracker-v2-critical-path.md)  
**Need details?** → Check [v1 Comprehensive](mcp-unified-architecture-tracker.md)  
**Ready to code?** → Start with Sprint 1, Task 1.0!

---

Ready to start? Begin with the [next session prompt](next-session-prompt.md).