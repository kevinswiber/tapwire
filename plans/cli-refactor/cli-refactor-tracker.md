# CLI Refactor Tracker

## Overview

This tracker coordinates the refactoring of Shadowcat's CLI implementation to move core functionality from main.rs into a modular cli module structure, improving maintainability and testability.

**Last Updated**: 2025-01-09  
**Total Estimated Duration**: 12-16 hours  
**Status**: Planning

## Goals

1. **Modularize CLI** - Extract all command execution logic from main.rs into dedicated cli modules
2. **Improve Testability** - Enable unit testing of CLI commands without full binary compilation
3. **Reduce main.rs** - Keep main.rs lean, focused only on argument parsing and delegation
4. **Maintain Compatibility** - Ensure all existing CLI commands work identically after refactor

## Architecture Vision

```
main.rs (lean entry point)
    ├── Parse CLI args (clap)
    ├── Initialize logging
    └── Delegate to cli module
    
cli/
├── mod.rs           (public API, command dispatch)
├── forward.rs       (forward proxy commands)
├── reverse.rs       (reverse proxy commands)  
├── record.rs        (recording commands)
├── replay.rs        (replay commands)
├── tape.rs          (tape management - exists)
├── intercept.rs     (intercept management - exists)
├── session.rs       (session management - exists)
└── common.rs        (shared utilities, config)
```

## Work Phases

### Phase 1: Analysis & Design (Week 1)
Analyze current implementation and design the refactored module structure

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.1 | **Analyze main.rs structure** | 1h | None | ⬜ Not Started | | [Details](tasks/A.1-analyze-main-structure.md) |
| A.2 | **Design module boundaries** | 1h | A.1 | ⬜ Not Started | | [Details](tasks/A.2-design-module-boundaries.md) |
| A.3 | **Plan migration strategy** | 1h | A.2 | ⬜ Not Started | | [Details](tasks/A.3-migration-strategy.md) |

**Phase 1 Total**: 3 hours

### Phase 2: Core Infrastructure (Week 1)
Set up the basic cli module structure and common utilities

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.1 | **Create common utilities module** | 1h | A.3 | ⬜ Not Started | | [Details](tasks/B.1-common-utilities.md) |
| B.2 | **Create command dispatcher** | 1h | B.1 | ⬜ Not Started | | [Details](tasks/B.2-command-dispatcher.md) |
| B.3 | **Set up error handling** | 1h | B.1 | ⬜ Not Started | | [Details](tasks/B.3-error-handling.md) |

**Phase 2 Total**: 3 hours

### Phase 3: Command Migration (Week 1-2)
Migrate each command group to its dedicated module

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | **Migrate forward proxy commands** | 2h | B.2 | ⬜ Not Started | | [Details](tasks/C.1-forward-proxy.md) |
| C.2 | **Migrate reverse proxy command** | 1.5h | B.2 | ⬜ Not Started | | [Details](tasks/C.2-reverse-proxy.md) |
| C.3 | **Migrate record commands** | 1.5h | B.2 | ⬜ Not Started | | [Details](tasks/C.3-record-commands.md) |
| C.4 | **Migrate replay command** | 1h | B.2 | ⬜ Not Started | | [Details](tasks/C.4-replay-command.md) |

**Phase 3 Total**: 6 hours

### Phase 4: Testing & Cleanup (Week 2)
Add comprehensive tests and clean up main.rs

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.1 | **Add unit tests for CLI modules** | 2h | C.1-C.4 | ⬜ Not Started | | [Details](tasks/D.1-unit-tests.md) |
| D.2 | **Clean up main.rs** | 1h | C.1-C.4 | ⬜ Not Started | | [Details](tasks/D.2-cleanup-main.md) |
| D.3 | **Integration testing** | 1h | D.1, D.2 | ⬜ Not Started | | [Details](tasks/D.3-integration-tests.md) |

