# Next Session: Multi-Session Forward Proxy - Phase B Implementation

## Quick Context
The forward proxy currently only handles **one client-server connection** at a time. We need to enable **multiple concurrent client connections** with independent upstream servers.

## Completed Work (Phase A - 2.5 hours)
✅ Analyzed current single-session architecture  
✅ Designed multi-session architecture with accept loop  
✅ Researched connection pooling (HTTP-only, defer to Phase 2)  
✅ Planned resource management and limits  

All analysis documents are in: `plans/multi-session-forward-proxy/analysis/`

## Your Mission: Phase B - Core Implementation (6-8 hours)

### Priority Tasks

#### B.0: Refactor ForwardProxy Structure (1.5h)
1. Read the current implementation: `shadowcat/src/proxy/forward.rs`
2. Review the proposed architecture: `plans/multi-session-forward-proxy/analysis/multi-session-architecture.md`
3. Create new `MultiSessionForwardProxy` struct with:
   - Session registry (`HashMap<SessionId, SessionHandle>`)
   - Max sessions configuration
   - Shared resources (interceptors, rate limiters, etc.)
4. Keep backward compatibility with single-session mode

#### B.1: Implement Connection Accept Loop (2h)
1. Add TCP listener for HTTP transport
2. Implement accept loop that:
   - Checks session limits before accepting
   - Creates new session for each connection
   - Spawns session handler tasks
3. Reference the design in `analysis/multi-session-architecture.md`

#### B.2: Add Session Registry (1.5h)
1. Implement `SessionHandle` to track:
   - Session ID and client address
   - Task handles for cleanup
   - Last activity time
2. Thread-safe session storage with `Arc<RwLock<HashMap>>`
3. Methods to add, remove, and query sessions

#### B.3: Per-Client Task Spawning (2h)
1. Extract forwarding logic into standalone functions
2. Spawn independent tasks for each client-server pair
3. Ensure proper isolation between sessions
4. Handle task cleanup on disconnection

#### B.4: Graceful Shutdown (1h)
1. Implement cleanup loop for expired sessions
2. Handle shutdown signal to close all sessions
3. Ensure no resource leaks on shutdown

### Key Files to Modify
- `shadowcat/src/proxy/forward.rs` - Main implementation
- `shadowcat/src/proxy/mod.rs` - Export new types
- `shadowcat/src/cli/forward.rs` - Add multi-session CLI flags
- `shadowcat/src/api.rs` - Update API methods

### Testing Approach
1. Start with unit tests for session registry
2. Test with 2-3 concurrent clients manually
3. Verify resource cleanup
4. Ensure single-session mode still works

### Success Criteria
- [ ] Accept multiple HTTP clients concurrently
- [ ] Each client gets independent upstream connection
- [ ] Sessions cleaned up properly
- [ ] Resource limits enforced
- [ ] Backward compatibility maintained

## Important Notes
- **Start simple**: Get basic multi-session working before optimizations
- **No pooling yet**: Leave connection pooling for Phase C
- **HTTP first**: Focus on HTTP transport, stdio stays single-session
- **Test frequently**: Multi-threading bugs are hard to debug

## Commands to Run
```bash
cd shadowcat

# Build and test
cargo build
cargo test proxy::forward

# Manual testing with multiple clients
# Terminal 1: Start proxy
RUST_LOG=debug cargo run -- forward http --port 8080 --multi-session --url http://upstream

# Terminal 2-4: Send requests
curl http://localhost:8080/...
```

## References
- Tracker: `plans/multi-session-forward-proxy/multi-session-forward-proxy-tracker.md`
- Current architecture analysis: `analysis/current-architecture.md`
- Proposed design: `analysis/multi-session-architecture.md`
- Resource management: `analysis/resource-management-plan.md`

## Next After Phase B
Phase C: Transport-specific optimizations and HTTP connection pooling