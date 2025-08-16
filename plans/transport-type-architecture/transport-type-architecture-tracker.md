# Transport Type Architecture Tracker

## Overview

This tracker coordinates the refactoring of transport type handling in shadowcat to eliminate the `is_sse_session` code smell and create a cohesive, foundational transport architecture that properly models bidirectional proxy behavior.

**Last Updated**: 2025-08-16  
**Total Estimated Duration**: 20-30 hours  
**Status**: Phase A Complete - Ready for Implementation  
**Working Branch**: `feat/transport-type-architecture` (in shadowcat repo)

## Context

During Phase D.0 of the reverse proxy refactor, we discovered that the `is_sse_session` boolean field in the Session struct is a code smell indicating deeper architectural issues. Investigation revealed:

1. **TransportType is for session categorization**, not transport implementation
2. **We have two separate transport architectures**: Forward proxy uses clean directional traits, reverse proxy has duplicate logic
3. **The `is_sse_session` flag is actually tracking response mode**, not transport type
4. **We need proper bidirectional transport tracking** for proxy scenarios

## Goals

1. **Eliminate Code Smells** - Remove `is_sse_session` boolean and similar hacks
2. **Unify Transport Architecture** - Align forward and reverse proxy transport handling
3. **Model Bidirectional Nature** - Properly track client→proxy and proxy→upstream transports
4. **Improve Type Safety** - Use proper enums and types instead of boolean flags
5. **Clean Architecture** - No need for backward compatibility since shadowcat is unreleased

## Architecture Vision

```
Current State (Problematic):
┌─────────┐                    ┌─────────┐
│ Session │──transport_type──> │ Single  │
│         │──is_sse_session──> │ Boolean │ <- CODE SMELL
└─────────┘                    └─────────┘

Target State (Clean):
┌─────────┐──client_transport────> ┌────────────┐
│ Session │                        │ Directional│
│         │──upstream_transport──> │ Transports │
│         │──response_mode───────> │ ResponseMode│
└─────────┘                        └────────────┘
```

## Work Phases

### Phase A: Deep Analysis (8-10 hours)
Understand the full scope of the problem and design a comprehensive solution.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | **Transport Usage Audit** | 3h | None | ✅ Complete | | [Details](tasks/A.0-transport-usage-audit.md) - [Audit](analysis/transport-usage-audit.md) |
| A.1 | **Directional Transport Analysis** | 2h | None | ✅ Complete | | [Details](tasks/A.1-directional-transport-analysis.md) - [Analysis](analysis/directional-transport-analysis.md) |
| A.2 | **Response Mode Investigation** | 2h | None | ✅ Complete | | [Details](tasks/A.2-response-mode-investigation.md) - [Investigation](analysis/response-mode-investigation.md) |
| A.3 | **Architecture Proposal** | 3h | A.0, A.1, A.2 | ✅ Complete | | [Details](tasks/A.3-architecture-proposal.md) - [Proposal](analysis/architecture-proposal.md) |

**Phase A Total**: 10 hours

### Phase B: Quick Fix Implementation (4-6 hours)
Implement the immediate fix to eliminate the code smell.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.0 | **Add ResponseMode Enum** | 1h | A.3 | ⬜ Not Started | | [Details](tasks/B.0-add-response-mode.md) |
| B.1 | **Update Session Structure** | 2h | B.0 | ⬜ Not Started | | [Details](tasks/B.1-update-session-structure.md) |
| B.2 | **Migrate Usage Sites** | 2h | B.1 | ⬜ Not Started | | [Details](tasks/B.2-migrate-usage-sites.md) |
| B.3 | **Test and Validate** | 1h | B.2 | ⬜ Not Started | | [Details](tasks/B.3-test-validate.md) |

**Phase B Total**: 6 hours

