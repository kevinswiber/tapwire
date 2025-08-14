# Full Batch Support Implementation Plan

## Overview

This plan addresses the inconsistent batch message support in Shadowcat's MCP implementation. Currently, we have conflicting approaches: some areas explicitly reject batch messages while Phase 3 added full batch support to the protocol layer.

**Last Updated**: 2025-08-14  
**Total Estimated Duration**: 20-30 hours  
**Status**: Analysis Phase  
**Priority**: Medium (MCP doesn't require batch support, but JSON-RPC 2.0 allows it)

## Problem Statement

### Current Conflicts
1. **Batch Support DISABLED in:**
   - `src/interceptor/batch_handler.rs` - Returns "Batch messages not supported" error
   - `src/proxy/handlers.rs` - Explicitly checks and rejects batch messages
   - Various transport implementations reject arrays

2. **Batch Support ENABLED in:**
   - `src/transport/protocol/mod.rs` - Full batch support with serialize_batch/deserialize_batch
   - Comprehensive tests for batch handling
   - JSON-RPC 2.0 compliant batch processing

### Key Questions
- Should Shadowcat support batch messages?
- What are the performance implications?
- How would batch support affect interceptors and session management?
- What changes are needed for full implementation?

## Goals

1. **Analyze**: Determine if batch support is worth implementing
2. **Design**: Create architecture for batch message handling if needed
3. **Implement**: Either remove or fully implement batch support
4. **Test**: Ensure consistent behavior throughout the codebase
5. **Document**: Clear documentation of capabilities and limitations

## Work Phases

### Phase 0: Analysis and Decision (Week 1)
Understand requirements and make informed decision about batch support.

| ID | Task | Duration | Dependencies | Status | Notes |
|----|------|----------|--------------|--------|-------|
| A.0 | Analyze MCP specification requirements | 2h | None | ⬜ | Review spec for batch requirements |
| A.1 | Inventory current batch-related code | 3h | None | ⬜ | Find all batch handling locations |
| A.2 | Performance impact analysis | 2h | A.1 | ⬜ | Benchmark single vs batch |
| A.3 | Architecture impact assessment | 3h | A.1 | ⬜ | How batches affect system design |
| A.4 | Decision document | 2h | A.0-A.3 | ⬜ | Go/No-go recommendation |

**Phase 0 Total**: 12 hours

### Phase 1: Design (Week 1-2) - IF PROCEEDING
Design comprehensive batch support architecture.

| ID | Task | Duration | Dependencies | Status | Notes |
|----|------|----------|--------------|--------|-------|
| D.1 | Batch message flow design | 3h | A.4 | ⬜ | End-to-end batch handling |
| D.2 | Interceptor chain modifications | 2h | D.1 | ⬜ | How interceptors handle batches |
| D.3 | Session management updates | 2h | D.1 | ⬜ | Track batch requests/responses |
| D.4 | Error handling strategy | 2h | D.1 | ⬜ | Partial failures in batches |

**Phase 1 Total**: 9 hours

### Phase 2: Implementation (Week 2-3) - IF PROCEEDING
Implement full batch support across the codebase.

| ID | Task | Duration | Dependencies | Status | Notes |
|----|------|----------|--------------|--------|-------|
| I.1 | Update proxy handlers | 3h | D.1-D.4 | ⬜ | Remove batch rejection |
| I.2 | Modify interceptor engine | 4h | I.1 | ⬜ | Support batch processing |
| I.3 | Update session tracking | 2h | I.1 | ⬜ | Handle batch correlations |
| I.4 | Transport layer updates | 3h | I.1 | ⬜ | Ensure transports support arrays |
| I.5 | Error response handling | 2h | I.1-I.4 | ⬜ | Batch error responses |

**Phase 2 Total**: 14 hours

### Phase 3: Testing and Validation (Week 3)
Comprehensive testing of batch functionality.

| ID | Task | Duration | Dependencies | Status | Notes |
|----|------|----------|--------------|--------|-------|
| T.1 | Unit tests for batch handling | 2h | I.1-I.5 | ⬜ | Component-level tests |
| T.2 | Integration tests | 3h | I.1-I.5 | ⬜ | End-to-end batch flows |
| T.3 | Performance benchmarks | 2h | T.1-T.2 | ⬜ | Measure batch vs single |
| T.4 | Edge case testing | 2h | T.1-T.2 | ⬜ | Large batches, errors |

**Phase 3 Total**: 9 hours

### Alternative: Removal Phase (Week 1) - IF NOT PROCEEDING
Remove batch support for consistency.

| ID | Task | Duration | Dependencies | Status | Notes |
|----|------|----------|--------------|--------|-------|
| R.1 | Remove batch methods from protocol | 1h | A.4 | ⬜ | Clean up protocol layer |
| R.2 | Update tests | 1h | R.1 | ⬜ | Remove batch tests |
| R.3 | Document decision | 1h | R.1-R.2 | ⬜ | Explain why not supported |

**Removal Total**: 3 hours

## Key Code Locations

### Current Batch Rejection Points
- `src/interceptor/batch_handler.rs:18-45` - BatchHandler that returns errors
- `src/proxy/handlers.rs:~350` - Check for array messages
- `src/transport/stdio.rs` - Rejects JSON arrays
- `src/transport/http_server.rs` - Rejects batch requests

### New Batch Support
- `src/transport/protocol/mod.rs:138-193` - serialize_batch/deserialize_batch
- `src/transport/protocol/mod.rs:466-592` - Batch tests

### Related Components
- `src/session/manager.rs` - Would need batch correlation
- `src/interceptor/engine.rs` - Would need batch iteration
- `src/recorder/session_recorder.rs` - Would need batch recording

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Increased complexity | HIGH | Thorough design phase |
| Performance degradation | MEDIUM | Benchmarking and optimization |
| Interceptor complications | HIGH | Careful chain design |
| Breaking changes | MEDIUM | Feature flag for rollout |
| Session correlation issues | MEDIUM | Robust ID tracking |

## Success Criteria

### For Implementation
- [ ] All batch messages processed correctly
- [ ] No performance regression for single messages
- [ ] Interceptors handle batches properly
- [ ] Session tracking maintains correlation
- [ ] Comprehensive test coverage
- [ ] Clear documentation

### For Removal
- [ ] No batch support anywhere in codebase
- [ ] Consistent error messages for batch attempts
- [ ] Documentation explains limitation
- [ ] Clean, simple codebase

## Decision Factors

### Pro Batch Support
- JSON-RPC 2.0 compliance
- Potential performance gains for bulk operations
- Future-proofing for MCP evolution
- Better for high-throughput scenarios

### Against Batch Support
- MCP doesn't require it
- Significant complexity increase
- Limited real-world usage
- Complicates error handling
- Interceptor chain complexity

## Related Documents

- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Phase 3 Follow-up Analysis](../transport-refactor/phase3-followup-analysis.md)
- [Transport Refactor Tracker](../transport-refactor/transport-refactor-tracker.md)

## Notes

- This plan starts with analysis to make an informed decision
- Implementation phases only proceed if analysis recommends it
- Removal phase is simpler but needs careful documentation
- Consider feature flag for gradual rollout if implementing

---

**Document Version**: 1.0  
**Created**: 2025-08-14  
**Author**: Architecture Team