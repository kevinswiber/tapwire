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

**Last Updated**: 2025-08-19  
**Total Estimated Duration**: 16–22 hours  
**Status**: Planning

## Goals

1. **Generic Pool** - Provide `Pool<T>` that is transport-agnostic and reusable
2. **Clean API** - `Pool`, `PoolOptions`, `PoolConnection`, `PoolStats` with clear docs
3. **Reliability** - Close semantics, maintenance shut down, backpressure-safe returns
4. **Low Churn Migration** - Re-export old path temporarily; update imports incrementally

## Architecture Vision

```
shadowcat::pool
  ├── mod.rs           // re-exports, public API
  ├── inner.rs         // PoolInner<T>; Weak-backed maintenance; close event
  ├── connection.rs    // PoolConnection<T>; drop/return semantics
  ├── options.rs       // PoolOptions; knobs and (optional) hooks
  └── traits.rs        // PoolableResource

proxy/reverse/upstream/stdio.rs
  └── PoolableOutgoingTransport (adapter implementing PoolableResource)
```

## Work Phases

### Phase A: Analysis & Design
High-level design and API decisions.

| ID  | Task | Duration | Dependencies | Status | Notes |
|-----|------|----------|--------------|--------|-------|
| A.0 | Current State Analysis | 2h | None | ⬜ Not Started | tasks/A.0-current-state-analysis.md |
| A.1 | sqlx Patterns Review | 1h | None | ⬜ Not Started | tasks/A.1-sqlx-patterns-review.md |
| A.2 | Design Proposal & API | 2h | A.0–A.1 | ⬜ Not Started | tasks/A.2-design-proposal.md |

**Phase A Total**: 5h

### Phase B: Scaffolding & Migration Plan
Create module skeleton and plan migration.

| ID  | Task | Duration | Dependencies | Status | Notes |
|-----|------|----------|--------------|--------|-------|
| B.1 | Scaffold shadowcat::pool module | 1h | A.2 | ⬜ Not Started | tasks/B.1-scaffold-shadowcat-pool-module.md |
| B.2 | Adapter Strategy for transports | 1h | A.2 | ⬜ Not Started | tasks/B.2-adapter-strategy.md |
| B.3 | Migration Plan & Re-exports | 1h | A.2 | ⬜ Not Started | tasks/B.3-migration-plan-and-reexports.md |

**Phase B Total**: 3h

### Phase C: Core Implementation
Move pool implementation and update a first consumer.

| ID  | Task | Duration | Dependencies | Status | Notes |
|-----|------|----------|--------------|--------|-------|
| C.1 | Move generic pool | 3h | B.* | ⬜ Not Started | tasks/C.1-move-generic-pool.md |
| C.2 | Update stdio upstream | 2h | C.1 | ⬜ Not Started | tasks/C.2-update-stdio-upstream.md |
| C.3 | Type aliases & deprecations | 1h | C.1 | ⬜ Not Started | tasks/C.3-type-aliases-and-deprecations.md |

**Phase C Total**: 6h

### Phase D: Reliability Enhancements (optional)
Adopt sqlx-like refinements if valuable.

| ID  | Task | Duration | Dependencies | Status | Notes |
|-----|------|----------|--------------|--------|-------|
| D.1 | Add is_closed + CloseEvent | 2h | C.* | ⬜ Not Started | tasks/D.1-add-close-event-and-is-closed.md |
| D.2 | RAII capacity guard / fairness | 2h | C.* | ⬜ Not Started | tasks/D.2-raii-capacity-guard.md |
| D.3 | ArrayQueue + atomics (scale) | 3h | C.* | ⬜ Not Started | tasks/D.3-arrayqueue-and-atomics.md |
| D.4 | Health hooks | 2h | C.* | ⬜ Not Started | tasks/D.4-health-hooks.md |

**Phase D Total**: 9h (optional)

### Phase E: Testing & Benchmarks

| ID  | Task | Duration | Dependencies | Status | Notes |
|-----|------|----------|--------------|--------|-------|
| E.1 | Unit tests | 2h | C.* | ⬜ Not Started | tasks/E.1-unit-tests.md |
| E.2 | Integration tests (stdio) | 2h | C.* | ⬜ Not Started | tasks/E.2-integration-tests-stdio.md |
| E.3 | Benchmark harness | 2h | C.* | ⬜ Not Started | tasks/E.3-benchmark-harness.md |

**Phase E Total**: 6h

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (Aug 19–23)
- [ ] A.0: Current State Analysis
- [ ] A.1: sqlx Patterns Review
- [ ] A.2: Design Proposal & API

### Completed Tasks
- [x] H.0 pool reliability improvements (pre-refactor) - Completed Aug 19

## Success Criteria

### Functional Requirements
- ✅ {Requirement 1}
- ✅ {Requirement 2}
- ✅ {Requirement 3}

### Performance Requirements
- ✅ Preserve throughput and latency vs. current pool
- ✅ No additional allocations in hot path (no regressions)
- ✅ Support fairness or document behavior

### Quality Requirements
- ✅ {X}% test coverage
- ✅ No clippy warnings
- ✅ Full documentation
- ✅ Integration tests passing

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| {Risk description} | {HIGH/MEDIUM/LOW} | {Mitigation strategy} | {Active/Planned/Resolved} |

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

1. **{Immediate next step}**
2. **{Following step}**
3. **{Additional steps as needed}**

## Notes

- {Important notes about the project}
- {Any special considerations}
- {Dependencies or constraints}

---

**Document Version**: {X.Y}  
**Created**: {DATE}  
**Last Modified**: {DATE}  
**Author**: {Author/Team}

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| {DATE} | {X.Y} | {Description of changes} | {Author} |
