# Next Session: Phase 1 - Core Abstractions Implementation

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
**Status**: Phase 1 - Core Abstractions (Ready to Start)
**Git Worktree**: `shadowcat-tape-storage-providers` (branch: `feat/tape-storage-providers`)

## Current Status

### What Has Been Completed
- **Phase 0: Analysis & Investigation** (✅ Completed 2025-08-14)
  - Current state assessment documented
  - Storage patterns researched (8 projects analyzed)
  - Requirements gathered (8 use cases defined)
  - API design proposal created
  - Design decisions documented

### What's Ready to Start
- **B.1: Define TapeStorageBackend Trait** (Ready)
  - Duration: 2 hours
  - Dependencies: Phase 0 analysis complete

## Your Mission

Implement the core abstractions for the tape storage provider system based on the completed analysis and design.

### Priority 1: Core Trait Definition (2 hours)

1. **Create trait module** (30m)
   - Set up `src/recorder/backend/mod.rs`
   - Define module structure
   - Add necessary imports
   
2. **Implement TapeStorageBackend trait** (1h)
   - Core async methods (save, load, delete, list, search)
   - Capability discovery
   - Configuration validation
   
3. **Define supporting types** (30m)
   - StorageConfig structure
   - StorageCapabilities enum
   - Error types

### Priority 2: Factory Pattern Implementation (2 hours)

1. **Create factory trait** (45m)
   - StorageProviderFactory definition
   - Metadata and versioning
   - Configuration validation
   
2. **Implement builder helpers** (45m)
   - Default implementations
   - Helper macros if needed
   - Testing utilities
   
3. **Documentation** (30m)
   - Trait documentation with examples
   - Provider implementation guide

### Priority 3: Configuration System (1 hour)

1. **Define configuration types** (30m)
   - StorageConfig with serde support
   - StorageOptions for common settings
   - Validation logic
   
2. **Environment variable support** (30m)
   - Config loading from env vars
   - Override mechanisms
   - Default values

## Essential Context Files to Read

1. **Primary Tracker**: `plans/tape-storage-providers/tape-storage-providers-tracker.md` - Full project context
2. **API Design**: `plans/tape-storage-providers/analysis/api-design-proposal.md` - Detailed API specification
3. **Design Decisions**: `plans/tape-storage-providers/analysis/design-decisions.md` - Key architectural choices
4. **Current Implementation**: `shadowcat-tape-storage-providers/src/recorder/` - Existing code to refactor (IN WORKTREE)
5. **Task B.1**: `plans/tape-storage-providers/tasks/B.1-core-trait-design.md` - Current task details

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

# Review the completed analysis
cat ../plans/tape-storage-providers/analysis/api-design-proposal.md
cat ../plans/tape-storage-providers/analysis/design-decisions.md

# Load Phase 1 task
cat ../plans/tape-storage-providers/tasks/B.1-core-trait-design.md

# Create new module structure (in worktree)
mkdir -p src/recorder/backend
touch src/recorder/backend/mod.rs
touch src/recorder/backend/traits.rs
touch src/recorder/backend/config.rs

# Start implementation
vim src/recorder/backend/traits.rs
```

## Success Criteria

- [ ] TapeStorageBackend trait fully defined with async methods
- [ ] StorageProviderFactory trait implemented
- [ ] Configuration types created with serde support
- [ ] All code compiles without warnings
- [ ] Basic tests for trait implementations
- [ ] Documentation complete for public APIs

## Key Implementation Points

1. Use `async-trait` crate for async trait methods
2. Ensure all types are Send + Sync for thread safety
3. Use serde_json::Value for flexible configuration
4. Maintain backward compatibility markers
5. Follow error handling patterns from analysis

## Deliverables

### Required
- `src/recorder/backend/traits.rs` - Core trait definitions
- `src/recorder/backend/config.rs` - Configuration types
- `src/recorder/backend/mod.rs` - Module exports

### Next Session Setup
- Registry implementation (Task B.4)
- Filesystem provider extraction (Task C.1)
- Integration with TapeRecorder

## Definition of Done

- [ ] Tasks B.1, B.2, and B.3 completed (or as many as time permits)
- [ ] Core traits compile and are well-documented
- [ ] Configuration system functional
- [ ] Tests pass with `cargo test`
- [ ] No clippy warnings
- [ ] Tracker updated with progress
- [ ] This prompt updated for next session
- [ ] Commit (in worktree): `feat(tape-storage): implement core storage provider abstractions`

## Notes

- **USE THE WORKTREE**: All shadowcat code changes in `shadowcat-tape-storage-providers`
- Phase 0 analysis is complete - we have a solid design foundation
- Start with minimal viable trait, then enhance incrementally
- Focus on getting the abstraction right - providers come later
- Ensure backward compatibility paths are clear
- **When updating this prompt**: Always include the worktree reminder for next session

---

*Remember: 
1. **WORK IN THE WORKTREE** - `cd shadowcat-tape-storage-providers` first!
2. The analysis is done - now we build based on the solid design foundation
3. Start simple, ensure it compiles, then add complexity
4. Test frequently to catch issues early
5. When creating the next session prompt, include the worktree reminder*