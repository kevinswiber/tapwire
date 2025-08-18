# Next Session: Error Refactoring - Phase A Analysis

## Project Context

We're refactoring Shadowcat's error handling from centralized error types to module-local Error and Result types for better ergonomics and clarity, while maintaining full backward compatibility.

**Project**: Error & Result Modularization
**Tracker**: `plans/refactor-errors/refactor-errors-tracker.md`
**Status**: Phase A - Analysis & Planning (0% Complete)

## Current Status

### What Has Been Completed
- Plan structure created with all task files
- Comprehensive tracker with timeline and risk assessment
- Detailed task breakdowns for all phases

### What's In Progress
- **Phase A**: Analysis & Planning (Not Started)
  - A.0: Current State Inventory
  - A.1: Migration Impact Analysis
  - A.2: Compatibility Strategy

## Your Mission

Complete the analysis phase to understand the current error architecture and plan the migration strategy.

### Priority 1: Analysis Tasks (5 hours)

1. **A.0: Current State Inventory** (2h)
   - Document all error enums and their variants
   - Map Result type aliases and usage counts
   - Create From implementation graph
   - Identify public API exposure
   - Success: Complete inventory in `analysis/error-inventory.md`
   
2. **A.1: Migration Impact Analysis** (2h)
   - Assess public API impact
   - Identify import conflict risks
   - Create complexity matrix by module
   - Determine migration order
   - Success: Impact analysis in `analysis/migration-impact.md`

3. **A.2: Compatibility Strategy** (1h)
   - Design deprecation approach
   - Create migration tooling plan
   - Define version strategy
   - Success: Strategy document in `analysis/compatibility-strategy.md`

## Essential Context Files to Read

1. **Primary Tracker**: `plans/refactor-errors/refactor-errors-tracker.md` - Full project context
2. **Current Error Module**: `shadowcat/src/error.rs` - Existing error definitions
3. **Task Details**: 
   - `plans/refactor-errors/tasks/A.0-current-state-inventory.md`
   - `plans/refactor-errors/tasks/A.1-migration-impact-analysis.md`
   - `plans/refactor-errors/tasks/A.2-compatibility-strategy.md`

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Commands to Run First

```bash
# Check current error structure
rg "pub enum \w+Error" src/error.rs
rg "pub type \w+Result" src/error.rs

# Count usage across codebase
rg "TransportResult|SessionResult|StorageResult" --type rust -c

# Verify everything builds
cargo build
cargo test --lib
cargo clippy --all-targets -- -D warnings
```

## Implementation Strategy

### Phase 1: Inventory (2 hours)
1. List all error enums with variant counts
2. Document Result type aliases
3. Map From implementations
4. Count usage per module
5. Create comprehensive inventory document

### Phase 2: Impact Analysis (2 hours)
1. Identify public API changes
2. Find potential import conflicts
3. Assess module complexity
4. Determine safe migration order
5. Document risks and mitigations

### Phase 3: Strategy Design (1 hour)
1. Design deprecation messages
2. Plan re-export structure
3. Create migration tooling
4. Define version timeline
5. Draft migration guide outline

## Success Criteria Checklist

- [ ] Complete error inventory created
- [ ] Usage patterns documented
- [ ] Impact analysis complete
- [ ] Migration order determined
- [ ] Compatibility strategy defined
- [ ] All analysis documents in `analysis/` directory
- [ ] Tracker updated with findings

## Key Commands

```bash
# Analysis commands
rg "pub enum \w+Error" src/error.rs -A 5
rg "pub type \w+Result" src/error.rs
rg "impl From<.*Error>" src/error.rs

# Usage analysis
for error in Transport Session Storage Auth Config Intercept Recorder Proxy ReverseProxy; do
  echo "=== ${error}Error ==="
  rg "${error}Error" --type rust -g '!target' -c
done

# Public API check
rg "pub.*fn.*->.*Result" --type rust src/
```

## Important Notes

- **Always use TodoWrite tool** to track progress through analysis tasks
- **Document everything** - This analysis will guide the entire refactoring
- **Check public API carefully** - We must maintain backward compatibility
- **Consider all platforms** - Migration commands should work on macOS/Linux
- **Think about users** - Make migration as painless as possible

## Key Design Considerations

1. **Backward Compatibility**: All existing code must continue to compile
2. **Gradual Migration**: Users need time to adapt to new patterns
3. **Clear Documentation**: Migration path must be obvious
4. **Minimal Disruption**: Changes should be transparent to most users

## Risk Factors & Blockers

- **Public API Changes**: Must identify all public surfaces
- **Import Conflicts**: Multiple Result types could confuse
- **Documentation Clarity**: Must be crystal clear

## Next Steps After This Task

Once analysis is complete:
- **Phase B**: Implementation (B.1-B.4) - 7 hours
- **Phase C**: Testing & Documentation (C.1-C.2) - 4 hours

Total project estimate: 16 hours

## Model Usage Guidelines

- **IMPORTANT**: If context window approaches 70%, save progress and create new session
- Consider using focused searches rather than reading entire files

## Session Time Management

**Estimated Session Duration**: 5 hours
- Setup & Context: 15 min
- Inventory: 2 hours
- Impact Analysis: 2 hours
- Strategy Design: 1 hour
- Documentation: 15 min

---

**Session Goal**: Complete Phase A analysis with comprehensive documentation in the `analysis/` directory

**Last Updated**: 2025-01-18
**Next Review**: After Phase A completion