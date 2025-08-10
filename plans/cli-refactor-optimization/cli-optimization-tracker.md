# Shadowcat CLI Optimization Tracker

## Overview

This tracker coordinates the optimization of Shadowcat's CLI refactor to transform it from a functional CLI tool into a production-ready library and CLI. Based on the comprehensive review conducted in `reviews/cli-refactor/`, we need to address critical architectural issues that prevent library usage and improve overall code quality.

**Last Updated**: 2025-08-10  
**Total Estimated Duration**: 60-70 hours  
**Status**: Phase B In Progress (50% Complete)

## Goals

1. **Library-First Architecture** - Make Shadowcat usable as a Rust library with clean, ergonomic APIs
2. **Production Readiness** - Implement proper error handling, graceful shutdown, and comprehensive testing
3. **Developer Experience** - Provide clear documentation, examples, and consistent patterns

## Architecture Vision

```
Before (Current State):
┌─────────────────────────────────────┐
│           main.rs (140 lines)       │
│                  ↓                   │
│         pub mod cli (exposed)       │
│         ↓          ↓        ↓       │
│    forward.rs  reverse.rs  etc.     │
│         ↓          ↓        ↓       │
│    [Direct exit() calls]            │
│    [Config duplication]             │
│    [No abstraction layers]          │
└─────────────────────────────────────┘

After (Target State):
┌─────────────────────────────────────┐
│         Library (shadowcat)         │
│  ┌─────────────────────────────┐    │
│  │   Public API (lib.rs)       │    │
│  │   - Shadowcat facade        │    │
│  │   - Builder patterns        │    │
│  │   - Clean abstractions      │    │
│  └─────────────────────────────┘    │
│           ↓                          │
│  ┌─────────────────────────────┐    │
│  │   Internal modules          │    │
│  │   - Transport factories     │    │
│  │   - Config management       │    │
│  │   - Session handling        │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
                ↓
┌─────────────────────────────────────┐
│      CLI Binary (main.rs)           │
│  ┌─────────────────────────────┐    │
│  │  Private CLI module         │    │
│  │  - Thin wrapper over lib    │    │
│  │  - Argument parsing only    │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
```

## Work Phases

### Phase A: Critical Fixes (Days 1-2)
Remove immediate blockers to library usage

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.1 | **Make CLI Module Private** | 2h | None | ✅ Complete | | [Details](tasks/A.1-make-cli-private.md) |
| A.2 | **Remove Exit() Calls** | 2h | A.1 | ✅ Complete | | [Details](tasks/A.2-remove-exit-calls.md) |
| A.3 | **Fix Configuration Duplication** | 3h | A.1, A.2 | ✅ Complete | | [Details](tasks/A.3-fix-config-duplication.md) |

**Phase A Total**: 7 hours

### Phase B: Library Readiness (Days 3-6)
Create ergonomic library APIs and core functionality

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.1 | **Implement Builder Patterns** | 6h | Phase A | ✅ Complete | | [Details](tasks/B.1-builder-patterns.md) - All tests passing! |
| B.2 | **Add Graceful Shutdown** | 4h | B.1 | ✅ Complete | | [Details](tasks/B.2-graceful-shutdown.md) - 679 tests passing, full shutdown system |
| B.2.1 | **Fix Shutdown Integration** | 4h | B.2 | 🔴 Critical | | [Details](tasks/B.2.1-proper-shutdown-integration.md) - Fix shortcuts, proper proxy implementation |
| B.3 | **Create Library Facade** | 3h | B.1, B.2.1 | ⬜ Not Started | | [Details](tasks/B.3-library-facade.md) |
| B.4 | **Extract Transport Factory** | 3h | B.1 | ⬜ Not Started | | [Details](tasks/B.4-transport-factory.md) |
| B.5 | **Standardize Error Handling** | 2h | Phase A | ⬜ Not Started | | [Details](tasks/B.5-error-handling.md) |
| B.6 | **Add Basic Integration Tests** | 2h | B.1-B.5 | ⬜ Not Started | | [Details](tasks/B.6-integration-tests.md) |

**Phase B Total**: 24 hours (added B.2.1)

