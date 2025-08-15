# Reverse Proxy Refactoring Plan

## Problem Statement
The Shadowcat reverse proxy has grown organically to 3400+ lines in a single file (`reverse.rs`), making it difficult to maintain and extend. Additionally, SSE (Server-Sent Events) streaming is not working correctly - the proxy attempts to buffer entire SSE streams, causing client timeouts. The architecture needs refactoring to properly handle both JSON and SSE responses, support interceptors, and implement session mapping.

## Goals
1. **Fix SSE Streaming**: Ensure SSE responses are properly proxied without buffering
2. **Modularize Code**: Break up the monolithic `reverse.rs` into manageable modules
3. **Reuse Infrastructure**: Leverage existing SSE transport code in `src/transport/sse/`
4. **Support Interceptors**: Enable message interception for both JSON and SSE responses
5. **Implement Session Mapping**: Support proxy-managed sessions separate from upstream sessions
6. **Maintain Performance**: Keep latency overhead < 5% p95 as per requirements

## Non-Goals
- Complete rewrite of the reverse proxy
- Changes to the forward proxy
- Changes to the transport layer interface
- Breaking changes to the CLI interface

## Key Questions
1. How should we handle the fundamental difference between JSON (request-response) and SSE (streaming)?
2. Can we reuse the existing `SseConnectionManager` and `SseStream` from the transport layer?
3. How do we process SSE events through interceptors without breaking the stream?
4. What's the best way to map between client and upstream session IDs in SSE events?
5. Should SSE and JSON requests follow completely separate code paths?

## Proposed Solution

### Architecture Overview
```
src/proxy/
├── mod.rs                 # Public interface
├── reverse/
│   ├── mod.rs            # Core reverse proxy logic
│   ├── config.rs         # Configuration structures
│   ├── handlers.rs       # HTTP request handlers
│   ├── json.rs           # JSON request/response handling
│   ├── sse.rs            # SSE streaming handling
│   ├── session.rs        # Session mapping logic
│   ├── upstream.rs       # Upstream connection management
│   ├── interceptor.rs   # Interceptor integration
│   └── metrics.rs        # Metrics and monitoring
```

### Key Design Decisions
1. **Separate Paths**: JSON and SSE requests will have separate handling paths after initial routing
2. **Streaming First**: SSE responses will be streamed directly, with events parsed for interceptors
3. **Session Mapping**: Implement a bidirectional mapping table for client<->upstream sessions
4. **Event Buffering**: Buffer last N events for SSE reconnection support
5. **Reuse SSE Infrastructure**: Integrate with existing `transport/sse/` modules

## Approach

### Phase 1: Analysis and Design
- Deep analysis of current code structure
- Document all dependencies and interactions
- Design module boundaries and interfaces
- Create detailed technical specifications

### Phase 2: Modularization
- Extract code into logical modules
- Maintain backward compatibility
- Add comprehensive tests for each module

### Phase 3: SSE Integration
- Integrate existing SSE transport infrastructure
- Implement proper SSE streaming without buffering
- Add event parsing for interceptors

### Phase 4: Session Mapping
- Implement session mapping architecture
- Support SSE reconnection with event replay
- Handle multiple clients per upstream

## Success Criteria
- [ ] SSE responses stream without client timeouts
- [ ] `reverse.rs` reduced to < 500 lines
- [ ] All existing tests pass
- [ ] New integration tests for SSE streaming
- [ ] Performance targets maintained (< 5% latency overhead)
- [ ] Session mapping works for both JSON and SSE

## Rollout Plan
1. Complete analysis and design documentation
2. Create new module structure without changing functionality
3. Implement SSE streaming fix in new architecture
4. Add session mapping capabilities
5. Deprecate old code paths
6. Remove legacy code

## References
- [SSE Status Document](../../shadowcat/SSE_STATUS.md)
- [SSE Refactoring Notes](../../shadowcat/REFACTOR_SSE.md)
- [Session Mapping Plan](../reverse-proxy-session-mapping/reverse-proxy-session-mapping-tracker.md)
- [MCP Specification](https://spec.modelcontextprotocol.io)

## MCP Reference Implementations
Available in `~/src/modelcontextprotocol/` for validation and testing:

### Core Resources
- **Specifications** (`modelcontextprotocol/specs/`): Official protocol versions and schemas
- **Inspector** (`inspector/`): MCP debugging tool with SSE support - excellent for testing our proxy
- **TypeScript SDK** (`typescript-sdk/`): Most up-to-date reference implementation
- **Rust SDK** (`rust-sdk/`): Official rmcp crate implementation for comparison
- **Example Servers** (`servers/`): Test targets including `everything` server with full MCP features

### How to Use for This Refactor
1. **Validate SSE Behavior**: Compare our SSE handling with Inspector's implementation
2. **Protocol Compliance**: Check TypeScript SDK for canonical message handling
3. **Testing Targets**: Use `servers/everything` for comprehensive integration tests
4. **Edge Cases**: Review how official SDKs handle error conditions and reconnection

## Open Questions
1. Should we support HTTP/2 and HTTP/3 for SSE?
2. How many events should we buffer for reconnection?
3. Should interceptors be able to modify SSE event IDs?
4. How do we handle upstream disconnections during SSE streaming?