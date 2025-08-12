# Redis Session Storage Analysis

This directory contains architectural analysis and design documents for implementing Redis as a session storage backend for Shadowcat.

## Documents

### [redis-architecture.md](redis-architecture.md)
Comprehensive architectural design covering:
- Current state analysis of in-memory storage
- Redis data model and schema design
- Connection architecture and pooling
- Performance optimization strategies
- Failover and reliability patterns
- Security considerations
- Monitoring and observability

## Key Design Decisions

### Why Redis?

1. **Distributed Sessions**: Enable session sharing across multiple Shadowcat instances
2. **Persistence**: Sessions survive proxy restarts
3. **Scalability**: Handle 10,000+ concurrent sessions
4. **Performance**: Sub-5ms latency for most operations
5. **Reliability**: Built-in replication and clustering

### Storage Model

We chose a hybrid approach:
- **Hash** for session metadata (efficient field updates)
- **List** for message frames (append-only, ordered)
- **Sets** for indices (fast membership checks)
- **Sorted Sets** for TTL management (efficient expiry)

### Serialization Format

After evaluation, we recommend **MessagePack**:
- Smaller than JSON (50-60% size reduction)
- Faster than JSON (2-3x encode/decode speed)
- Better schema evolution than Bincode
- Good library support in Rust

### Connection Strategy

Using **bb8** connection pool with **redis-rs**:
- Proven production stability
- Async/await support via Tokio
- Connection multiplexing
- Automatic reconnection
- Health monitoring

## Performance Targets

| Operation | Target Latency (p95) | Notes |
|-----------|---------------------|-------|
| get_session | < 2ms | Most frequent operation |
| update_activity | < 1ms | High frequency |
| add_frame | < 5ms | Includes serialization |
| create_session | < 10ms | Less frequent |
| list_sessions | < 50ms | Pagination recommended |

## Risk Analysis

### High Priority Risks

1. **Redis Unavailability**
   - Mitigation: Automatic fallback to in-memory storage
   - Implementation: Circuit breaker pattern

2. **Network Latency**
   - Mitigation: Local caching layer for hot sessions
   - Implementation: LRU cache with TTL

3. **Serialization Overhead**
   - Mitigation: Efficient binary format (MessagePack)
   - Implementation: Async serialization

### Medium Priority Risks

1. **Connection Pool Exhaustion**
   - Mitigation: Adaptive pool sizing
   - Implementation: Dynamic pool management

2. **Memory Growth**
   - Mitigation: Aggressive TTL, frame pagination
   - Implementation: Cleanup tasks, lazy loading

## Implementation Phases

### Phase 1: Foundation (Week 1)
- Extract SessionStore trait
- Design Redis schema
- Basic RedisStore implementation

### Phase 2: Reliability (Week 2)
- Connection pooling
- Circuit breaker
- Fallback mechanisms
- Health monitoring

### Phase 3: Optimization (Week 3)
- Caching layer
- Pipelining
- Batch operations
- Performance tuning

## Testing Requirements

### Functional Tests
- ✅ All SessionStore operations
- ✅ Concurrent access patterns
- ✅ Session lifecycle (create → update → delete)
- ✅ Frame storage and retrieval

### Reliability Tests
- ✅ Redis connection failures
- ✅ Network timeouts
- ✅ Failover to in-memory
- ✅ Recovery after outage

### Performance Tests
- ✅ Latency under load
- ✅ Throughput limits
- ✅ Memory usage
- ✅ Connection pool behavior

### Integration Tests
- ✅ Multiple Shadowcat instances
- ✅ Session sharing
- ✅ Distributed operations
- ✅ Redis cluster mode

## Operational Considerations

### Deployment

1. **Redis Configuration**
   ```
   maxmemory 2gb
   maxmemory-policy allkeys-lru
   save ""  # Disable persistence for cache-only
   ```

2. **Network Security**
   - Use Redis ACLs
   - TLS for connections
   - Private network only

3. **Monitoring**
   - Redis INFO statistics
   - Connection pool metrics
   - Circuit breaker state
   - Failover events

### Maintenance

1. **Session Cleanup**
   - Automatic TTL expiry
   - Periodic cleanup job
   - Manual cleanup tools

2. **Data Migration**
   - Export/import tools
   - Format version tracking
   - Backward compatibility

## Future Enhancements

### Short Term (3-6 months)
- Redis Cluster support
- Redis Streams for event sourcing
- Geo-distributed sessions

### Long Term (6-12 months)
- Redis modules integration
- Multi-region replication
- Session analytics
- Real-time monitoring dashboard

## References

- [Redis Best Practices](https://redis.io/docs/manual/patterns/)
- [bb8 Connection Pool](https://docs.rs/bb8/)
- [redis-rs Documentation](https://docs.rs/redis/)
- [MessagePack Specification](https://msgpack.org/)
- [Circuit Breaker Pattern](https://martinfowler.com/bliki/CircuitBreaker.html)