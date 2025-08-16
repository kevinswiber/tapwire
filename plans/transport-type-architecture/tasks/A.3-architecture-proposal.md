# Task A.3: Architecture Proposal

## Objective

Synthesize findings from A.0, A.1, and A.2 to create a comprehensive architecture proposal that eliminates code smells, unifies transport handling, and provides a clean foundation for future development.

## Background

After analyzing:
- Current TransportType usage (A.0)
- Directional transport architecture (A.1)  
- Response mode patterns (A.2)

We need to design a cohesive architecture that:
- Eliminates the `is_sse_session` boolean
- Unifies forward and reverse proxy transport handling
- Properly models bidirectional proxy behavior
- Provides clear type safety

Since shadowcat is unreleased, we have freedom to make breaking changes.

## Key Questions to Answer

1. What's the ideal transport architecture for shadowcat?
2. How do we model client→proxy and proxy→upstream transports?
3. Where does ResponseMode fit in the architecture?
4. Should we unify immediately or phase the changes?
5. What's the migration path from current to target state?

## Step-by-Step Process

### 1. Synthesis Phase (45 min)

Review and synthesize findings from previous analyses:

```bash
cd /Users/kevin/src/tapwire/plans/transport-type-architecture

# Review analysis outputs
cat analysis/transport-usage-audit.md
cat analysis/directional-transport-analysis.md
cat analysis/response-mode-investigation.md

# Look for patterns and commonalities
# Identify the core architectural issues
# Note opportunities for simplification
```

### 2. Design Phase (60 min)

Design the target architecture:

```rust
// Potential architecture
pub struct Session {
    pub id: SessionId,
    pub client_transport: TransportType,    // Or use directional traits?
    pub upstream_transport: TransportType,  
    pub response_mode: ResponseMode,
    // ... other fields
}

pub enum ResponseMode {
    Unknown,
    Json,
    SseStream,
}

// Should reverse proxy use directional transports?
impl ReverseProxy {
    pub async fn handle_request(
        &self,
        incoming: Box<dyn IncomingTransport>,
        outgoing: Box<dyn OutgoingTransport>,
    ) -> Result<()> {
        // Unified handling
    }
}
```

### 3. Planning Phase (45 min)

Create detailed implementation plan:
- What changes first
- What can be done in parallel
- How to validate each step
- Testing strategy

### 4. Documentation Phase (30 min)

Document the complete proposal.

## Expected Deliverables

### New Files
- `analysis/architecture-proposal.md` - Complete architecture proposal
- `analysis/implementation-roadmap.md` - Step-by-step implementation plan
- `analysis/design-decisions.md` - Rationale for key decisions

### Proposal Structure

```markdown
# Transport Type Architecture Proposal

## Executive Summary

### Problem Statement
- Code smells (is_sse_session)
- Duplicate transport logic
- Unclear transport modeling

### Proposed Solution
- Clean transport architecture
- Unified proxy handling
- Type-safe response modes

## Current State Analysis

### Key Issues
1. is_sse_session boolean tracks response mode, not transport
2. Forward and reverse proxies use different transport approaches
3. TransportType conflates configuration with runtime state

### Opportunities
1. Unify using existing directional transports
2. Model response modes explicitly
3. Clean breaking changes (no compatibility needed)

## Target Architecture

### Core Types
```rust
// Transport configuration
pub enum TransportType {
    Stdio,
    StreamableHttp, // Renamed from Sse
}

// Response handling
pub enum ResponseMode {
    Unknown,
    Json,
    SseStream,
}

