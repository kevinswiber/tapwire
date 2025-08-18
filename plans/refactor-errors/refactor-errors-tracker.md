# Error & Result Modularization Tracker

## Overview

This tracker coordinates the migration from a central error.rs holding all error enums and many *Result<T> aliases to an idiomatic layout with module-local Error and Result, while keeping a crate-wide ShadowcatError and crate::Result<T> for high-level APIs.

**Last Updated**: 2025-01-18  
**Total Estimated Duration**: 8-10 hours  
**Status**: Planning

## Goals

1. **Locality** - Each domain module exposes `pub use crate::error::XxxError as Error;` and `pub type Result<T> = std::result::Result<T, Error>;`
2. **Ergonomics** - High-level surfaces return `crate::Result<T>` (with ShadowcatError). Domain surfaces return their module `Result<T>`
3. **Clarity** - Consumers can pick precision (module Result) or convenience (crate::Result)
4. **Documentation** - Rustdoc shows errors in the module where they are used; crate root documents the top-level error

## Architecture Vision

```
Before:
  src/error.rs
    ├── ShadowcatError (top-level)
    ├── TransportError
    ├── SessionError
    ├── StorageError
    ├── AuthError
    ├── ConfigError
    ├── InterceptError
    ├── RecorderError
    ├── ProxyError
    └── ReverseProxyError
    + Type aliases: TransportResult<T>, SessionResult<T>, etc.

After:
  src/error.rs (keeps enum definitions centralized)
    └── ShadowcatError with #[from] module::Error variants
  
  src/transport/mod.rs
    ├── pub use crate::error::TransportError as Error;
    └── pub type Result<T> = std::result::Result<T, Error>;
  
  src/session/mod.rs
    ├── pub use crate::error::SessionError as Error;
    └── pub type Result<T> = std::result::Result<T, Error>;
  
  (similar for storage, auth, config, interceptor, recorder, proxy::reverse)
```

## Work Phases

### Phase A: Analysis & Planning (Week 1)
Understand current error usage patterns

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | **Current State Inventory** | 1h | None | ⬜ Not Started | | [Details](tasks/A.0-current-state-inventory.md) |

**Phase A Total**: 1 hour

### Phase B: Implementation (Week 1)
Execute the modularization

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.1 | **Add Module Re-exports** | 1h | A.0 | ⬜ Not Started | | [Details](tasks/B.1-add-module-reexports.md) |
| B.2 | **Update ShadowcatError** | 1h | B.1 | ⬜ Not Started | | [Details](tasks/B.2-update-shadowcat-error.md) |
| B.3 | **Migrate Internal Usage** | 3h | B.2 | ⬜ Not Started | | [Details](tasks/B.3-migrate-internal-usage.md) |
| B.4 | **Remove Old Aliases** | 1h | B.3 | ⬜ Not Started | | [Details](tasks/B.4-remove-old-aliases.md) |

**Phase B Total**: 6 hours

### Phase C: Testing & Documentation (Week 1)
Validate changes and update documentation

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | **Test Suite Updates** | 1h | B.4 | ⬜ Not Started | | [Details](tasks/C.1-test-suite-updates.md) |
| C.2 | **Documentation Updates** | 1h | C.1 | ⬜ Not Started | | [Details](tasks/C.2-documentation-updates.md) |

**Phase C Total**: 2 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (2025-01-18 to 2025-01-24)
- [ ] A.0: Current State Inventory
- [ ] B.1: Add Module Re-exports
- [ ] B.2: Update ShadowcatError
- [ ] B.3: Migrate Internal Usage
- [ ] B.4: Remove Old Aliases
- [ ] C.1: Test Suite Updates
- [ ] C.2: Documentation Updates

### Completed Tasks
(None yet)

## Success Criteria

### Functional Requirements
- ✅ Each module has local Error and Result types
- ✅ ShadowcatError aggregates all module errors via #[from]
- ✅ Clean migration with no legacy code

### Performance Requirements
- ✅ No runtime performance impact
- ✅ Compile time increase < 5%
- ✅ Binary size increase < 1%

### Quality Requirements
- ✅ No clippy warnings
- ✅ Full rustdoc documentation
- ✅ All tests passing
- ✅ Example code updated
- ✅ Clean codebase with no deprecated patterns

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Import confusion with multiple Result types | MEDIUM | Use qualified paths in docs, avoid glob imports | Planned |
| Merge conflicts during migration | MEDIUM | Complete in single PR, coordinate with team | Planned |
| Missing some usage sites | MEDIUM | Use comprehensive search patterns | Planned |
| Documentation becomes unclear | LOW | Add module-level error docs, cross-reference | Planned |

## Session Planning Guidelines

### Next Session Prompt
See `next-session-prompt.md` in this directory for the current session setup.

### Optimal Session Structure
1. **Review** (5 min): Check this tracker and relevant task files
2. **Implementation** (2-3 hours): Complete the task deliverables
3. **Testing** (30 min): Run tests, fix issues
4. **Documentation** (15 min): Update tracker, create PR if needed
5. **Handoff** (10 min): Update next-session-prompt.md

### Context Window Management
- Each task is designed to require < 50% context window
- If approaching 70% usage, create next-session-prompt.md
- Keep focus on single task to avoid context bloat

### Task Completion Criteria
- [ ] All deliverables checked off
- [ ] Tests passing
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Tracker status updated

## Critical Implementation Guidelines

### Migration Principles
1. **Clean Break**: Remove all old patterns in one go
2. **Comprehensive**: Update all usage sites
3. **Clear Documentation**: Module-level error handling patterns
4. **Consistency**: Same pattern across all modules

### Error Handling Patterns
```rust
// Module-local (precise)
use crate::transport::Result;
fn transport_operation() -> Result<Data> { ... }

// High-level (convenient)
use crate::Result;
fn orchestrate() -> Result<()> { ... }
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
1. Save progress to next-session-prompt.md
2. Include:
   - Current task status
   - Completed deliverables
   - Remaining work
   - Any blockers or decisions needed

## Related Documents

### Primary References
- [Current error.rs](../../shadowcat/src/error.rs)
- [Rust API Guidelines - Errors](https://rust-lang.github.io/api-guidelines/interoperability.html#error-types)

### Task Files
- [Analysis Tasks](tasks/)
- [Implementation Tasks](tasks/)
- [Testing Tasks](tasks/)

### Output Documents
- [Analysis Results](analysis/)
- [Migration Guide](analysis/migration-guide.md) (to be created)

## Next Actions

1. **Start with A.0**: Inventory current error usage
2. **Analyze impact**: Understand what needs migration
3. **Design compatibility layer**: Plan deprecation strategy

## Notes

- Keep actual enum definitions centralized in error.rs to avoid churn
- Modules simply re-export them as Error and define a local Result alias
- Update all usage sites in a single PR
- Document the new patterns clearly in module docs

---

**Document Version**: 1.0  
**Created**: 2025-01-18  
**Last Modified**: 2025-01-18  
**Author**: Kevin

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-01-18 | 1.0 | Initial plan creation | Kevin |