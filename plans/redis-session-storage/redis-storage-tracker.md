# Redis Session Storage Tracker

## Overview

This tracker coordinates the implementation of Redis as an additional session storage backend for Shadowcat's SessionManager. Currently, sessions are only stored in-memory, which limits scalability and prevents session sharing across multiple proxy instances. Adding Redis support will enable distributed session management, persistence, and horizontal scaling.

**Last Updated**: 2025-08-12  
**Total Estimated Duration**: 16-24 hours  
**Status**: Planning

## Goals

1. **Add Redis Backend** - Implement Redis as an alternative to in-memory storage for session data
2. **Maintain Compatibility** - Ensure existing in-memory storage continues to work unchanged
3. **Enable Distribution** - Support session sharing across multiple Shadowcat instances
4. **Preserve Performance** - Keep latency overhead minimal (< 5ms p95 for Redis operations)
5. **Support Failover** - Gracefully handle Redis unavailability with fallback options

## Architecture Vision

```
                SessionManager
                      |
              SessionStore (trait)
                   /     \
                  /       \
    InMemoryStore         RedisStore
         |                    |
    HashMap<SessionId>    Redis Cluster
                              |
                        - Sessions (hash)
                        - Frames (list)
                        - Expiry (sorted set)
                        - Metrics (counters)
```

## Work Phases

### Phase 1: Design & Abstraction (Week 1)
Extract storage interface and prepare for multiple backends

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| 1.1 | **Extract SessionStore Trait** | 3h | None | ⬜ Not Started | | [Details](tasks/1.1-extract-store-trait.md) |
| 1.2 | **Design Redis Data Model** | 2h | 1.1 | ⬜ Not Started | | [Details](tasks/1.2-redis-data-model.md) |
| 1.3 | **Add Storage Configuration** | 2h | 1.1 | ⬜ Not Started | | [Details](tasks/1.3-storage-config.md) |

**Phase 1 Total**: 7 hours

### Phase 2: Redis Implementation (Week 1-2)
Implement the Redis storage backend

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| 2.1 | **Implement RedisStore** | 4h | 1.1, 1.2 | ⬜ Not Started | | [Details](tasks/2.1-implement-redis-store.md) |
| 2.2 | **Add Connection Pooling** | 2h | 2.1 | ⬜ Not Started | | [Details](tasks/2.2-connection-pooling.md) |
| 2.3 | **Implement Atomic Operations** | 3h | 2.1 | ⬜ Not Started | | [Details](tasks/2.3-atomic-operations.md) |
| 2.4 | **Add TTL & Expiry** | 2h | 2.1 | ⬜ Not Started | | [Details](tasks/2.4-ttl-expiry.md) |

**Phase 2 Total**: 11 hours

### Phase 3: Integration & Testing (Week 2)
Integrate Redis backend and ensure reliability

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| 3.1 | **CLI Integration** | 2h | 2.1, 1.3 | ⬜ Not Started | | [Details](tasks/3.1-cli-integration.md) |
| 3.2 | **Failover & Fallback** | 3h | 2.1 | ⬜ Not Started | | [Details](tasks/3.2-failover-fallback.md) |
| 3.3 | **Performance Testing** | 2h | 2.1-2.4 | ⬜ Not Started | | [Details](tasks/3.3-performance-testing.md) |
| 3.4 | **Integration Tests** | 3h | All Phase 2 | ⬜ Not Started | | [Details](tasks/3.4-integration-tests.md) |

**Phase 3 Total**: 10 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (TBD)
- [ ] 1.1: Extract SessionStore Trait
- [ ] 1.2: Design Redis Data Model
- [ ] 1.3: Add Storage Configuration
- [ ] 2.1: Implement RedisStore (start)

### Week 2 (TBD)
- [ ] 2.1: Implement RedisStore (complete)
- [ ] 2.2: Add Connection Pooling
- [ ] 2.3: Implement Atomic Operations
- [ ] 2.4: Add TTL & Expiry
- [ ] 3.1: CLI Integration
- [ ] 3.2: Failover & Fallback
- [ ] 3.3: Performance Testing
- [ ] 3.4: Integration Tests

## Success Criteria

### Functional Requirements
- ✅ SessionStore trait abstracts storage operations
- ✅ Redis backend implements all SessionStore methods
- ✅ CLI supports selecting storage backend (--storage redis|memory)
- ✅ Sessions persist across proxy restarts when using Redis
- ✅ Multiple proxy instances can share sessions via Redis
- ✅ Graceful fallback when Redis is unavailable

### Performance Requirements
- ✅ < 5ms p95 latency for Redis operations
- ✅ < 10% memory overhead for Redis client
- ✅ Support 10,000+ concurrent sessions in Redis
- ✅ Connection pooling prevents connection exhaustion

### Quality Requirements
- ✅ 90% test coverage for new code
- ✅ No clippy warnings
- ✅ Full documentation with examples
- ✅ Integration tests for both storage backends
- ✅ Load tests demonstrating scalability

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Redis unavailability causes proxy failure | HIGH | Implement fallback to in-memory storage with warnings | Planned |
| Serialization overhead impacts performance | MEDIUM | Use efficient binary formats (bincode/msgpack) | Planned |
| Connection pool exhaustion under load | MEDIUM | Implement adaptive pool sizing and circuit breaker | Planned |
| Data consistency during failover | MEDIUM | Use versioning and conflict resolution | Planned |
| Memory leak in long-running sessions | LOW | Implement aggressive TTL and cleanup | Planned |

