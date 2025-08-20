# Tracker Template

<!-- INSTRUCTIONS (DO NOT COPY THESE TO YOUR TRACKER):
This template provides a standard structure for development trackers in the Shadowcat project.
When creating a new tracker:
1. Copy everything BELOW the "START OF TEMPLATE" marker
2. Replace all placeholders marked with {PLACEHOLDER_NAME}
3. Customize sections as needed for your specific project
4. Delete any sections that don't apply
5. Add project-specific sections as needed

Key principles:
- Each task should be completable in one Claude session (2-4 hours)
- Dependencies should be clearly marked
- Status tracking should be consistent
- Include both functional and quality requirements
- Always consider both forward and reverse proxy modes
END OF INSTRUCTIONS -->

<!-- ==================== START OF TEMPLATE ==================== -->

# Refactor Pool to shadowcat::pool — Tracker

## Overview

Extract the generic connection pool from `proxy` into a top-level `shadowcat::pool` module, align with sqlx patterns, and integrate with current reverse proxy usage with minimal churn.

**Last Updated**: 2025-08-20  
**Total Estimated Duration**: 16–22 hours  
**Status**: Core impl + reverse stdio migrated; Cleanup & tests in progress

## Goals

1. **Generic Pool** - Provide `Pool<T>` that is transport-agnostic and reusable
2. **Clean API** - `Pool`, `PoolOptions`, `PoolConnection`, `PoolStats` with clear docs
3. **Reliability** - Close semantics, maintenance shut down, backpressure-safe returns
4. **Low Churn Migration** - Re-export old path temporarily; update imports incrementally

## Architecture (Actual)

```
shadowcat::pool
  ├── mod.rs           // Main implementation: Pool, PoolConnection, PoolOptions, PoolHooks
  └── traits.rs        // PoolableResource trait

proxy/reverse/upstream/stdio.rs
  └── OutgoingResource (wrapper implementing PoolableResource)
```

**Note**: Old `proxy::pool` module still exists but deprecated - to be removed.

## Work Phases

### Phase A: Analysis & Design
High-level design and API decisions.

| ID  | Task | Duration | Dependencies | Status | Notes |
|-----|------|----------|--------------|--------|-------|
| A.0 | Current State Analysis | 2h | None | ✅ Complete | Analyzed proxy pool patterns |
| A.1 | sqlx Patterns Review | 1h | None | ✅ Complete | Reviewed sqlx pool design |
| A.2 | Design Proposal & API | 2h | A.0–A.1 | ✅ Complete | API designed and implemented |

**Phase A Total**: 5h

### Phase B: Scaffolding & Migration Plan
Create module skeleton and plan migration.

| ID  | Task | Duration | Dependencies | Status | Notes |
|-----|------|----------|--------------|--------|-------|
| B.1 | Scaffold shadowcat::pool module | 1h | A.2 | ✅ Complete | Module created at src/pool/ |
| B.2 | Adapter Strategy for transports | 1h | A.2 | ✅ Complete | OutgoingResource wrapper impl |
| B.3 | Migration Plan & Re-exports | 1h | A.2 | ⚫ N/A | No backward compat needed |

**Phase B Total**: 3h

### Phase C: Core Implementation
Move pool implementation and update a first consumer.

| ID  | Task | Duration | Dependencies | Status | Notes |
|-----|------|----------|--------------|--------|-------|
| C.1 | Wire new pool in repo | 1h | B.* | ✅ Complete | New pool lives under `shadowcat::pool` in this worktree |
| C.2 | Update stdio upstream | 2h | C.1 | ✅ Complete | ReverseProxyServer now uses shadowcat::pool::Pool |
| C.3 | Type aliases & deprecations | 1h | C.2 | ⚫ Not Needed | No backward compatibility required |

**Phase C Total**: 6h

### Phase D: Reliability Enhancements (optional)
Adopt sqlx-like refinements if valuable.