### Phase C: Quality & Testing (Days 7-12)
Polish, optimize, and prepare for production

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | **Comprehensive Documentation** | 4h | Phase B | ⬜ Not Started | | [Details](tasks/C.1-documentation.md) |
| C.2 | **Configuration File Support** | 3h | A.3 | ⬜ Not Started | | [Details](tasks/C.2-config-files.md) |
| C.3 | **Improve Error Messages** | 2h | B.5 | ⬜ Not Started | | [Details](tasks/C.3-error-messages.md) |
| C.4 | **Add Telemetry/Metrics** | 4h | Phase B | ⬜ Not Started | | [Details](tasks/C.4-telemetry.md) |
| C.5 | **Performance Optimization** | 6h | Phase B | ⬜ Not Started | | [Details](tasks/C.5-performance.md) |
| C.6 | **Extensive Test Coverage** | 6h | Phase B | ⬜ Not Started | | [Details](tasks/C.6-test-coverage.md) |
| C.7 | **CLI Shell Completions** | 2h | Phase A | ⬜ Not Started | | [Details](tasks/C.7-shell-completions.md) |
| C.8 | **Example Programs** | 3h | Phase B | ⬜ Not Started | | [Details](tasks/C.8-examples.md) |
| C.9 | **Connection Pooling** | 3h | B.4 | ⬜ Not Started | | [Details](tasks/C.9-connection-pooling.md) |
| C.10 | **Load Testing** | 2h | C.6 | ⬜ Not Started | | [Details](tasks/C.10-load-testing.md) |
| C.11 | **Release Preparation** | 2h | All above | ⬜ Not Started | | [Details](tasks/C.11-release-prep.md) |

**Phase C Total**: 37 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (2025-08-10 to 2025-08-16)
- [ ] A.1: Make CLI Module Private
- [ ] A.2: Remove Exit() Calls
- [ ] A.3: Fix Configuration Duplication
- [ ] B.1: Implement Builder Patterns (partial)
- [ ] B.2: Add Graceful Shutdown

### Week 2 (2025-08-17 to 2025-08-23)
- [ ] B.1: Implement Builder Patterns (complete)
- [ ] B.3: Create Library Facade
- [ ] B.4: Extract Transport Factory
- [ ] B.5: Standardize Error Handling
- [ ] B.6: Add Basic Integration Tests
- [ ] C.1: Comprehensive Documentation
- [ ] C.2: Configuration File Support

### Completed Tasks
*(None yet - project just starting)*

## Success Criteria

### Functional Requirements
- ⬜ Shadowcat builds as library without CLI features
- ⬜ No direct exit() calls in production code
- ⬜ Clean builder API for all major types
- ⬜ Graceful shutdown on Ctrl+C
- ⬜ Configuration from file and environment

### Performance Requirements
- ⬜ < 5% latency overhead for proxy operations
- ⬜ < 100MB memory for 1000 concurrent sessions
- ⬜ Support > 10,000 requests/second

### Quality Requirements
- ⬜ 70% test coverage minimum
- ⬜ No clippy warnings with `-D warnings`
- ⬜ Full documentation for public APIs
- ⬜ Integration tests for all commands
- ⬜ Examples for common use cases

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Breaking changes for existing users | HIGH | Use feature flags during transition | Planned |
| API design lock-in | HIGH | Review API with team before implementing | Planned |
| Async race conditions in shutdown | MEDIUM | Extensive testing with timeouts | Planned |
| Performance regression | MEDIUM | Benchmark before/after each phase | Planned |
| Scope creep | MEDIUM | Stick to defined phases, defer nice-to-haves | Active |

## Session Planning Guidelines

### Optimal Session Structure
1. **Review** (5 min): Check this tracker and relevant task files
2. **Implementation** (2-3 hours): Complete the task deliverables
3. **Testing** (30 min): Run tests, fix issues
4. **Documentation** (15 min): Update tracker, create PR if needed
5. **Handoff** (10 min): Update NEXT_SESSION_PROMPT.md if needed

### Using the rust-code-reviewer
For complex Rust implementation tasks, consider using the `rust-code-reviewer` subagent to:
- Review memory safety and ownership patterns
- Validate async/await correctness with tokio
- Check for performance optimizations
- Ensure proper error handling with Result types
- Verify test coverage for critical paths

### Context Window Management
- Each task is designed to require < 50% context window
- If approaching 70% usage, create NEXT_SESSION_PROMPT.md
- Keep focus on single task to avoid context bloat
- Reference documentation only when needed

### Task Completion Criteria
- [ ] All deliverables checked off
- [ ] Tests passing
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Tracker status updated

## Critical Implementation Guidelines

### Crate Structure Decision
**Decision**: Start with single crate using feature flags
```toml
[features]
default = []
cli = ["clap", "directories", "rustyline"]
```

### API Stability Policy
- Use `0.x` versioning until 6 months of production use
- Mark experimental features with `#[doc(hidden)]` or feature gates
- Document breaking changes in CHANGELOG.md

### Error Handling Pattern
All functions return `Result<T, ShadowcatError>`:
```rust
use anyhow::Context;
something.await.context("Failed to do something")?;
```

## Communication Protocol

### Status Updates
After completing each task, update:
1. Task status in this tracker
2. Completion date and notes
3. Any new issues discovered
4. Next recommended task

### Handoff Notes
If context window becomes limited:
1. Save progress to NEXT_SESSION_PROMPT.md
2. Include:
   - Current task status
   - Completed deliverables
   - Remaining work
   - Any blockers or decisions needed