## Technical Design Decisions

### Storage Trait Design

```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create_session(&self, session: Session) -> SessionResult<()>;
    async fn get_session(&self, id: &SessionId) -> SessionResult<Session>;
    async fn update_session(&self, session: Session) -> SessionResult<()>;
    async fn delete_session(&self, id: &SessionId) -> SessionResult<()>;
    async fn list_sessions(&self) -> SessionResult<Vec<Session>>;
    async fn count_sessions(&self) -> SessionResult<usize>;
    
    // Frame operations
    async fn add_frame(&self, frame: MessageEnvelope) -> SessionResult<()>;
    async fn get_frames(&self, session_id: &SessionId) -> SessionResult<Vec<MessageEnvelope>>;
    async fn delete_frames(&self, session_id: &SessionId) -> SessionResult<()>;
}
```

### Redis Data Structure

```
Key Structure:
- shadowcat:sessions:{session_id}        -> Hash (session metadata)
- shadowcat:frames:{session_id}          -> List (message frames)
- shadowcat:sessions:active              -> Set (active session IDs)
- shadowcat:sessions:expiry              -> Sorted Set (TTL tracking)
- shadowcat:metrics:sessions:created     -> Counter
- shadowcat:metrics:sessions:completed   -> Counter

Session Hash Fields:
- id, transport_type, status, state
- created_at, last_activity, frame_count
- client_info, server_info, version_state
- tags (JSON array)

Frame List Entry:
- Serialized MessageEnvelope (bincode/msgpack)
```

### Configuration Schema

```toml
[storage]
backend = "redis"  # or "memory"

[storage.redis]
url = "redis://localhost:6379"
pool_size = 10
connection_timeout = "5s"
operation_timeout = "1s"
key_prefix = "shadowcat"
ttl = "24h"
fallback_to_memory = true

[storage.redis.cluster]
enabled = false
nodes = ["redis://node1:6379", "redis://node2:6379"]
```

## Implementation Guidelines

### Error Handling
- All Redis operations must handle connection failures gracefully
- Implement exponential backoff for retries
- Log warnings when falling back to memory storage
- Never panic on storage errors - return SessionError

### Serialization
- Use `serde` with `bincode` for efficient binary serialization
- Support migration between serialization formats
- Version the serialized data for backward compatibility

### Connection Management
- Use `bb8` or `deadpool` for connection pooling
- Implement health checks for Redis connections
- Monitor pool metrics (active, idle, wait time)
- Circuit breaker pattern for failing Redis instances

### Testing Strategy
- Unit tests with Redis mocks (`redis-test` or custom mocks)
- Integration tests with real Redis (`testcontainers`)
- Stress tests to validate connection pooling
- Failover tests simulating Redis outages
- Multi-instance tests for session sharing

## Session Planning Guidelines

### Optimal Session Structure
1. **Review** (10 min): Check this tracker and relevant task files
2. **Implementation** (2-3 hours): Complete the task deliverables
3. **Testing** (30 min): Run tests, fix issues
4. **Documentation** (15 min): Update tracker, create PR if needed
5. **Handoff** (10 min): Update NEXT_SESSION_PROMPT.md if needed

### Context Window Management
- Each task is designed to require < 50% context window
- Focus on single storage backend at a time
- Reference existing InMemoryStore implementation as needed

## Critical Implementation Guidelines

### Backward Compatibility
**MUST maintain full compatibility with existing code:**
- InMemoryStore remains the default
- No breaking changes to SessionManager API
- Configuration is optional (defaults work)
- Existing tests must continue passing

### Performance Considerations
- Use pipelining for batch operations
- Implement lazy loading for frames (pagination)
- Cache frequently accessed sessions locally
- Use Redis Lua scripts for atomic operations

## Related Documents

### Primary References
- [SessionManager Implementation](../../shadowcat/src/session/manager.rs)
- [InMemoryStore Implementation](../../shadowcat/src/session/store.rs)
- [Architecture Design](analysis/redis-architecture.md)

### Task Files
- [Phase 1 Tasks](tasks/)
- [Phase 2 Tasks](tasks/)
- [Phase 3 Tasks](tasks/)

### External Documentation
- [Redis Best Practices](https://redis.io/docs/manual/patterns/)
- [bb8 Connection Pool](https://docs.rs/bb8/latest/bb8/)
- [redis-rs Client](https://docs.rs/redis/latest/redis/)

## Next Actions

1. **Review current SessionManager implementation** to understand all storage touchpoints
2. **Extract SessionStore trait** from existing code
3. **Design Redis schema** optimized for MCP session patterns
4. **Select Redis client library** (redis-rs vs fred vs others)

## Notes

- Consider using Redis Streams for frame storage (append-only log)
- Evaluate Redis modules (RedisJSON, RedisTimeSeries) for enhanced functionality
- Plan for Redis Sentinel/Cluster support in future phases
- Consider implementing session migration tools for moving between storage backends
- May need to implement session compaction for long-running sessions

---

**Document Version**: 1.0  
**Created**: 2025-08-12  
**Last Modified**: 2025-08-12  
**Author**: Claude/Kevin

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-12 | 1.0 | Initial plan creation | Claude |