**Phase 4 Total**: 4 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (2025-01-09 to 2025-01-16)
- [ ] A.1: Analyze main.rs structure
- [ ] A.2: Design module boundaries
- [ ] A.3: Plan migration strategy
- [ ] B.1: Create common utilities module
- [ ] B.2: Create command dispatcher
- [ ] B.3: Set up error handling
- [ ] C.1: Migrate forward proxy commands

### Completed Tasks
(None yet)

## Success Criteria

### Functional Requirements
- ⬜ All existing CLI commands work identically
- ⬜ main.rs under 200 lines
- ⬜ Each command type has its own module
- ⬜ Shared configuration properly abstracted

### Code Quality Requirements
- ⬜ No clippy warnings
- ⬜ All public functions documented
- ⬜ Unit tests for each command module
- ⬜ Integration tests passing

### Maintainability Requirements
- ⬜ Clear module boundaries
- ⬜ Consistent error handling
- ⬜ Testable without full binary
- ⬜ Easy to add new commands

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Breaking existing CLI interface | HIGH | Comprehensive integration tests before/after | Planned |
| Complex interdependencies | MEDIUM | Careful analysis phase, incremental migration | Active |
| Session/rate limiting config duplication | MEDIUM | Extract to common module early | Planned |

## Key Findings from Analysis

### main.rs Current State (1568 lines)
- **CLI Structure**: Uses clap with nested subcommands
- **Major Commands**: forward, reverse, record, replay, tape, intercept, session
- **Helper Functions**: 
  - HTTP proxy handlers (3 functions, ~200 lines)
  - JSON conversion utilities (2 functions, ~50 lines)
  - Message matching helper (1 function, ~10 lines)
- **Configuration**: ProxyConfig struct with session/rate limiting settings
- **Duplication**: Rate limiter setup code repeated 4 times

### Existing CLI Modules
- `cli/tape.rs`: Already modularized
- `cli/intercept.rs`: Already modularized  
- `cli/session.rs`: Already modularized
- `cli/mod.rs`: Simple re-exports only

### Migration Opportunities
1. **ProxyConfig** → `cli/common.rs` (shared by forward/reverse/replay)
2. **Forward proxy logic** → `cli/forward.rs` (~400 lines)
3. **Reverse proxy logic** → `cli/reverse.rs` (~100 lines)
4. **Recording logic** → `cli/record.rs` (~250 lines)
5. **Replay logic** → `cli/replay.rs` (~200 lines)
6. **HTTP handlers** → `cli/handlers.rs` (shared utilities)

## Session Planning Guidelines

### Optimal Session Structure
1. **Review** (5 min): Check this tracker and relevant task files
2. **Implementation** (2-3 hours): Complete the task deliverables
3. **Testing** (30 min): Run tests, fix issues
4. **Documentation** (15 min): Update tracker, create PR if needed
5. **Handoff** (10 min): Update NEXT_SESSION_PROMPT.md if needed

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

## Related Documents

### Primary References
- [main.rs](../../src/main.rs)
- [CLI Module](../../src/cli/)

### Task Files
- [Analysis Tasks](tasks/)

### Specifications
- [Shadowcat Architecture](../002-shadowcat-architecture-plan.md)
- [Developer Guide](../003-shadowcat-developer-guide.md)

## Next Actions

1. **Complete analysis of main.rs structure**
2. **Document module boundaries and interfaces**
3. **Create migration plan with minimal disruption**

## Notes

- Existing tape, intercept, and session modules provide good patterns to follow
- Rate limiting and session configuration is heavily duplicated - prime candidate for extraction
- HTTP proxy handlers could be shared utilities or part of forward module
- Consider keeping clap structs in main.rs for clarity, moving only execution logic

---

**Document Version**: 1.0  
**Created**: 2025-01-09  
**Last Modified**: 2025-01-09  
**Author**: CLI Refactor Team

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-01-09 | 1.0 | Initial tracker creation and analysis | Team |