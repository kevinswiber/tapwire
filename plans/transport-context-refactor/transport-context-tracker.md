# Transport Context Refactor Tracker

## Overview

This tracker coordinates the refactoring of Shadowcat's transport layer to properly separate protocol concerns (JSON-RPC messages) from transport-specific metadata (HTTP headers, SSE events, stdio). This is a prerequisite for SSE proxy integration and must be completed before continuing with the proxy-sse-message-tracker.md work.

**Last Updated**: 2025-08-08  
**Total Estimated Duration**: 30-40 hours  
**Status**: Planning

## Problem Statement

The current `TransportMessage` enum conflates two concerns:
1. **Protocol Layer**: JSON-RPC message structure (Request/Response/Notification)
2. **Transport Layer**: How messages are delivered (HTTP, SSE, stdio)

With SSE integration, we need to track transport-specific metadata like:
- SSE event IDs, event types, retry hints
- HTTP headers, status codes, content types
- Stream state and correlation IDs
- Session continuity across transports

The `TransportMessage` type is used in 90 files with 658 occurrences, making this a significant architectural change.

## Goals

1. **Separate Concerns** - Decouple protocol messages from transport metadata
2. **Maintain Compatibility** - Enable incremental migration without breaking existing code
3. **Enable SSE Integration** - Provide proper abstractions for SSE-specific requirements
4. **Improve Type Safety** - Make transport-specific handling explicit and type-safe
5. **Support Future Transports** - Create extensible architecture for WebSocket, gRPC, etc.

## Architecture Vision

```
Current Architecture:
┌─────────────────────────────────────┐
│        TransportMessage             │
│  (Protocol + Some Transport Mixed)  │
└─────────────────────────────────────┘
           ↓ Used by 90 files

Target Architecture:
┌─────────────────────────────────────┐
│      Protocol Layer                 │
│   TransportMessage (unchanged)      │
│  Request/Response/Notification      │
└─────────────────────────────────────┘
           ↓ Wrapped by
┌─────────────────────────────────────┐
│      Transport Layer                │
│       MessageEnvelope               │
│  ┌────────────┬─────────────┐      │
│  │  Message   │   Context    │      │
│  │            │              │      │
│  │ Transport  │  Transport   │      │
│  │  Message   │  Metadata    │      │
│  └────────────┴─────────────┘      │
└─────────────────────────────────────┘
```

## Work Phases

### Phase 0: Analysis and Design (Week 1, Day 1)
Analyze current usage and design migration strategy.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.1 | **Analyze TransportMessage Usage** | 3h | None | ⬜ Not Started | | Map all 90 files using TransportMessage |
| A.2 | **Design MessageEnvelope Structure** | 2h | A.1 | ⬜ Not Started | | Define new types and traits |
| A.3 | **Create Migration Strategy** | 2h | A.2 | ⬜ Not Started | | Plan incremental migration path |
| A.4 | **Document Breaking Changes** | 1h | A.3 | ⬜ Not Started | | Identify unavoidable breaks |

**Phase 0 Total**: 8 hours

### Phase 1: Core Infrastructure (Week 1, Day 2-3)
Build the new transport context system alongside existing code.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | **Create MessageEnvelope Types** | 3h | A.2 | ⬜ Not Started | | `src/transport/envelope.rs` |
| C.2 | **Implement Transport Metadata** | 2h | C.1 | ⬜ Not Started | | HTTP, SSE, stdio variants |
| C.3 | **Add Context Extraction** | 2h | C.2 | ⬜ Not Started | | Extract metadata from transports |
| C.4 | **Create Compatibility Layer** | 3h | C.3 | ⬜ Not Started | | Bridge old and new APIs |

**Phase 1 Total**: 10 hours

### Phase 2: Transport Migration (Week 1, Day 4-5)
Migrate transport implementations to use new context system.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| T.1 | **Migrate StdioTransport** | 2h | C.4 | ⬜ Not Started | | Simplest transport to start |
| T.2 | **Migrate HttpTransport** | 3h | C.4 | ⬜ Not Started | | Add header context |
| T.3 | **Migrate HttpMcpTransport** | 2h | T.2 | ⬜ Not Started | | MCP-specific HTTP |
| T.4 | **Update Transport Trait** | 2h | T.1-T.3 | ⬜ Not Started | | Add context-aware methods |

**Phase 2 Total**: 9 hours

