# MCP Unified Architecture Plan

## Overview

This plan coordinates the integration of three major architectural improvements into the MCP crate:

1. **Hyper Pattern Adoption** - 80% reduction in task spawns
2. **Session Management** - Comprehensive session tracking with persistence  
3. **Interceptor Chain** - Message processing pipeline for both client and server

## Current Status

**Phase**: A - Foundation Analysis  
**Status**: Planning Complete, Ready for Implementation  
**Next Step**: Execute Phase A tasks (A.0, A.1, A.2)

## Quick Start

1. Review the tracker: [mcp-unified-architecture-tracker.md](mcp-unified-architecture-tracker.md)
2. Check current phase: [next-session-prompt.md](next-session-prompt.md)
3. Start with task A.0: [tasks/A.0-inventory-existing-code.md](tasks/A.0-inventory-existing-code.md)

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

## Implementation Phases

| Phase | Focus | Duration | Status |
|-------|-------|----------|--------|
| A | Foundation Analysis | 18h | ⬜ Ready |
| B | Server Refactoring | 44h | ⬜ Planned |
| C | Client Optimization | 28h | ⬜ Planned |
| D | Session Integration | 38h | ⬜ Planned |
| E | Interceptor Chain | 40h | ⬜ Planned |
| F | Testing & Hardening | 42h | ⬜ Planned |

**Total**: 210 hours (6-8 weeks)

## Key Documents

### Analysis (Existing)
- [Spawn Audit](analysis/spawn-audit.md) - Current inefficiencies
- [Server Architecture](analysis/server-architecture.md) - Target design
- [SSE Implementation](analysis/sse-implementation.md) - Streaming guide

### Design (To Create)
- Session Architecture - How sessions integrate
- Interceptor Design - Message processing pipeline
- Migration Strategy - Incremental rollout plan

### Implementation
- [Task Files](tasks/) - Detailed work breakdown
- [Tracker](mcp-unified-architecture-tracker.md) - Progress tracking

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

1. **Start Session**: Read `next-session-prompt.md`
2. **Check Tracker**: Review phase status
3. **Execute Tasks**: Follow task files
4. **Test Changes**: Run targeted tests
5. **Update Progress**: Mark completed in tracker
6. **Prepare Handoff**: Update next-session-prompt

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

## Next Actions

1. Execute Phase A analysis tasks
2. Create integration design documents
3. Begin Phase B server refactoring

---

Ready to start? Begin with the [next session prompt](next-session-prompt.md).