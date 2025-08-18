# Next Session: Error Refactoring - Full Implementation

## Project Context

We're refactoring Shadowcat's error handling from centralized error types to module-local Error and Result types for better ergonomics and clarity. Since Shadowcat hasn't been released yet, we can make breaking changes without backward compatibility concerns.

**Project**: Error & Result Modularization
**Tracker**: `plans/refactor-errors/refactor-errors-tracker.md`
**Status**: Ready to implement (0% Complete)

## Current Status

### What Has Been Completed
- Plan structure created with all task files
- Simplified tracker without backward compatibility
- All tasks updated for clean migration

### What's Ready to Start
- **Phase A**: Analysis (1 hour)
- **Phase B**: Implementation (6 hours)
- **Phase C**: Testing & Documentation (2 hours)

## Your Mission

Complete the entire error refactoring in a single session, migrating from centralized to module-local error patterns.

### Full Implementation Plan (9 hours)

1. **A.0: Current State Inventory** (1h)
   - Document all error enums and variants
   - Map Result type aliases and usage
   - Create From implementation graph
   - Success: Complete inventory in `analysis/error-inventory.md`
   
2. **B.1: Add Module Re-exports** (1h)
   - Add Error and Result to each module
   - Update module documentation
   - Success: All modules have local types
   
3. **B.2: Update ShadowcatError** (1h)
   - Update From implementations to use module paths
   - Verify error conversions work
   - Success: Clean error flow from modules to top-level

4. **B.3: Migrate Internal Usage** (3h)
   - Update all files to use module patterns
   - Fix imports and type references
   - Success: No usage of old patterns

5. **B.4: Remove Old Aliases** (1h)
   - Delete all Result type aliases from error.rs
   - Clean up error.rs
   - Success: Clean error module

6. **C.1: Test Suite Updates** (1h)
   - Update test imports
   - Add error conversion tests
   - Success: All tests passing

7. **C.2: Documentation Updates** (1h)
   - Update rustdoc comments
   - Update README and examples
   - Success: Clear documentation

## Essential Context Files to Read

1. **Primary Tracker**: `plans/refactor-errors/refactor-errors-tracker.md`
2. **Current Error Module**: `shadowcat/src/error.rs`
3. **Key Task Files**: 
   - `plans/refactor-errors/tasks/A.0-current-state-inventory.md`
   - `plans/refactor-errors/tasks/B.3-migrate-internal-usage.md`

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

### Phase 1: Analysis (1 hour)
1. Document all error types
2. Map usage patterns
3. Create inventory document

### Phase 2: Add New Patterns (2 hours)
1. Add module re-exports
2. Update ShadowcatError
3. Verify everything still compiles

### Phase 3: Migration (4 hours)
1. Update all internal usage
2. Remove old aliases
3. Fix any compilation errors

### Phase 4: Finalize (2 hours)
1. Update all tests
2. Update documentation
3. Final verification

## Success Criteria Checklist

- [ ] Complete error inventory created
- [ ] All modules have Error and Result types
- [ ] ShadowcatError uses module paths
- [ ] All internal usage migrated
- [ ] Old aliases removed
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Clean, consistent error patterns

## Key Commands

```bash
# Analysis
rg "pub enum \w+Error" src/error.rs -A 5
rg "pub type \w+Result" src/error.rs

# Migration helpers
# For each module, add at top of mod.rs:
echo "pub use crate::error::TransportError as Error;" >> src/transport/mod.rs
echo "pub type Result<T> = std::result::Result<T, Error>;" >> src/transport/mod.rs

# Update usage (example for transport)
rg -l "TransportResult<" | xargs sed -i '' 's/TransportResult</Result</g'

# Verification
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

## Important Notes

- **No backward compatibility needed** - Shadowcat is pre-release
- **Clean break** - Remove all old patterns completely
- **Use TodoWrite tool** to track progress through tasks
- **Test frequently** - Run cargo build after each major change
- **Document as you go** - Update module docs with new patterns

## Key Design Principles

1. **Module Locality**: Errors belong with their modules
2. **Clear Hierarchy**: Module errors flow into ShadowcatError
3. **Ergonomics**: Use unqualified Result within modules
4. **Consistency**: Same pattern across all modules

## Risk Factors

- **Missing usage sites**: Use comprehensive search patterns
- **Import conflicts**: Use qualified paths when needed
- **Test failures**: Fix as encountered

## Session Time Management

**Estimated Session Duration**: 9 hours
- Analysis: 1 hour
- Implementation: 6 hours
- Testing & Docs: 2 hours

If time is limited, prioritize:
1. Analysis (essential)
2. Module re-exports (foundation)
3. Migration (can be incremental)

---

**Session Goal**: Complete full error refactoring with all old patterns removed

**Last Updated**: 2025-01-18
**Next Review**: After completion