### Phase 3: Proxy Layer Migration (Week 2, Day 1-2)
Update proxy implementations to handle transport context.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| P.1 | **Update Forward Proxy** | 3h | T.4 | ⬜ Not Started | | Handle context in forwarding |
| P.2 | **Update Reverse Proxy** | 3h | T.4 | ⬜ Not Started | | Extract/inject HTTP context |
| P.3 | **Session Context Integration** | 2h | P.1-P.2 | ⬜ Not Started | | Link context to sessions |

**Phase 3 Total**: 8 hours

### Phase 4: Testing and Documentation (Week 2, Day 3)
Ensure everything works and is documented.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.1 | **Unit Tests for Envelope** | 2h | C.1-C.4 | ⬜ Not Started | | Test new types |
| D.2 | **Integration Tests** | 2h | P.3 | ⬜ Not Started | | End-to-end with context |
| D.3 | **Migration Guide** | 1h | All | ⬜ Not Started | | Document for other components |

**Phase 4 Total**: 5 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Implementation Details

### New Type Definitions

```rust
// src/transport/envelope.rs

/// Wraps a TransportMessage with transport-specific context
#[derive(Debug, Clone)]
pub struct MessageEnvelope {
    /// The protocol-level message (unchanged)
    pub message: TransportMessage,
    /// Transport-specific metadata
    pub context: TransportContext,
}

/// Transport-specific context and metadata
#[derive(Debug, Clone)]
pub struct TransportContext {
    /// Which transport this came from/going to
    pub transport_type: TransportType,
    /// Session identifier
    pub session_id: SessionId,
    /// Optional correlation ID for request/response matching
    pub correlation_id: Option<String>,
    /// Transport-specific metadata
    pub metadata: TransportMetadata,
    /// Timestamp when received/sent
    pub timestamp: std::time::Instant,
}

/// Transport-specific metadata variants
#[derive(Debug, Clone)]
pub enum TransportMetadata {
    /// Standard I/O transport (no additional metadata)
    Stdio,
    
    /// HTTP transport metadata
    Http {
        headers: HeaderMap,
        status_code: Option<u16>,
        method: Option<http::Method>,
        uri: Option<http::Uri>,
    },
    
    /// Server-Sent Events metadata
    Sse {
        event_id: Option<String>,
        event_type: Option<String>,
        retry_after: Option<u64>,
        last_event_id: Option<String>,
    },
    
    /// Future: WebSocket metadata
    WebSocket {
        frame_type: ws::FrameType,
        is_final: bool,
    },
}

/// Extended Transport trait with context support
#[async_trait]
pub trait TransportWithContext: Transport {
    /// Receive message with full context
    async fn receive_with_context(&mut self) -> TransportResult<MessageEnvelope>;
    
    /// Send message with specific context
    async fn send_with_context(&mut self, envelope: MessageEnvelope) -> TransportResult<()>;
}
```

### Migration Strategy

#### Step 1: Parallel Implementation (Non-breaking)
- Add new types alongside existing ones
- Implement `TransportWithContext` trait
- Provide default implementations that create minimal context

#### Step 2: Gradual Adoption
- Update transports one by one to support context
- Add context extraction in proxy layers
- Interceptors can start using context when available

#### Step 3: Full Migration
- Once all critical paths support context, deprecate old methods
- Update remaining components
- Remove compatibility shims

### Backward Compatibility

```rust
/// Compatibility extension for existing Transport trait
impl<T: Transport> TransportWithContext for T {
    default async fn receive_with_context(&mut self) -> TransportResult<MessageEnvelope> {
        let message = self.receive().await?;
        Ok(MessageEnvelope {
            message,
            context: TransportContext::default_for(self.transport_type()),
        })
    }
    
    default async fn send_with_context(&mut self, envelope: MessageEnvelope) -> TransportResult<()> {
        self.send(envelope.message).await
    }
}
```

## Success Criteria

### Functional Requirements
- ✅ Transport metadata properly separated from protocol messages
- ✅ SSE-specific metadata can be tracked through the system
- ✅ HTTP headers preserved through proxy layers
- ✅ Session correlation maintained across transports
- ✅ Backward compatibility maintained during migration

### Performance Requirements
- ✅ < 1% additional latency from context handling
- ✅ < 5MB additional memory for context storage
- ✅ No performance regression in existing code paths

