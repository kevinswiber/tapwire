# Next Session: MCP Unified Architecture - Phase A

## Context
We're integrating hyper patterns, session management, and interceptors into the MCP crate for both client and server. This creates a production-ready, high-performance MCP implementation.

## Previous Work
- Completed comprehensive spawn audit showing 80% reduction opportunity
- Designed server architecture with SSE/WebSocket support
- Created SSE implementation guide with hyper v1 patterns
- **NEW**: Analyzed actual usage patterns in shadowcat proxy code
- **NEW**: Documented integration requirements based on real usage

## Current Focus: Phase A - Foundation Analysis
Complete analysis and design tasks to prepare for implementation.

## Critical Insights from Usage Analysis
- SessionManager and InterceptorChain always work together via shared AppState
- Sessions track protocol versions, upstream IDs, and event IDs for SSE
- Interceptors integrate deeply with pause controller and tape recorder
- Both forward and reverse proxies use same patterns

## Tasks for This Session

### 1. A.0: Inventory Session & Interceptor Code (4h)
**Goal**: Understand what exists in shadowcat vs MCP crate

**Actions**:
- Analyze `~/src/tapwire/shadowcat-mcp-compliance/src/session/`
- Analyze `~/src/tapwire/shadowcat-mcp-compliance/src/interceptor/`
- Compare with `crates/mcp/src/` current implementations
- Create inventory documents

**Deliverables**:
- `analysis/session-inventory.md`
- `analysis/interceptor-inventory.md`
- `analysis/dependency-map.md`

### 2. A.1: Design Unified Session Architecture (6h)
**Goal**: Design how sessions work in both client and server

**Key Decisions**:
- Session store trait design
- Persistence strategy
- SSE session tracking approach
- Client vs server session differences

**Deliverable**:
- `analysis/session-architecture.md`

### 3. A.2: Design Interceptor Integration (4h)
**Goal**: Design interceptor chain for client and server

**Key Decisions**:
- Interceptor trait for MCP
- Chain composition
- Async interceptor handling
- Error propagation

**Deliverable**:
- `analysis/interceptor-design.md`

## Key References
- **Tracker**: `plans/mcp-unified-architecture/mcp-unified-architecture-tracker.md`
- **Server Analysis**: `plans/mcp-unified-architecture/analysis/server-architecture.md`
- **SSE Guide**: `plans/mcp-unified-architecture/analysis/sse-implementation.md`

## Success Criteria
- [ ] Complete understanding of existing code
- [ ] Clear integration design documented
- [ ] No architectural conflicts identified
- [ ] Migration plan ready for Phase B

## Commands to Start
```bash
cd ~/src/tapwire/shadowcat-mcp-compliance

# Review the tracker
cat plans/mcp-unified-architecture/mcp-unified-architecture-tracker.md

# Start with inventory task
cat plans/mcp-unified-architecture/tasks/A.0-inventory-existing-code.md

# Examine session code
ls -la src/session/
ls -la src/interceptor/
```

## Important Notes
- We're in the MCP crate, not main shadowcat
- Session/interceptor code needs to be ported, not referenced
- Maintain compatibility with existing Connection trait
- Focus on understanding before implementing

## Time Estimate
14-18 hours total for Phase A. Focus on A.0 and A.1 first (10h), then A.2 if time permits.