# Next Session: Analysis & Design Phase

## Project Context

Refactoring the monolithic 3,465-line `legacy.rs` reverse proxy implementation into a clean, modular architecture. This is a focused extraction from the broader reverse-proxy-refactor plan to specifically address the legacy code issue.

**Project**: Refactor Legacy Reverse Proxy
**Tracker**: `plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`
**Status**: Phase A - Analysis & Design (0% Complete)

## Current Status

### What Has Been Completed
- **Critical Fix**: block_on deadlock in hyper_sse_intercepted.rs (✅ Completed 2025-08-18)
  - Implemented state machine pattern
  - Removed all blocking operations
  - All 20 reverse proxy tests passing

### What's In Progress
- **Task A.0**: Current State Analysis (Not Started)
  - Duration: 2 hours
  - Dependencies: None

## Your Mission

Perform comprehensive analysis of the legacy.rs file to understand its structure, dependencies, and design a clean modular architecture for the refactoring.

### Priority 1: Analysis Tasks (7 hours total)

1. **A.0 - Current State Analysis** (2h)
   - Map code structure and line counts
   - Identify major components
   - Document complexity hotspots
   - Create `analysis/current-structure.md`
   
2. **A.1 - Dependency Mapping** (2h)
   - Create dependency graphs
   - Identify extraction order
   - Document circular dependencies
   - Create `analysis/dependencies.md`

3. **A.2 - Module Design** (3h)
   - Design target architecture
   - Define module boundaries
   - Estimate line counts per module
   - Create `analysis/module-architecture.md`

### Priority 2: Interface Design (if time permits)
- **A.3 - Interface Design** (2h)
  - Define core traits
  - Design dependency injection
  - Plan testing strategy

## Essential Context Files to Read

1. **Primary Tracker**: `plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`
2. **Current Code**: `shadowcat/src/proxy/reverse/legacy.rs` - The 3,465-line file to analyze
3. **Fixed Module**: `shadowcat/src/proxy/reverse/hyper_sse_intercepted.rs` - Example of clean extraction
4. **Related Plan**: `plans/reverse-proxy-refactor/` - Original broader plan for context

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Commands to Run First

```bash
# Check current state
wc -l src/proxy/reverse/legacy.rs

# See what's already extracted
ls -la src/proxy/reverse/

# Run tests to ensure baseline
cargo test proxy::reverse --lib

# Check for warnings
cargo clippy --all-targets -- -D warnings
```

## Implementation Strategy

### Phase 1: Analysis (30 min)
1. Read through legacy.rs to understand overall structure
2. Identify major sections (config, server, handlers, admin UI)
3. Note embedded HTML and large constants

### Phase 2: Documentation (6 hours)
1. Create detailed structure analysis (A.0)
2. Map all dependencies (A.1)
3. Design clean module architecture (A.2)
4. Define interfaces if time permits (A.3)

### Phase 3: Validation (30 min)
1. Verify no circular dependencies in design
2. Confirm all modules under 500 lines
3. Check extraction order is feasible

## Success Criteria Checklist

- [ ] Complete structure analysis with line numbers
- [ ] Dependency graph created and documented
- [ ] Module architecture designed (<500 lines each)
- [ ] Natural boundaries identified
- [ ] Extraction order determined
- [ ] No circular dependencies in design

## Key Commands

```bash
# Analysis commands
grep -n "^impl\|^pub struct\|^pub fn" src/proxy/reverse/legacy.rs
rg "use crate::" src/proxy/reverse/legacy.rs | cut -d':' -f3 | sort | uniq -c

# Find large functions
awk '/^(pub )?async fn|^(pub )?fn/ {start=NR} /^}$/ {if(start) print NR-start, "lines at", start; start=0}' src/proxy/reverse/legacy.rs | sort -rn | head

# Check existing module usage
rg "legacy::" src/proxy/reverse/
```

## Important Notes

- **Focus on understanding before designing** - Don't jump to implementation
- **The admin UI is ~876 lines** - Likely easiest to extract
- **Config types are together** - Lines 56-336 approximately
- **Some streaming code already extracted** - Check what's in hyper_client.rs, etc.
- **Maintain backward compatibility** - Public API must not break

## Key Design Considerations

1. **Single Responsibility**: Each module does ONE thing well
2. **Testability**: Design interfaces that allow mocking
3. **No Circular Dependencies**: Plan extraction order carefully
4. **Feature Flags**: Admin UI should be optional

## Risk Factors & Blockers

- **Risk**: Breaking existing functionality - Mitigation: Incremental extraction with tests
- **Risk**: Circular dependencies - Mitigation: Design clean interfaces upfront
- **Risk**: Large PR - Mitigation: Plan for multiple smaller PRs

## Next Steps After This Task

Once analysis is complete:
- **B.0**: Extract Config Types (2 hours, depends on A.3)
- **B.1**: Extract Server Core (3 hours, depends on B.0)

After Phase A (Analysis):
- Move to Phase B (Core Extraction)
- Begin actual code refactoring

## Session Time Management

**Estimated Session Duration**: 7-9 hours
- Setup & Context: 30 min
- Analysis Tasks: 7 hours
- Documentation: 30 min
- Planning next steps: 30 min

---

**Session Goal**: Complete comprehensive analysis and design a clean, modular architecture for the reverse proxy refactor

**Last Updated**: 2025-08-18
**Next Review**: After A.2 is complete