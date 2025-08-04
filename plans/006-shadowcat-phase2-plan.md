# Shadowcat Phase 2 Implementation Plan

**Project:** Shadowcat Phase 2 - HTTP Support & Core Proxy  
**Timeline:** Weeks 2-4  
**Status:** Planning

---

## Overview

Phase 2 focuses on implementing the actual proxy functionality, HTTP transport support, and basic session management. This phase transforms the proof-of-concept into a working proxy that can handle real MCP traffic.

---

## High Priority Tasks

### 1. Implement Forward Proxy Logic
**File:** `src/proxy/forward.rs`  
**Details:** [tasks/001-forward-proxy.md](tasks/001-forward-proxy.md)
- [ ] Create ForwardProxy struct
- [ ] Implement bidirectional message routing
- [ ] Handle client/server communication loop
- [ ] Add connection pooling for transports
- [ ] Implement graceful shutdown

### 2. Session Management Foundation
**Files:** `src/session/manager.rs`, `src/session/store.rs`  
**Details:** [tasks/002-session-manager.md](tasks/002-session-manager.md)
- [ ] Implement SessionManager with in-memory storage
- [ ] Track session lifecycle (creation, frames, termination)
- [ ] Associate transport connections with sessions
- [ ] Add session timeout handling
- [ ] Create Frame type for message tracking

### 3. HTTP Transport Implementation
**File:** `src/transport/http.rs`  
**Details:** [tasks/003-http-transport.md](tasks/003-http-transport.md)
- [ ] Implement Transport trait for HTTP
- [ ] Support MCP-Session-Id header
- [ ] Handle streaming responses
- [ ] Add connection pooling with hyper
- [ ] Implement SSE fallback support

---

## Medium Priority Tasks

### 4. Basic Recording Engine
**File:** `src/recorder/tape.rs`  
**Details:** [tasks/004-tape-recorder.md](tasks/004-tape-recorder.md)
- [ ] Design tape format (JSON structure)
- [ ] Implement TapeRecorder
- [ ] Add frame capture with timing
- [ ] Create tape storage interface
- [ ] Add tape metadata

### 5. Integration with Proxy
- [ ] Wire SessionManager into ForwardProxy
- [ ] Add recording hooks to proxy
- [ ] Implement transport selection logic
- [ ] Add metrics collection points
- [ ] Create proxy configuration

### 6. CLI Enhancements
- [ ] Make forward stdio command actually proxy
- [ ] Implement forward http command
- [ ] Add session listing command
- [ ] Add configuration file support
- [ ] Improve error messages

---

## Low Priority Tasks

### 7. Basic Interceptor Framework
**File:** `src/interceptor/engine.rs`
- [ ] Create Interceptor trait
- [ ] Implement InterceptorChain
- [ ] Add logging interceptor
- [ ] Create pass-through interceptor
- [ ] Design InterceptContext

### 8. Integration Tests
**Directory:** `tests/integration/`
- [ ] End-to-end proxy tests
- [ ] Multi-transport tests
- [ ] Session management tests
- [ ] Recording/playback tests
- [ ] Error scenario tests

---

## Week-by-Week Breakdown

### Week 2: Core Proxy Implementation
1. Implement ForwardProxy with basic routing
2. Create working stdio-to-stdio proxy
3. Add session creation and tracking
4. Test with real MCP servers

### Week 3: HTTP Transport
1. Implement HTTP transport
2. Add header handling
3. Test HTTP-to-stdio proxy scenarios
4. Implement transport negotiation

### Week 4: Recording & Polish
1. Implement basic tape recording
2. Add recording to proxy flow
3. Create integration tests
4. Fix bugs and improve performance

---

## Success Criteria

### Functional Requirements
- [ ] Can proxy stdio MCP server with full bidirectional communication
- [ ] Can proxy HTTP MCP server with streaming support
- [ ] Sessions are tracked and can be listed
- [ ] Basic recording captures all traffic
- [ ] Clean shutdown without dropping messages

### Performance Requirements
- [ ] < 5% latency overhead for stdio proxy
- [ ] < 10ms overhead for HTTP proxy
- [ ] Handle 100 concurrent sessions
- [ ] No memory leaks over 1 hour run

### Quality Requirements
- [ ] All new code has unit tests
- [ ] Integration tests cover main flows
- [ ] No new compiler warnings
- [ ] Documentation for public APIs

---

## Technical Decisions

### Message Routing
- Use tokio::select! for bidirectional flow
- Separate tasks for each direction
- Channel-based communication between components

### Session Storage
- Start with in-memory HashMap
- Prepare interface for future SQLite backend
- Use Arc<RwLock<>> for concurrent access

### HTTP Implementation
- Use existing axum setup from dependencies
- Leverage tower middleware for common concerns
- Connection pooling via hyper's built-in support

---

## Risks and Mitigations

### Risk: Complexity of bidirectional proxy
**Mitigation:** Start simple, test thoroughly, add features incrementally

### Risk: HTTP streaming complexity
**Mitigation:** Implement basic POST/response first, add streaming later

### Risk: Session state management
**Mitigation:** Keep session state minimal, use immutable patterns where possible

---

## Dependencies on Other Work

- Requires Phase 1 completion ✅
- No external dependencies
- May need to revisit error types as we learn more

---

## Definition of Done

Phase 2 is complete when:
1. `cargo run -- forward stdio -- npx some-mcp-server` works for full session
2. `cargo run -- forward http --port 8080 --target http://localhost:3000` works
3. Sessions can be listed with a new CLI command
4. Basic recording produces valid tape files
5. All tests pass and code is documented