| ID  | Task | Duration | Dependencies | Status | Notes |
|-----|------|----------|--------------|--------|-------|
| D.1 | Add is_closed + CloseEvent | 2h | C.* | ✅ Complete | Implemented: `Pool::is_closed()`, close event; acquire races shutdown |
| D.2 | RAII capacity guard / fairness | 2h | C.* | ⬜ Not Started | Current design has fairness via drop task; consider if spawn overhead observed |
| D.3 | ArrayQueue + atomics (scale) | 3h | C.* | ⬜ Not Started | tasks/D.3-arrayqueue-and-atomics.md |
| D.4 | Health hooks | 2h | C.* | ✅ Complete | Implemented SQLx-style hooks: after_create, before_acquire, after_release + metadata |

**Phase D Total**: 9h (optional)

### Phase E: Testing & Benchmarks

| ID  | Task | Duration | Dependencies | Status | Notes |
|-----|------|----------|--------------|--------|-------|
| E.1 | Unit tests | 2h | C.* | ✅ Complete | Added comprehensive stress tests with concurrency/exhaustion |
| E.2 | Integration tests (stdio) | 2h | C.* | ✅ Complete | Stress tests added in tests/pool_stress_test.rs |
| E.3 | Benchmark harness | 2h | C.* | ✅ Complete | Migrated to new pool API in benches/reverse_proxy_latency.rs |

**Phase E Total**: 6h

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (Aug 19–23)
- [x] A.0: Current State Analysis
- [x] A.1: sqlx Patterns Review
- [x] A.2: Design Proposal & API
- [x] C.1: Wire new pool in repo
- [x] C.2: Update stdio upstream - Completed Aug 20

### Completed Tasks
- [x] H.0 pool reliability improvements (pre-refactor) - Completed Aug 19
- [x] D.1 Close event + is_closed — Completed Aug 19
- [x] D.4 Health hooks (SQLx-style) — Completed Aug 19
- [x] C.2 Update stdio upstream to use shadowcat::pool::Pool - Completed Aug 20

## Old Pool Removal Checklist

- [x] Remove `shadowcat/src/proxy/pool.rs` - Completed Aug 20
- [x] Stop exporting pool in `shadowcat/src/proxy/mod.rs` - Completed Aug 20
- [x] Migrate files still using `proxy::pool` - All completed Aug 20:
  - [x] `tests/test_stdio_pool_reuse.rs`
  - [x] `tests/test_pool_reuse_integration.rs`
  - [x] `tests/test_subprocess_health.rs`
  - [x] `examples/test_pool_shutdown.rs`
  - [x] `benches/reverse_proxy_latency.rs`

## Notes
- **Migration Policy**: Breaking changes acceptable; no deprecation window required
- **Forward Proxy**: Does not use pooling (N/A for this refactor)
- **Metadata Bug**: ✅ FIXED - Now properly tracks creation time and idle time separately

## Success Criteria

### Functional Requirements
- ✅ Idle resource reuse working
- ✅ Acquire timeout enforced (2s default)
- ✅ Close cancels pending acquires
- ✅ SQLx-style hooks implemented
- ✅ Pool exhaustion handling verified (tests/pool_stress_test.rs)
- ✅ Heavy concurrency stress tested (100 concurrent, p95 ~113ms)

### Performance Requirements
- ✅ p95 acquire latency ~113ms at 100 concurrent (acceptable for subprocess spawning)
- ✅ < 5% end-to-end overhead for stdio echo (benchmarks passing)
- ✅ Memory < 1KB per idle connection (minimal ResourceMetadata overhead)

### Quality Requirements
- ✅ No clippy warnings
- ✅ Public API documented
- ✅ Integration tests (comprehensive coverage)
- ✅ Edge case test coverage > 80% (exhaustion, cancellation, max_lifetime)

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Spawn storm on drop path | MEDIUM | Bounded executor feature flag added | ✅ Mitigated |
| Idle queue contention under load | LOW | Consider lock-free queue (D.3) | Monitored |
| Hook misconfiguration causes hangs | MEDIUM | Timeout hooks, clear docs | Planned |
| Metadata.age always 0 | LOW | Fixed with ResourceMetadata tracking | ✅ Resolved |

