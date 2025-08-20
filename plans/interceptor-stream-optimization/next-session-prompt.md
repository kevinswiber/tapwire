# Next Session Prompt - Interceptor Stream Optimization

## Quick Start
Begin the interceptor-stream-optimization plan by reviewing existing research and designing the batch processing API.

## Context
During Session 11 of the refactor-legacy-reverse-proxy plan, we discovered fundamental performance issues with SSE stream processing:
- Processing events individually with async overhead causes 50-100μs per event
- At 10,000 events/sec, this results in 60% CPU usage
- Solution: Batch processing at transport layer while keeping interceptors protocol-focused

## Current Session Goals
Focus on Phase A analysis and design tasks:
1. Review the existing research documents
2. Design a batch processing API for interceptors
3. Create a migration strategy

## Priority Tasks

### A.0: Review Existing Research (2 hours)
**Consolidate findings from the interceptor-streams research.**

Key documents to review:
- `shadowcat/docs/research/interceptor-streams/CLARIFICATION.md` - Architectural decisions
- `shadowcat/docs/research/interceptor-streams/root-cause-analysis.md` - Sync requirement issue
- `shadowcat/docs/research/interceptor-streams/transport-interceptor-analysis.md` - Transport layers

### A.1: Design Batch Processing API (3 hours)
**Design the batch interceptor trait and API.**

Consider:
- Backward compatibility with existing interceptors
- Configurable batch sizes and timeouts
- Stream transformation patterns
- Memory management and backpressure

### A.2: Migration Strategy (2 hours)
**Plan how to migrate from individual to batch processing.**

Define:
- Adapter pattern for legacy interceptors
- Incremental migration path
- Testing strategy
- Performance validation approach

## Success Criteria
- [ ] Research findings consolidated into analysis/README.md
- [ ] Batch API design documented in analysis/batch-api-design.md
- [ ] Migration strategy documented in analysis/migration-strategy.md
- [ ] Task files created for Phase B implementation
- [ ] Tracker updated with progress

## Key Constraints
- Must maintain backward compatibility with existing interceptors
- Interceptors must remain transport-agnostic
- Solution must work with existing SSE reconnection logic
- Performance target: <5μs per event

## Testing Commands
```bash
# Current performance baseline
cargo bench --bench sse_performance

# Test SSE streaming
cargo test --test test_reverse_proxy_sse

# Integration tests
cargo test --test e2e_resilience_test
```

## Files to Reference
```bash
# Research documents
shadowcat/docs/research/interceptor-streams/

# Current interceptor implementation
shadowcat/src/interceptor/

# SSE streaming code
shadowcat/src/proxy/reverse/upstream/http/streaming/

# Reconnection workaround
shadowcat/src/proxy/reverse/upstream/http/streaming/reconnect_simple.rs
```

---

**Note**: This plan focuses on design first. Implementation will follow in subsequent sessions after the design is validated.