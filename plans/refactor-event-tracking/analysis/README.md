# Event Tracking Refactor - Analysis Documents

This directory contains the analysis and findings that led to the event tracking refactor.

## Documents

### [last-event-id-tracking-analysis.md](last-event-id-tracking-analysis.md)
**Created**: 2025-08-17

Comprehensive analysis of all Last-Event-Id tracking systems in Shadowcat, revealing:
- 5 overlapping tracking systems with no synchronization
- Multiple sources of truth causing potential bugs
- Redundant functionality across layers
- Proposed unified architecture with transport as authority

## Key Findings Summary

### The Problem
We discovered **5 different systems** tracking Last-Event-Id:

1. **Session Store Layer** - Persistent database storage
2. **SSE Session Integration** - Runtime per-connection tracking  
3. **Reverse Proxy SSE Resilience** - Per-session deduplication
4. **Transport Layer Event Tracking** - Core deduplication logic
5. **SSE Connection Level** - Raw wire-level tracking

These systems have:
- No synchronization between them
- Unclear ownership boundaries
- Potential for divergent state
- Redundant code and logic

### The Solution
Consolidate around the transport layer's `EventTracker` as the single authoritative source:
- Transport owns event tracking and deduplication
- Session store only persists for recovery
- One-way data flow: Transport → Session
- Remove redundant tracking systems

### Implementation Strategy
**Option A: Minimal Change** (Selected)
- 2-3 hours to implement
- Wire transport tracker to proxy
- Update session from transport
- Low risk approach

**Option C: Gradual Migration** (Future)
- Deprecate redundant systems over time
- Clean architecture as end goal
- Spread over multiple releases

## Impact

This analysis blocked the reverse proxy SSE resilience feature, leading to:
- Creation of this refactor plan
- Reverse proxy refactor put on hold
- Clear path to consolidation

## Related Work

- [Reverse Proxy Refactor](../../reverse-proxy-refactor/) - On hold pending this work
- [Transport SSE Implementation](../../../shadowcat/src/transport/sse/) - Contains authoritative EventTracker

## Next Steps

1. Implement Phase B (minimal integration)
2. Test with MCP Inspector  
3. Resume reverse proxy refactor
4. Plan gradual deprecation of redundant systems