## Session Planning Guidelines

### Next Session Prompt
Each plan should have a corresponding `next-session-prompt.md` file in the same directory as this tracker, based on the template in `plans/template/next-session-prompt.md`. This file should be updated at the end of each session to set up the next session with proper context.

### Optimal Session Structure
1. **Review** (5 min): Check this tracker and relevant task files
2. **Implementation** (2-3 hours): Complete the task deliverables
3. **Testing** (30 min): Run tests, fix issues
4. **Documentation** (15 min): Update tracker, create PR if needed
5. **Handoff** (10 min): Update next-session-prompt.md in this plan directory

### Using the rust-code-reviewer
For complex Rust implementation tasks, consider using the `rust-code-reviewer` subagent to:
- Review memory safety and ownership patterns
- Validate async/await correctness with tokio
- Check for performance optimizations
- Ensure proper error handling with Result types
- Verify test coverage for critical paths

### Context Window Management
- Each task is designed to require < 50% context window
- If approaching 70% usage, create next-session-prompt.md
- Keep focus on single task to avoid context bloat
- Reference documentation only when needed

### Task Completion Criteria
- [ ] All deliverables checked off
- [ ] Tests passing
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Tracker status updated

## Critical Implementation Guidelines

### Proxy Mode Parity
**ALWAYS implement changes in BOTH proxy modes:**
- **Forward Proxy** (`src/proxy/forward.rs`): Client → Shadowcat → Server
- **Reverse Proxy** (`src/proxy/reverse.rs`): Client → Shadowcat (HTTP) → Server

When implementing any MCP compliance feature:
1. ✅ Implement in forward proxy
2. ✅ Implement in reverse proxy  
3. ✅ Add tests for both modes
4. ✅ Verify behavior consistency

**Common oversights:**
- Version tracking (must track in both modes)
- Error handling (must be consistent)
- Session state management (must be synchronized)
- Protocol validation (must enforce equally)

## Communication Protocol

### Status Updates
After completing each task, update:
1. Task status in this tracker
2. Completion date and notes
3. Any new issues discovered
4. Next recommended task

### Handoff Notes
If context window becomes limited:
1. Save progress to next-session-prompt.md
2. Include:
   - Current task status
   - Completed deliverables
   - Remaining work
   - Any blockers or decisions needed

## Related Documents

### Primary References
- research/connection-pool-cleanup-gpt5/sqlx-overview.md
- research/connection-pool-cleanup-gpt5/patterns.md
- research/connection-pool-cleanup-gpt5/comparison-shadowcat.md
- plans/refactor-legacy-reverse-proxy/gpt-findings/*

### Task Files
- tasks/
  - See tasks under phases A–F in this directory

## Next Actions

✅ All critical actions completed:
1. ✅ **Fixed Metadata.age** - Now tracks created_at and last_idle_at separately
2. ✅ **Removed Old Pool** - Deleted proxy::pool.rs and migrated all files
3. ✅ **Stress Testing** - Added comprehensive tests with 100+ concurrent operations
4. ✅ **Benchmarks** - Measured performance, p95 acceptable for subprocess operations

## Follow-up Enhancements Completed

1. ✅ **Documentation** - Created MIGRATION.md with examples and profiles
2. ✅ **Observability** - Added metrics.rs with counters and gauges
3. ✅ **CI Guards** - Added perf-guard.yml workflow for regression testing
4. ✅ **Configuration Profiles** - Added profiles.rs with recommended settings
5. ✅ **Safety Valve** - Added bounded-return-executor feature flag

---

**Document Version**: 2.0  
**Created**: 2025-08-19  
**Last Modified**: 2025-08-20 (Pool Refactor Complete)  
**Author**: Pool Refactor Team

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-19 | 1.0 | Initial tracker creation | Pool Team |
| 2025-08-20 | 2.0 | Updated to reflect actual implementation status, added concrete criteria | Review Update |
| 2025-08-20 | 3.0 | Pool refactor complete - all tasks done, old pool removed, enhancements added | Completion |
