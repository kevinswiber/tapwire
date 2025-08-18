# Next Session: Multi-Session Forward Proxy or Alternative Priorities

## ✅ Completed in This Session (2025-08-18)

### SSE Resilience Integration - FULLY COMPLETE
- EventTracker already integrated in handle_sse() (line 1676-1689)
- Events recorded with deduplication (line 1966) 
- Upstream reconnection with Last-Event-Id (line 1877-1880)
- All functionality verified and working!

### Session Store Architecture - COMPLETE
- Lazy persistence initialization in SessionManager
- ReverseProxyServerBuilder for custom stores
- Library API exports for SessionStore interface
- Ready for Redis/SQLite backends

## Next Priority Options

### Option 1: Multi-Session Forward Proxy (Recommended)

The forward proxy currently only supports a single client. We need multiple concurrent clients.

**Implementation Plan (4 hours):**
1. Make ForwardProxy accept multiple connections
2. Session isolation per client
3. Connection pool sharing
4. Resource limits and cleanup

**Key Files:**
- `src/proxy/forward/mod.rs`
- `src/proxy/forward/stdio.rs`
- `src/proxy/pool.rs`

**Success Criteria:**
- Support 100+ concurrent sessions
- Clean resource management
- All tests passing

### Option 2: Dual Session ID Mapping

Track both client and server session IDs in reverse proxy for better routing.

**Implementation Plan (3 hours):**
1. Create session_mapping module
2. Store bidirectional mappings
3. Handle ID conflicts
4. Persistent storage

**Key Files:**
- `src/proxy/reverse/session_mapping.rs` (new)
- `src/proxy/reverse/legacy.rs`

### Option 3: Redis Session Store

Implement distributed session storage for production deployments.

**Implementation Plan (4 hours):**
1. Create RedisSessionStore implementing SessionStore trait
2. Handle connection pooling
3. Implement TTL and cleanup
4. Add to library exports

**Key Files:**
- `src/session/redis.rs` (new)
- `src/session/mod.rs`
- Integration with ReverseProxyServerBuilder

## Testing Any Option

```bash
cd shadowcat

# Run relevant tests
cargo test proxy::forward        # Option 1
cargo test proxy::reverse        # Option 2  
cargo test session::             # Option 3

# Build and manual test
cargo build --release
./target/release/shadowcat <relevant commands>
```

## Recommendation

Start with **Option 1: Multi-Session Forward Proxy** as it:
- Addresses a clear limitation (single client only)
- Builds on existing SessionManager work
- Enables important use cases (shared proxy)
- Has clear success metrics

The SessionManager and ConnectionPool are already thread-safe and ready for this enhancement.

## Repository Status

### shadowcat (submodule)
- Latest commit: feat: add ReverseProxyServerBuilder for custom session stores
- Branch: main
- All tests passing ✅

### tapwire (parent)
- Updated tracker and documentation
- Ready for next phase
- Clear roadmap defined

## Quick Start for Next Session

```bash
# Pull latest changes
cd ~/src/tapwire
git pull
git submodule update --init --recursive

# Check current status
cd shadowcat
cargo test --lib

# Review the specific plan
cat ../plans/multi-session-forward-proxy/README.md  # If it exists
# OR start fresh with the outline above
```

Pick Option 1 and implement multi-session support in the forward proxy!