// Session with bidirectional tracking
pub struct Session {
    pub id: SessionId,
    pub client_transport: TransportType,
    pub upstream_transport: TransportType,
    pub response_mode: ResponseMode,
    // Remove: is_sse_session
}
```

### Unified Proxy Architecture
- Both proxies use IncomingTransport/OutgoingTransport
- Shared transport implementations
- Consistent session management

## Implementation Roadmap

### Phase 1: Quick Fix (4-6 hours)
1. Add ResponseMode enum
2. Update Session structure
3. Replace is_sse_session usage
4. Update tests

### Phase 2: Transport Unification (8-10 hours)
1. Refactor reverse proxy to use directional transports
2. Share transport implementations
3. Unify session management
4. Comprehensive testing

## Design Decisions

### Why ResponseMode enum?
- is_sse_session is actually tracking response format
- Multiple response modes possible
- Type safety and clarity

### Why unify on directional transports?
- Eliminate code duplication
- Consistent behavior
- Better abstraction

### Why rename Sse to StreamableHttp?
- Match MCP spec terminology
- More accurate (handles both JSON and SSE)

## Benefits

### Immediate
- Eliminate code smell
- Clearer semantics
- Better type safety

### Long-term
- Easier maintenance
- Consistent behavior
- Foundation for new features

## Testing Strategy

### Unit Tests
- ResponseMode transitions
- Transport type handling
- Session management

### Integration Tests
- Forward proxy scenarios
- Reverse proxy scenarios
- Mixed transport configurations

### Performance Tests
- Benchmark before/after
- Memory usage
- Latency impact

## Migration Steps

### Step 1: Add new types
- ResponseMode enum
- Updated Session fields

### Step 2: Update usage sites
- Replace is_sse_session checks
- Update transport handling

### Step 3: Unify proxies
- Refactor reverse proxy
- Share implementations

### Step 4: Cleanup
- Remove old code
- Update documentation

## Risk Assessment

### Technical Risks
- Performance impact: Low (benchmark to verify)
- Complexity: Medium (phased approach)

### Mitigation
- Thorough testing
- Incremental changes
- Performance monitoring

## Success Metrics

- ✅ is_sse_session completely removed
- ✅ No duplicate transport logic
- ✅ All tests passing
- ✅ Performance maintained
- ✅ Clean architecture

## Recommendations

1. **Do Phase 1 immediately** - Quick win, unblocks SSE work
2. **Schedule Phase 2** - Plan for dedicated refactor time
3. **Update documentation** - Keep architecture docs current
```

## Success Criteria Checklist

- [ ] All findings synthesized from A.0, A.1, A.2
- [ ] Clear target architecture defined
- [ ] Implementation roadmap created
- [ ] Design decisions documented with rationale
- [ ] Benefits clearly articulated
- [ ] Testing strategy defined
- [ ] Risk assessment complete
- [ ] Success metrics identified

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Over-engineering | MEDIUM | Focus on solving identified problems |
| Missing requirements | HIGH | Review with team before implementing |
| Performance impact | MEDIUM | Benchmark critical paths |

## Duration Estimate

**Total: 3 hours**
- Synthesis: 45 minutes
- Design: 60 minutes
- Planning: 45 minutes
- Documentation: 30 minutes

## Dependencies

- A.0: Transport Usage Audit (must be complete)
- A.1: Directional Transport Analysis (must be complete)
- A.2: Response Mode Investigation (must be complete)

## Integration Points

- **Session Management**: Core changes to Session struct
- **Forward Proxy**: May need minor updates
- **Reverse Proxy**: Major refactor to use directional transports
- **Configuration**: Update transport type naming

## Notes

- This is the critical design document
- Get team review before proceeding to implementation
- Consider performance implications carefully
- Think about future extensibility

## Commands Reference

```bash
cd /Users/kevin/src/tapwire/plans/transport-type-architecture

# Review previous analyses
ls -la analysis/

# Check current implementation
cd ../../shadowcat
rg "struct Session" --type rust -A 10
rg "enum TransportType" --type rust -A 5

# Look at directional traits
cat src/transport/directional/mod.rs
```

---

**Task Status**: ⬜ Not Started
**Created**: 2025-08-16
**Last Modified**: 2025-08-16
**Author**: Transport Architecture Team