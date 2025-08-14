# Next Session: Phase 0 - Analysis & Investigation

## 🔴 CRITICAL: Use Git Worktree

**ALL SHADOWCAT WORK MUST BE DONE IN THE WORKTREE:**
```bash
cd shadowcat-tape-storage-providers
git status  # Verify: On branch feat/tape-storage-providers
```

**DO NOT MODIFY** shadowcat files in the main tapwire directory!

## Project Context

Implementing a pluggable storage backend system for tape recordings in Shadowcat, allowing users to provide custom storage implementations beyond the default filesystem storage.

**Project**: Tape Storage Providers
**Tracker**: `plans/tape-storage-providers/tape-storage-providers-tracker.md`
**Status**: Phase 0 - Analysis & Investigation (0% Complete)
**Git Worktree**: `shadowcat-tape-storage-providers` (branch: `feat/tape-storage-providers`)

## Current Status

### What Has Been Completed
- **Plan Structure**: Upgraded to include analysis phase (✅ Completed 2025-08-13)
  - Added Phase 0 for analysis before implementation
  - Created analysis task files
  - Set up analysis output directory

### What's In Progress
- **A.0: Current State Analysis** (Not Started)
  - Duration: 2 hours
  - Dependencies: None

## Your Mission

Analyze the existing tape storage implementation and research best practices to inform the design of a pluggable storage provider system.

### Priority 1: Current State Analysis (2 hours)

1. **Explore existing implementation** (45m)
   - Map out current tape storage architecture
   - Document all storage-related operations
   - Identify integration points with recorder/replay
   
2. **Document limitations** (45m)
   - Performance bottlenecks
   - Missing features
   - User pain points
   
3. **Create assessment** (30m)
   - Write `analysis/current-state-assessment.md`
   - Include architecture diagrams

### Priority 2: Storage Patterns Research (2 hours)

1. **Research similar projects** (1h)
   - Database abstraction layers (sqlx, diesel)
   - Object storage systems
   - Plugin architectures in Rust
   
2. **Compile findings** (1h)
   - Document patterns in `analysis/storage-patterns-research.md`
   - Identify best practices and anti-patterns

### Priority 3: Requirements Gathering (1.5 hours) - If time permits

1. **Define use cases** (45m)
   - Local development needs
   - Production deployment scenarios
   - CI/CD requirements
   
2. **Establish requirements** (45m)
   - Performance targets
   - Security requirements
   - Configuration needs

## Essential Context Files to Read

1. **Primary Tracker**: `plans/tape-storage-providers/tape-storage-providers-tracker.md` - Full project context
2. **Task A.0**: `plans/tape-storage-providers/tasks/A.0-current-state-analysis.md` - First task details
3. **Task A.1**: `plans/tape-storage-providers/tasks/A.1-storage-patterns-research.md` - Research task
4. **Current Implementation**: `shadowcat-tape-storage-providers/src/recorder/` - Existing tape storage code (IN WORKTREE)

## Working Directory

```bash
# ALWAYS use the worktree for shadowcat code:
cd shadowcat-tape-storage-providers

# Verify you're in the right place:
git status  # Should show: On branch feat/tape-storage-providers
pwd        # Should end with: shadowcat-tape-storage-providers
```

## Commands to Get Started

```bash
# FIRST: Navigate to the worktree
cd shadowcat-tape-storage-providers
git status  # Verify: On branch feat/tape-storage-providers

# Review the tracker (from main tapwire)
cat ../plans/tape-storage-providers/tape-storage-providers-tracker.md

# Load first task
cat ../plans/tape-storage-providers/tasks/A.0-current-state-analysis.md

# Explore current tape storage (in worktree)
rg -t rust "tape" --files-with-matches
rg -t rust "TapeStorage|TapeRecorder" src/recorder/

# Check module structure
ls -la src/recorder/
cat src/recorder/mod.rs
```

## Success Criteria

- [ ] Current tape storage implementation fully understood and documented
- [ ] Architecture diagrams created
- [ ] At least 5 relevant projects researched for patterns
- [ ] Best practices and anti-patterns identified
- [ ] Analysis documents created in `analysis/` directory
- [ ] Tracker updated with task completion

## Key Questions to Answer

1. How is tape storage currently implemented?
2. What are the main limitations users face?
3. What patterns work well for storage abstraction in Rust?
4. What should our plugin architecture look like?
5. How do we maintain backward compatibility?

## Deliverables

### Required
- `analysis/current-state-assessment.md` - Complete analysis of existing implementation
- `analysis/storage-patterns-research.md` - Research findings and recommendations

### Optional (if time permits)
- `analysis/requirements-analysis.md` - User requirements and use cases
- Architecture diagrams (can use ASCII or markdown diagrams)

## Definition of Done

- [ ] Tasks A.0 and A.1 completed
- [ ] Analysis documents comprehensive and actionable
- [ ] Current limitations clearly documented
- [ ] Research provides clear recommendations
- [ ] Tracker updated with progress
- [ ] This prompt updated for next session
- [ ] Commit (in worktree): `feat(tape-storage): complete analysis phase for storage providers`

## Notes

- **USE THE WORKTREE**: All shadowcat code changes in `shadowcat-tape-storage-providers`
- This plan runs in parallel with transport-refactor (no code overlap)
- Shadowcat is pre-release - we can make breaking changes if beneficial
- Focus on understanding before proposing solutions
- Document everything for future reference
- **When updating this prompt**: Always include the worktree reminder for next session

---

*Remember: 
1. **WORK IN THE WORKTREE** - `cd shadowcat-tape-storage-providers` first!
2. The goal of this session is to thoroughly understand the problem space before designing solutions
3. Take time to explore and document findings properly
4. When creating the next session prompt, include the worktree reminder*