### Phase C: Architectural Unification (8-10 hours)
Unify transport handling across forward and reverse proxies.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.0 | **Design Unified Architecture** | 3h | B.3 | ⬜ Not Started | | [Details](tasks/C.0-design-unified-architecture.md) |
| C.1 | **Refactor Reverse Proxy** | 4h | C.0 | ⬜ Not Started | | [Details](tasks/C.1-refactor-reverse-proxy.md) |
| C.2 | **Share Transport Implementations** | 2h | C.1 | ⬜ Not Started | | [Details](tasks/C.2-share-implementations.md) |
| C.3 | **Integration Testing** | 1h | C.2 | ⬜ Not Started | | [Details](tasks/C.3-integration-testing.md) |

**Phase C Total**: 10 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Phase A Completion Summary

**Completed**: 2025-08-16 (10 hours total)

### Key Deliverables
1. **[Transport Usage Audit](analysis/transport-usage-audit.md)** - Complete mapping of all TransportType usages
2. **[Directional Transport Analysis](analysis/directional-transport-analysis.md)** - Analysis of trait architecture
3. **[Response Mode Investigation](analysis/response-mode-investigation.md)** - Discovery that is_sse_session is dead code
4. **[Architecture Proposal](analysis/architecture-proposal.md)** - Comprehensive solution design
5. **[Implementation Roadmap](analysis/implementation-roadmap.md)** - Detailed step-by-step plan
6. **[Design Decisions](analysis/design-decisions.md)** - Rationale for architectural choices
7. **[Implementation Recommendations](analysis/implementation-recommendations.md)** - Specific implementation guidance
8. **[Distributed Storage Considerations](analysis/distributed-storage-considerations.md)** - SessionStore compatibility
9. **[Architecture Updates Summary](analysis/architecture-updates-summary.md)** - Refinements based on feedback
10. **[Naming Clarification](analysis/naming-clarification.md)** - ProxyCore vs UnifiedProxy explanation

### Critical Findings
- `is_sse_session` is completely unused (mark_as_sse_session never called)
- Response mode is per-response, not per-session
- Forward proxy architecture should be adopted by reverse proxy
- ~500 lines of duplicate code can be eliminated

### Recommended Next Steps
1. **Immediate**: Implement Phase B (Quick Fix) to add ResponseMode enum
2. **Short-term**: Phase C to extract shared transport logic
3. **Medium-term**: Phase D to unify proxy architectures

## Key Findings from Phase A Analysis

### Critical Discoveries
1. **`is_sse_session` is dead code** - The flag exists but `mark_as_sse_session()` is never called
2. **Response mode is per-response, not per-session** - Detected via Content-Type headers at runtime
3. **Forward proxy has clean architecture** - DirectionalTransports properly separate concerns
4. **Reverse proxy duplicates transport logic** - Direct HTTP client usage instead of traits
5. **TransportType conflates two concepts** - Session origin vs response format

### Architectural Insights
- The real issue is tracking **response format** (JSON vs SSE), not transport type
- SSE detection happens via `Content-Type: text/event-stream` header
- Response mode should be orthogonal to transport type
- DirectionalTransports can be adopted by reverse proxy with minimal changes