### Quality Requirements
- ✅ 95% test coverage for new code
- ✅ No clippy warnings
- ✅ Complete documentation for new types
- ✅ Migration guide for dependent code

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Breaking changes in 90 files | HIGH | Incremental migration with compatibility layer | Planned |
| Performance regression | MEDIUM | Benchmark before/after each phase | Planned |
| SSE integration delays | HIGH | This refactor is now a prerequisite | Active |
| Incomplete migration | MEDIUM | Feature flags to toggle old/new paths | Planned |
| Context memory overhead | LOW | Use Cow<> and Arc<> for shared data | Planned |

## Impact on SSE Integration

This refactor directly enables the following SSE integration tasks:
- **S.2**: SSE Transport Wrapper can properly handle event metadata
- **S.4**: Parser hooks can access transport context
- **R.2**: SSE Response Handler can set proper event IDs
- **R.4**: Early correlation can use transport context
- **I.4**: Stream interception can modify SSE metadata
- **I.5**: Reverse proxy can preserve context

## Session Planning Guidelines

### Optimal Session Structure
1. **Review** (10 min): Review this tracker and current TransportMessage usage
2. **Implementation** (2-3 hours): Focus on one phase at a time
3. **Testing** (30 min): Test both old and new code paths
4. **Documentation** (15 min): Update migration guide
5. **Handoff** (10 min): Document any compatibility issues found

### Using the rust-code-reviewer
For this refactor, the rust-code-reviewer should focus on:
- Ensuring zero-cost abstractions where possible
- Validating lifetime management for context data
- Checking for unnecessary clones of large structures
- Reviewing async trait implementations
- Ensuring backward compatibility is maintained

### Context Window Management
- Focus on one transport at a time to minimize context
- Keep the MessageEnvelope definition readily available
- Reference existing TransportMessage usage patterns

### Task Completion Criteria
- [ ] New types compile without warnings
- [ ] Existing tests still pass
- [ ] New tests for context handling pass
- [ ] No performance regression
- [ ] Migration guide updated

## Critical Implementation Guidelines

### Proxy Mode Parity
**ALWAYS implement context handling in BOTH proxy modes:**
- **Forward Proxy**: Must preserve context when forwarding
- **Reverse Proxy**: Must extract context from HTTP and inject into responses

### Common Pitfalls to Avoid
- Don't clone the entire context unnecessarily
- Don't lose context during error handling
- Don't assume all transports have all metadata types
- Don't break existing Transport trait users

## Communication Protocol

### Status Updates
After completing each task:
1. Update task status in this tracker
2. Run benchmarks to ensure no regression
3. Document any unexpected TransportMessage usage found
4. Update count of migrated files

### Handoff Notes
Track migration progress:
- Files migrated: X/90
- Tests passing: Y/Z
- Performance impact: +X% latency, +Y MB memory

## Related Documents

### Primary References
- [SSE Proxy Integration Tracker](../sse-proxy-integration/sse-proxy-integration-tracker.md)
- [Proxy-SSE-Message Tracker](../proxy-sse-message-tracker.md) - **BLOCKED ON THIS REFACTOR**
- [Transport Module](../../shadowcat/src/transport/mod.rs)

### Design Documents
- [Architecture Plan](../002-shadowcat-architecture-plan.md)
- [Developer Guide](../003-shadowcat-developer-guide.md)

### Impacted Components
- All Transport implementations
- Forward and Reverse Proxy
- Session Manager
- Interceptor Engine
- Recorder/Replay systems

## Next Actions

1. **Immediate**: Analyze all 90 files using TransportMessage to understand usage patterns
2. **Day 1**: Complete Phase 0 analysis and design
3. **Day 2-3**: Implement core infrastructure (Phase 1)
4. **Day 4-5**: Migrate transports (Phase 2)
5. **Week 2**: Complete proxy migration and testing

## Notes

- This refactor is a **prerequisite** for SSE proxy integration
- The compatibility layer is critical for incremental migration
- Performance benchmarks must be run after each phase
- Consider using feature flags to allow rollback if issues arise
- The 658 occurrences of TransportMessage suggest many are imports/uses that won't need changes

---

**Document Version**: 1.0  
**Created**: 2025-08-08  
**Last Modified**: 2025-08-08  
**Author**: Development Team

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-08 | 1.0 | Initial tracker creation based on SSE integration requirements | Dev Team |