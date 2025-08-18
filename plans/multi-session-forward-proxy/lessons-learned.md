# Multi-Session Forward Proxy - Lessons Learned

## Key Discovery: Fundamental Architecture Issue

During Phase B implementation, we discovered that the forward proxy has a fundamental architectural issue that prevents it from truly supporting multiple client connections.

## What We Built vs What's Actually Needed

### What We Built (Phase B)
- ✅ Session registry infrastructure (HashMap tracking sessions)
- ✅ Session cleanup loop for expired sessions
- ✅ SessionHandle for tracking individual connections
- ✅ Graceful shutdown with ShutdownController
- ✅ Tests for the session management infrastructure

### The Problem We Discovered

The forward proxy is fundamentally designed as a **tunnel** rather than a **proxy**:

1. **Current Design (Tunnel)**:
   - Takes a pre-created client transport and server transport
   - Creates a 1:1 forwarding loop between them
   - Each instance handles exactly one client-server pair

2. **What a Real Forward Proxy Needs**:
   - Listen on a TCP port (e.g., 8080)
   - Accept multiple incoming HTTP connections
   - Parse each HTTP request to determine the target
   - Create upstream connections on-demand based on the request
   - Route responses back to the correct client

### The HTTP Transport Issue

The `HttpIncoming` transport does create an axum server that can accept multiple connections, BUT:
- It serializes all requests through a single mpsc channel
- There's no per-connection tracking
- All requests go to the same pre-configured upstream
- No request-based routing

## The Real Fix Required

To make this work as a true forward proxy:

### 1. Redesign the Transport Layer
- HTTP incoming transport needs to track individual connections
- Each connection needs its own session ID
- Maintain connection-to-session mapping

### 2. Request-Based Routing
- Parse HTTP requests to extract target (Host header, URL)
- Dynamic upstream connection creation
- Connection pooling per upstream target

### 3. Response Routing
- Track which client connection each request came from
- Route responses back to the correct client
- Handle connection lifecycle properly

### 4. Remove Pre-Created Transport Assumption
- Forward proxy shouldn't take pre-created transports
- Should create transports on-demand based on requests
- Manage transport lifecycle internally

## Code That Needs Major Changes

1. **`src/transport/incoming/http.rs`**:
   - Currently channels all requests through one mpsc channel
   - Needs per-connection tracking

2. **`src/proxy/forward/`**:
   - Currently assumes pre-created transports
   - Needs to create transports dynamically

3. **`src/api.rs`**:
   - `forward_http()` creates one client and one server transport
   - Should create a listening server that spawns transports per request

## Session Registry Is Not Wrong, Just Premature

The session registry infrastructure we built isn't wrong - it's just managing something that doesn't exist yet. Once the transport layer properly creates sessions per connection, this infrastructure will be useful.

## Recommended Approach When Resuming

1. **Start with the Transport Layer**:
   - Fix `HttpIncoming` to track connections
   - Add connection-to-session mapping
   - Emit connection lifecycle events

2. **Then Fix the Forward Proxy**:
   - Remove the pre-created transport assumption
   - Add request parsing and routing
   - Create upstream connections dynamically

3. **Finally Wire Up Session Management**:
   - Use the registry we built
   - Track real sessions from real connections
   - Enable the cleanup loop for actual cleanup

## Why We're Pausing

This requires fundamental architectural changes at the transport layer, not just proxy-level session management. The scope is significantly larger than initially estimated, touching core transport abstractions that other components depend on.

## Estimation Update

Original estimate: 6-8 hours for Phase B
Revised estimate: 20-30 hours to properly redesign transport and proxy layers

## Files Created During This Work

- `src/proxy/forward/multi_session.rs` - Session management infrastructure (keep for later)
- `src/proxy/forward/session_handle.rs` - Session handle abstraction (useful)
- `src/proxy/forward/tests.rs` - Tests for session management (keep)
- `src/proxy/forward/mod.rs` - Module organization (keep)

These files contain useful infrastructure but need the underlying transport fixes to actually function.