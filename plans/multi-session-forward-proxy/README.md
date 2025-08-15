# Multi-Session Forward Proxy

## Overview
Enhance the forward proxy to handle multiple concurrent client connections, spawning a separate upstream connection for each client and managing them independently.

## Problem Statement
The current forward proxy implementation only handles a single client-server connection pair at a time. Once a client connects, the proxy:
- Accepts one connection
- Creates one upstream connection
- Forwards messages between them
- Exits when the connection closes

This is severely limiting for real-world usage where multiple clients need to connect through the same forward proxy instance.

## Goals
1. Accept multiple concurrent client connections
2. Spawn independent upstream connections for each client
3. Manage sessions independently with proper isolation
4. Support different transport types (stdio, HTTP, SSE)
5. Enable connection pooling and reuse where appropriate

## Technical Approach
- Refactor ForwardProxy to support multiple sessions
- Add connection accept loop for incoming transports
- Spawn independent tasks for each client-server pair
- Implement session registry for tracking active connections
- Add graceful shutdown for all active sessions
- Consider connection pooling for HTTP transports

## Success Criteria
- [ ] Forward proxy can handle N concurrent clients
- [ ] Each client gets its own upstream connection
- [ ] Sessions are properly isolated
- [ ] Clean shutdown terminates all connections gracefully
- [ ] Performance scales linearly with connection count
- [ ] All existing tests pass

## Timeline Estimate
- Research & Design: 2-3 hours
- Core Implementation: 6-8 hours
- Testing: 3-4 hours
- Total: 11-15 hours

## Risk Assessment
- **High Risk**: Breaking existing forward proxy functionality
- **Medium Risk**: Resource management with many connections
- **Low Risk**: Transport abstraction changes
- **Mitigation**: Feature flag for multi-session mode, extensive testing