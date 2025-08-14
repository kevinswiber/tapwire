# Next Session: Phase 2 - Built-in Providers

## 🔴 CRITICAL: Use Git Worktree

**ALL SHADOWCAT WORK MUST BE DONE IN THE WORKTREE:**
```bash
cd shadowcat-tape-storage-providers
git status  # Verify: On branch feat/tape-storage-providers
```

**DO NOT MODIFY** shadowcat files in the main tapwire directory!

## Project Context

Implementing built-in storage providers (filesystem for production and memory for testing) using the core abstractions created in Phase 1. Cloud storage providers will be implemented as external crates.

**Project**: Tape Storage Providers
**Tracker**: `plans/tape-storage-providers/tape-storage-providers-tracker.md`
**Status**: Phase 2 - Built-in Providers (Ready to Start)
**Git Worktree**: `shadowcat-tape-storage-providers` (branch: `feat/tape-storage-providers`)

## Current Status

### What Has Been Completed
- **Phase 0: Analysis & Investigation** (✅ Completed 2025-08-14)
  - Current state assessment documented
  - Storage patterns researched (8 projects analyzed)
  - Requirements gathered (8 use cases defined)
  - API design proposal created
  - Design decisions documented

- **Phase 1: Core Abstractions** (✅ Completed 2025-08-14)
  - TapeStorageBackend trait with async methods
  - StorageProviderFactory trait with metadata
  - Global StorageRegistry for provider management
  - Configuration types with serde support
  - StorageCapabilities for feature discovery
  - Unified error handling with StorageError
  - 4 unit tests for registry functionality

### What's Ready to Start
- **C.1: Filesystem Provider** (Ready)
  - Duration: 3 hours
  - Extract and refactor existing filesystem storage
  
- **C.2: Memory Provider** (Ready)
  - Duration: 2 hours
  - Testing-only implementation with warnings for production use

## Your Mission

Implement the built-in storage providers that will ship with Shadowcat.

### Priority 1: Filesystem Provider (3 hours)

1. **Extract existing implementation** (1h)
   - Move code from `src/recorder/storage.rs`
   - Create `src/recorder/backend/providers/filesystem.rs`
   - Preserve all existing functionality
   
2. **Implement TapeStorageBackend trait** (1h)
   - Map existing methods to trait interface
   - Add missing trait methods
   - Ensure backward compatibility
   
3. **Create factory and tests** (1h)
   - FilesystemProviderFactory implementation
   - Configuration validation
   - Unit tests for all operations

### Priority 2: Memory Provider (2 hours)

1. **Design data structures** (30m)
   - HashMap for tape storage
   - Arc<RwLock> for thread safety
   - Size tracking and limits
   
2. **Implement backend** (1h)
   - Full TapeStorageBackend trait
   - Warning logs for production use
   - Configurable size limits
   
3. **Factory and tests** (30m)
   - MemoryProviderFactory with #[cfg(test)] recommendations
   - Unit tests for all operations
   - Concurrent access tests

### Priority 3: Provider Testing Framework (3 hours)

1. **Conformance test suite** (1.5h)
   - Standard tests all providers must pass
   - CRUD operations
   - Concurrent access
   - Error scenarios
   
2. **Performance benchmarks** (1h)
   - Save/load performance
   - Search performance
   - Memory usage
   
3. **Documentation** (30m)
   - Provider implementation guide
   - Configuration examples

## Essential Context Files to Read

1. **Primary Tracker**: `plans/tape-storage-providers/tape-storage-providers-tracker.md`
2. **Existing Storage**: `shadowcat-tape-storage-providers/src/recorder/storage.rs` - Current filesystem implementation
3. **Task C.1**: `plans/tape-storage-providers/tasks/C.1-filesystem-provider.md`
4. **Core Traits**: `shadowcat-tape-storage-providers/src/recorder/backend/traits.rs` - Interfaces to implement

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

# Create providers module
mkdir -p src/recorder/backend/providers
touch src/recorder/backend/providers/mod.rs
touch src/recorder/backend/providers/filesystem.rs
touch src/recorder/backend/providers/memory.rs
touch src/recorder/backend/providers/tests.rs

# Review existing storage implementation
cat src/recorder/storage.rs | head -100

# Start filesystem provider
vim src/recorder/backend/providers/filesystem.rs
```

## Success Criteria

- [ ] Filesystem provider extracted and working
- [ ] All existing filesystem tests still pass
- [ ] Memory provider fully implemented with production warnings
- [ ] Both providers pass conformance tests
- [ ] No performance regression in filesystem provider
- [ ] Documentation complete

## Key Implementation Points

1. Filesystem provider MUST be 100% backward compatible
2. Use existing TapeStorage code as reference
3. Memory provider should warn loudly if used outside tests
4. Both providers must be thread-safe
5. Design traits to naturally support object storage patterns

## Deliverables

### Required
- `src/recorder/backend/providers/filesystem.rs` - Extracted filesystem provider
- `src/recorder/backend/providers/memory.rs` - Memory provider for testing
- `src/recorder/backend/providers/tests.rs` - Conformance test suite
- Updated `src/recorder/backend/providers/mod.rs` - Module exports

### Next Session Setup
- Phase 3: Integration with TapeRecorder
- Update public API
- Migration utilities

## Definition of Done

- [ ] Tasks C.1, C.2, and C.3 completed
- [ ] Both providers fully functional
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Benchmarks show acceptable performance
- [ ] Tracker updated with progress
- [ ] This prompt updated for next session
- [ ] Commit: `feat(tape-storage): add filesystem and memory storage providers`

## Notes

- **USE THE WORKTREE**: All shadowcat code changes in `shadowcat-tape-storage-providers`
- Phase 1 provides solid foundation - build on it
- Filesystem extraction is critical - must not break existing users
- Memory provider is for testing only - make this VERY clear in code
- Cloud providers (S3, Azure, etc.) will be external crates - keep this in mind for trait design
- Test thoroughly - these are the only providers shipping with core
- **When updating this prompt**: Always include the worktree reminder for next session

---

*Remember: 
1. **WORK IN THE WORKTREE** - `cd shadowcat-tape-storage-providers` first!
2. Phase 1 abstractions are complete - now implement concrete providers
3. Backward compatibility is non-negotiable for filesystem
4. SQLite can be optimized without legacy constraints  
5. When creating the next session prompt, include the worktree reminder*