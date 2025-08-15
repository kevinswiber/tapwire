# Reverse Proxy Session Mapping - Implementation Tracker

## Project Status
**Status**: 🔵 Planning  
**Started**: 2025-01-15  
**Last Updated**: 2025-01-15

## Context
The reverse proxy needs to maintain its own session IDs separate from upstream servers to properly handle SSE reconnection, connection pooling, and failover scenarios. This is required by the MCP specification and necessary for robust proxy operation.

## Phases

### Phase A: Research & Analysis (2-3 hours)
Understand current session management implementation and identify all touchpoints.

| Task | Status | Effort | Notes |
|------|--------|--------|-------|
| A.0 Analyze current session management | ⬜ Not Started | 1h | Review Session struct and SessionManager |
| A.1 Map session ID usage across codebase | ⬜ Not Started | 1h | Find all places using session IDs |
| A.2 Design session mapping architecture | ⬜ Not Started | 1h | Create detailed technical design |
| A.3 Analyze forward vs reverse proxy differences | ✅ Completed | 0.5h | Determined reverse-only scope |

### Phase B: Core Implementation (4-6 hours)
Implement dual session ID tracking and mapping infrastructure.

| Task | Status | Effort | Notes |
|------|--------|--------|-------|
| B.0 Extend Session struct for dual IDs | ⬜ Not Started | 1h | Add upstream_session_id field |
| B.1 Implement session mapping table | ⬜ Not Started | 2h | Bidirectional mapping with proper locking |
| B.2 Update request handlers | ⬜ Not Started | 2h | Use correct session IDs for client/upstream |
| B.3 Add SSE event buffering | ⬜ Not Started | 1h | Buffer for Last-Event-Id replay |

### Phase C: SSE & Reconnection (2-3 hours)
Implement SSE-specific features for proper reconnection handling.

| Task | Status | Effort | Notes |
|------|--------|--------|-------|
| C.0 Implement Last-Event-Id tracking | ⬜ Not Started | 1h | Track per proxy session |
| C.1 Add event replay on reconnection | ⬜ Not Started | 1h | Replay from buffer |
| C.2 Handle upstream SSE reconnection | ⬜ Not Started | 1h | Maintain client connection during upstream reconnect |

### Phase D: Testing & Documentation (2-3 hours)
Ensure robustness and document the new architecture.

| Task | Status | Effort | Notes |
|------|--------|--------|-------|
| D.0 Unit tests for session mapping | ⬜ Not Started | 1h | Test mapping operations |
| D.1 Integration tests for SSE reconnection | ⬜ Not Started | 1h | Test full reconnection flow |
| D.2 Update documentation | ⬜ Not Started | 1h | Document session mapping architecture |

## Key Decisions
- **Decision**: Use proxy-generated UUIDs for client-facing session IDs
- **Decision**: Store upstream session IDs as optional strings (server-assigned format varies)
- **Decision**: Buffer last N events (configurable, default 100) for SSE replay
- **Decision**: Maintain backward compatibility with existing session management
- **Decision**: Keep transport layer unchanged - do mapping at reverse proxy layer only
- **Decision**: Forward proxy remains single-session (no changes needed)
- **Decision**: Session mapping only needed in reverse proxy (many-to-many model)

## Technical Notes

### Session ID Flow
```
1. Client → Proxy (no session) → Proxy generates UUID
2. Proxy → Upstream (initialize) → No upstream session yet
3. Upstream → Proxy (response) → Upstream assigns session ID
4. Proxy stores mapping: proxy_id ↔ upstream_id
5. Future requests use mapped IDs appropriately
```

### Data Structures
```rust
pub struct Session {
    pub id: SessionId,                        // Proxy-owned
    pub upstream_session_id: Option<String>,  // Server-assigned
    pub last_event_id: Option<String>,        // SSE tracking
    pub event_buffer: VecDeque<SseEvent>,     // Replay buffer
    // ... existing fields
}

pub struct SessionMapping {
    proxy_to_upstream: HashMap<SessionId, String>,
    upstream_to_proxy: HashMap<String, SessionId>,
}
```

## Blockers & Issues
- None currently identified

## Next Steps
1. Begin with Phase A.0 - analyze current session management
2. Create detailed technical design document
3. Implement core mapping infrastructure

## References
- MCP Specification: `/specs/mcp/docs/specification/2025-03-26/basic/transports.mdx`
- Current Session Implementation: `/src/session/`
- SSE Implementation: `/src/transport/sse/`