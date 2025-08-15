# Reverse Proxy Session Mapping Architecture

## Overview
Implement dual session ID tracking in the reverse proxy to properly handle MCP session management, SSE reconnection, and upstream failover scenarios.

## Problem Statement
Currently, the reverse proxy doesn't maintain its own session IDs separate from upstream servers, which causes issues with:
- SSE reconnection handling (Last-Event-Id tracking)
- Connection pooling and session migration
- Recording/replay with consistent session IDs
- Rate limiting and interceptor state management
- Upstream server failover and load balancing

## Goals
1. Maintain proxy-owned session IDs for client connections
2. Map proxy sessions to upstream server sessions
3. Handle SSE reconnection with event replay
4. Support upstream failover while maintaining client sessions
5. Enable connection pooling across multiple clients

## Technical Approach
- Add dual session ID tracking to Session struct (reverse proxy only)
- Implement session mapping table in reverse proxy
- Add SSE event buffering for reconnection
- Update reverse proxy handlers to translate session IDs
- Keep transport layer unchanged (use proxy IDs)
- Forward proxy remains single-session (no changes)

## Success Criteria
- [ ] Proxy generates and maintains its own session IDs
- [ ] Correct mapping between proxy and upstream sessions
- [ ] SSE reconnection works with Last-Event-Id
- [ ] Sessions survive upstream server changes
- [ ] All existing tests pass
- [ ] New integration tests for session mapping

## Timeline Estimate
- Research & Design: 2-3 hours
- Implementation: 4-6 hours  
- Testing: 2-3 hours
- Total: 8-12 hours

## Risk Assessment
- **Medium Risk**: Changes touch core session management
- **Mitigation**: Extensive testing, feature flag for rollback
- **Compatibility**: Must maintain backward compatibility