# Next Session: CLI Refactor - Analysis Phase

## Context
We're starting a CLI refactor project for Shadowcat to move functionality from the monolithic main.rs (1568 lines) into a well-organized cli module structure. The goal is to improve maintainability, testability, and reduce main.rs to a lean entry point.

## Current Status
- ✅ Project structure created at `plans/cli-refactor/`
- ✅ Project tracker created with 4 phases and detailed task breakdown
- ✅ Initial analysis shows main.rs has ~1568 lines that need refactoring
- ✅ Identified that tape, intercept, and session modules are already modularized

## Your Tasks for This Session

### Primary Task: Complete Analysis Phase (A.1-A.3)
Focus on completing the analysis phase to fully understand the refactoring scope:

1. **Task A.1**: Analyze main.rs structure (1 hour)
   - Read `plans/cli-refactor/tasks/A.1-analyze-main-structure.md`
   - Create the analysis deliverables in `plans/cli-refactor/analysis/`
   - Document all components, dependencies, and opportunities

2. **Task A.2**: Design module boundaries (1 hour)
   - Create task file at `plans/cli-refactor/tasks/A.2-design-module-boundaries.md`
   - Design the new module structure
   - Define clear interfaces between modules

3. **Task A.3**: Plan migration strategy (1 hour)
   - Create task file at `plans/cli-refactor/tasks/A.3-migration-strategy.md`
   - Plan incremental migration approach
   - Ensure zero downtime/breaking changes

## Key Files to Reference
- **Main file to analyze**: `shadowcat-cli-refactor/src/main.rs`
- **Existing CLI modules**: `shadowcat-cli-refactor/src/cli/`
- **Project tracker**: `plans/cli-refactor/cli-refactor-tracker.md`
- **Current task**: `plans/cli-refactor/tasks/A.1-analyze-main-structure.md`

## Important Notes
- We're working in the `shadowcat-cli-refactor` git worktree
- Don't modify any code yet - this session is analysis only
- Document everything in the `analysis/` directory
- Update the tracker after completing each task

## Success Criteria for This Session
- [ ] Complete component inventory of main.rs
- [ ] Document all dependencies and shared code
- [ ] Design the target module structure
- [ ] Create migration strategy document
- [ ] Update tracker with completed tasks
- [ ] All analysis documents created in `plans/cli-refactor/analysis/`

## Commands to Get Started
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor
cat plans/cli-refactor/cli-refactor-tracker.md
cat plans/cli-refactor/tasks/A.1-analyze-main-structure.md
```

Start with Task A.1 and work through the analysis phase systematically. Good luck!