## Related Documents

### Primary References
- [CLI Refactor Review](../../reviews/cli-refactor/README.md)
- [Improvement Recommendations](../../reviews/cli-refactor/improvement-recommendations.md)
- [Migration Plan](../../reviews/cli-refactor/migration-plan.md)
- [Technical Debt Assessment](../../reviews/cli-refactor/technical-debt.md)

### Task Files
- [Phase A Tasks](tasks/) - Critical fixes
- [Phase B Tasks](tasks/) - Library readiness
- [Phase C Tasks](tasks/) - Quality & testing

### Analysis Documents
- [Prioritization Analysis](analysis/prioritization.md)

## Next Actions

1. **🔴 B.2.1: Fix Shutdown Integration** - CRITICAL: Fix placeholder implementations (4 hours)
2. **B.3: Create Library Facade** - Design high-level API for common use cases (3 hours)
3. **B.4: Extract Transport Factory** - May just need refinement of existing TransportFactory (3 hours)
4. **B.5: Standardize Error Handling** - Improve error context and chaining (2 hours)
5. **B.6: Add Basic Integration Tests** - Test builders in real scenarios (2 hours)

## Progress Summary

- **Phase A**: 100% Complete (7 hours) ✅
- **Phase B**: 42% Complete (10 of 24 hours)
  - B.1: ✅ Builder Patterns (6h)
  - B.2: ✅ Graceful Shutdown (4h)
  - B.2.1: 🔴 CRITICAL - Fix Integration (4h) 
  - B.3-B.6: ⬜ Not Started (10h remaining)
- **Phase C**: 0% Complete (27 hours)
- **Overall**: ~23% Complete (17 of 74 hours)

## Notes

### B.2.1 Added (2025-08-10) - CRITICAL TECHNICAL DEBT
- Identified critical shortcuts taken in B.2 implementation
- CLI commands use placeholder code instead of real proxy loops
- Missing stdin/stdout client transport
- Bypassed builder patterns from B.1
- Must be fixed before B.3 to maintain design integrity

### B.2 Completion Summary (2025-08-10)
- Implemented comprehensive shutdown system with ShutdownController and ShutdownToken
- Added shutdown support to SessionManager, ForwardProxy, ReverseProxy, and TapeRecorder
- Integrated signal handling (Ctrl+C) in main.rs
- Created 7 comprehensive tests for shutdown scenarios
- Updated CLAUDE.md with 5 new clippy warning patterns discovered during implementation
- All 679 tests passing, no clippy warnings
- **WARNING**: Integration uses placeholder implementations - see B.2.1

- The refactor branch is in a git worktree at `/Users/kevin/src/tapwire/shadowcat-cli-refactor`
- Main branch already has the CLI refactor merged
- Estimated 2 weeks total effort for complete implementation
- Consider using the rust-code-reviewer agent for complex Rust changes
- Keep backwards compatibility where possible using feature flags

### Build Errors to Fix (from B.1 implementation)

**Critical Issues:**
1. **Interceptor trait mismatch** - The `intercept` method in builder interceptors returns wrong error type
   - Need to ensure all interceptors return `std::result::Result<InterceptAction, InterceptError>`
   - Files affected: `src/interceptor/builder.rs`

2. **HttpTransport constructor** - Takes 2 args (url: String, session_id: SessionId) but some tests use old API
   - Need to update remaining test files to use `from_url()` or new constructor
   - Files affected: `src/transport/http.rs`, test files

3. **Type ambiguity in builders** - Some `.into()` calls are ambiguous
   - Need explicit type annotations or use specific error constructors
   - Files affected: `src/transport/builders.rs`, `src/proxy/builders.rs`

4. **Unused imports** - Clean up unused imports flagged by clippy
   - Remove `InterceptError`, `tokio::sync::RwLock` etc.

**Files Modified in B.1:**
- `src/transport/builders.rs` - New file with transport builders
- `src/proxy/builders.rs` - New file with proxy builders  
- `src/session/builder.rs` - New file with session builder
- `src/interceptor/builder.rs` - New file with interceptor chain builder
- `src/transport/mod.rs` - Added builder exports
- `src/proxy/mod.rs` - Added builder exports
- `src/session/mod.rs` - Added builder exports
- `src/interceptor/mod.rs` - Added builder exports
- `src/transport/http.rs` - Updated with new constructor and methods
- `src/transport/size_limit_tests.rs` - Updated to use `from_url()`

---

**Document Version**: 1.0  
**Created**: 2025-08-10  
**Last Modified**: 2025-08-10  
**Author**: Claude (with Kevin)

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-10 | 1.0 | Initial tracker creation based on review | Claude |
| 2025-08-10 | 1.1 | Completed B.1 with builder patterns, noted build errors | Claude |