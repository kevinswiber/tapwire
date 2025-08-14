# Next Session: Phase 3 - Integration

## 🔴 CRITICAL: Use Git Worktree

**ALL SHADOWCAT WORK MUST BE DONE IN THE WORKTREE:**
```bash
cd shadowcat-tape-storage-providers
git status  # Verify: On branch feat/tape-storage-providers
```

**DO NOT MODIFY** shadowcat files in the main tapwire directory!

## Project Context

Integrating the new storage provider system with the TapeRecorder and public API.

**Project**: Tape Storage Providers
**Tracker**: `plans/tape-storage-providers/tape-storage-providers-tracker.md`
**Status**: Phase 3 - Integration (Ready to Start)
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

- **Phase 2: Built-in Providers** (✅ Completed 2025-08-14)
  - FilesystemProvider extracted from existing storage
  - MemoryProvider for testing with production warnings
  - Both providers implement TapeStorageBackend trait
  - Factories with proper registration support
  - Comprehensive conformance test suite
  - All tests passing (11 provider tests)
  - No clippy warnings

### What's Ready to Start
- **D.1: API Integration** (Ready)
  - Duration: 3 hours
  - Update TapeRecorder to use new backend system
  - Add public API for provider registration
  - Ensure backward compatibility

## Your Mission

Integrate the new storage provider system into the public API and TapeRecorder.

### Priority 1: API Integration (3 hours)

1. **Update TapeRecorder** (1.5h)
   - Replace direct TapeStorage usage with backend trait
   - Add factory method for provider selection
   - Ensure backward compatibility for default filesystem storage
   
2. **Public API** (1h)
   - Add `register_storage_provider()` to Shadowcat API
   - Add `with_storage()` to ShadowcatBuilder
   - Configuration file support for provider selection
   
3. **Integration Tests** (30m)
   - Test provider switching
   - Test configuration loading
   - Verify backward compatibility

### Priority 2: Migration Strategy (2 hours)

1. **Migration Utilities** (1h)
   - Tool to convert old storage to new format
   - Backward compatibility adapter
   
2. **Migration Documentation** (1h)
   - Step-by-step migration guide
   - Configuration examples
   - Troubleshooting section

## Essential Context Files to Read

1. **Primary Tracker**: `plans/tape-storage-providers/tape-storage-providers-tracker.md`
2. **Task D.1**: `plans/tape-storage-providers/tasks/D.1-api-integration.md`
3. **Recorder Module**: `shadowcat-tape-storage-providers/src/recorder/mod.rs`
4. **Main API**: `shadowcat-tape-storage-providers/src/lib.rs`
5. **Providers**: `shadowcat-tape-storage-providers/src/recorder/backend/providers/`

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

# Review the current TapeRecorder implementation
grep -r "TapeStorage" src/recorder/ --include="*.rs"

# Check the main API
cat src/lib.rs | head -100

# Run existing tests to ensure nothing is broken
cargo test --lib
```

## Success Criteria

- [ ] TapeRecorder uses new backend trait
- [ ] Public API supports provider registration
- [ ] Configuration files support provider selection
- [ ] Backward compatibility maintained
- [ ] All existing tests still pass
- [ ] Integration tests for provider switching

## Key Implementation Points

1. Default to filesystem provider for backward compatibility
2. Registry should be accessible from public API
3. Configuration parsing should handle provider settings
4. Error messages should be clear about provider issues
5. Document the new API methods thoroughly

## Deliverables

### Required
- Updated `src/recorder/mod.rs` with backend support
- Updated `src/lib.rs` with public API methods
- Integration tests in `tests/`
- Updated configuration handling

### Next Session Setup
- Phase 4: Migration & Documentation
- Create migration utilities
- Write comprehensive documentation

## Definition of Done

- [ ] Task D.1 completed
- [ ] Public API fully integrated
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Backward compatibility verified
- [ ] Tracker updated with progress
- [ ] This prompt updated for next session
- [ ] Commit: `feat(tape-storage): integrate storage providers with public API`

## Notes

- **USE THE WORKTREE**: All shadowcat code changes in `shadowcat-tape-storage-providers`
- Phase 2 providers are working well - filesystem and memory both tested
- Focus on maintaining backward compatibility
- Registry is already thread-safe and ready for use
- Consider how external providers will register themselves
- **When updating this prompt**: Always include the worktree reminder for next session

---

*Remember: 
1. **WORK IN THE WORKTREE** - `cd shadowcat-tape-storage-providers` first!
2. Phase 2 is complete - all providers working and tested
3. Integration should be seamless for existing users
4. External providers will use the same registration API
5. When creating the next session prompt, include the worktree reminder*