### Recommended Solution
1. **Add ResponseMode enum** to track JSON vs SSE vs future formats
2. **Remove is_sse_session** boolean completely (it's unused)
3. **Adopt DirectionalTransports** in reverse proxy for consistency
4. **Separate concerns**: TransportType for origin, ResponseMode for format

## Open Questions

These questions need to be answered during Phase A analysis:

1. **Naming Convention**: Should we rename `TransportType` to better reflect its purpose?
2. **Protocol Alignment**: Should we rename `Sse` to `StreamableHttp` to match MCP spec?
3. **Transport Pooling**: How does this affect the reverse proxy's connection pooling?
4. **Performance Impact**: Will tracking more transport state affect performance?
5. **Test Coverage**: What new test scenarios do we need for bidirectional transports?
6. **Session Storage**: Should we update how sessions are persisted?
7. **Transport Factory**: Should we extend the existing factory pattern?

## Success Criteria

### Functional Requirements
- ✅ `is_sse_session` boolean completely removed
- ✅ Proper bidirectional transport tracking implemented
- ✅ Response mode explicitly tracked with enum
- ✅ Forward and reverse proxies use same transport abstractions
- ✅ All existing tests continue to pass

### Code Quality Requirements
- ✅ No clippy warnings
- ✅ No duplicate transport logic between proxies
- ✅ Clear separation of concerns (transport vs response mode)
- ✅ Type-safe transport handling throughout

### Documentation Requirements
- ✅ Architecture decision documented
- ✅ Migration guide for existing code
- ✅ Updated API documentation
- ✅ Examples showing proper usage

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Performance regression | MEDIUM | Benchmark before/after, optimize hot paths | Planned |
| Complex refactor | MEDIUM | Two-phase approach: quick fix then full refactor | Active |
| Incomplete analysis | HIGH | Thorough Phase A analysis before implementation | Planned |
| Test coverage gaps | MEDIUM | Comprehensive test suite updates | Planned |

## Dependencies and Blockers

### Dependencies
- Existing `IncomingTransport`/`OutgoingTransport` traits in `src/transport/directional/`
- Current `TransportType` enum usage across codebase
- Session management infrastructure

### Current Blockers
- Need to complete Phase A analysis before committing to implementation approach
- Reverse proxy refactor (Phase D.1-D.3) is blocked until this is complete

## Related Documents

### Primary References
- [Initial Analysis](analysis/transport-type-architecture.md) - Investigation that led to this plan
- [Reverse Proxy Refactor](../reverse-proxy-refactor/tracker.md) - Parent plan that discovered this issue
- [MCP Spec Transports](~/src/modelcontextprotocol/specs/draft/basic/transports.mdx) - Protocol specification

### Analysis Outputs
- [Transport Usage Audit](analysis/transport-usage-audit.md) - ✅ Complete
- [Directional Transport Analysis](analysis/directional-transport-analysis.md) - ✅ Complete
- [Response Mode Investigation](analysis/response-mode-investigation.md) - ✅ Complete
- [Architecture Proposal](analysis/architecture-proposal.md) - ✅ Complete
- [Implementation Roadmap](analysis/implementation-roadmap.md) - ✅ Complete
- [Design Decisions](analysis/design-decisions.md) - ✅ Complete
- [Implementation Recommendations](analysis/implementation-recommendations.md) - ✅ Complete
- [Additional Analysis Documents](analysis/) - See directory for full list
- [Migration Guide](analysis/migration-guide.md) - To be created during Phase B implementation

## Next Actions

1. **Start Phase A Analysis** - Begin with A.0 Transport Usage Audit
2. **Document all TransportType usage** - Create comprehensive map
3. **Investigate ResponseMode patterns** - Understand what we're really tracking
4. **Design comprehensive solution** - Create architecture proposal

## Notes

- This is a foundational refactor that affects core proxy behavior
- We discovered this issue while implementing SSE reconnection support
- The fix will unblock several other improvements in the reverse proxy
- No backward compatibility needed - shadowcat is unreleased
- We can make breaking changes freely to get the architecture right
- Phase B (quick fix) can be done immediately, Phase C (unification) can be deferred
- **All changes should be made in the `feat/transport-type-architecture` branch**

---

**Document Version**: 1.0  
**Created**: 2025-08-16  
**Last Modified**: 2025-08-16  
**Author**: Transport Architecture Team

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-16 | 1.0 | Initial tracker creation from reverse proxy analysis | Team |
| 2025-08-16 | 1.1 | Completed Phase A analysis (A.0, A.1, A.2) with key findings | Analysis Team |
| 2025-08-16 | 1.2 | Completed A.3 architecture proposal with implementation roadmap | Architecture Team |