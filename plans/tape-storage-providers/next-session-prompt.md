# Next Session: Phase 4 - Migration & Documentation

## 🔴 CRITICAL: Use Git Worktree

**ALL SHADOWCAT WORK MUST BE DONE IN THE WORKTREE:**
```bash
cd shadowcat-tape-storage-providers
git status  # Verify: On branch feat/tape-storage-providers
```

**DO NOT MODIFY** shadowcat files in the main tapwire directory!

## Project Context

Create migration utilities and comprehensive documentation for the storage provider system.

**Project**: Tape Storage Providers
**Tracker**: `plans/tape-storage-providers/tape-storage-providers-tracker.md`
**Status**: Phase 4 - Migration & Documentation (Ready to Start)
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

- **Phase 3: API Integration** (✅ Completed 2025-08-14)
  - TapeRecorder updated to use storage backend trait
  - Factory methods added for provider selection
  - Public API methods for provider registration
  - ShadowcatBuilder with_storage() method
  - Lazy registration of built-in providers
  - Integration tests for provider switching
  - Backward compatibility maintained
  - All tests passing (4 integration tests)

### What's Ready to Start
- **E.1: Migration Strategy** (Ready)
  - Duration: 2 hours
  - Create migration utilities for existing storage
  - Backward compatibility adapter
  
- **E.2: Documentation & Examples** (Ready)
  - Duration: 2 hours
  - Comprehensive documentation
  - Example external provider implementation

## Your Mission

Create migration utilities and comprehensive documentation for the storage provider system.

### Priority 1: Migration Strategy (2 hours)

1. **Migration Utilities** (1h)
   - Create tool to convert old storage format to new
   - Backward compatibility adapter for smooth transition
   - Auto-detection of legacy storage directories
   
2. **Migration Documentation** (1h)
   - Step-by-step migration guide
   - Configuration examples for all providers
   - Troubleshooting section for common issues

### Priority 2: Documentation & Examples (2 hours)

1. **API Documentation** (1h)
   - Document all public APIs with examples
   - Provider development guide
   - Best practices for external providers
   
2. **Example Implementation** (1h)
   - Create example S3 provider skeleton
   - Show proper error handling patterns
   - Demonstrate async operations

## Essential Context Files to Read

1. **Primary Tracker**: `plans/tape-storage-providers/tape-storage-providers-tracker.md`
2. **Task E.1**: `plans/tape-storage-providers/tasks/E.1-migration-strategy.md`
3. **Task E.2**: `plans/tape-storage-providers/tasks/E.2-documentation.md`
4. **Backend Module**: `shadowcat-tape-storage-providers/src/recorder/backend/`
5. **API Module**: `shadowcat-tape-storage-providers/src/api.rs`

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

- [ ] Migration utility created for existing storage
- [ ] Backward compatibility adapter implemented
- [ ] Comprehensive documentation written
- [ ] Example external provider created
- [ ] Migration guide completed
- [ ] All documentation reviewed and accurate

## Key Implementation Points

1. Default to filesystem provider for backward compatibility
2. Registry should be accessible from public API
3. Configuration parsing should handle provider settings
4. Error messages should be clear about provider issues
5. Document the new API methods thoroughly

## Deliverables

### Required
- Migration utility in `src/recorder/backend/migration.rs`
- Documentation in `docs/storage-providers.md`
- Example provider in `examples/s3-provider/`
- Migration guide in `docs/migration-guide.md`

### Next Session Setup
- Project Complete!
- Ready for PR and merge

## Definition of Done

- [ ] Task E.1 completed
- [ ] Task E.2 completed  
- [ ] Migration utilities created
- [ ] Documentation comprehensive
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Tracker updated to show completion
- [ ] Ready for PR: `feat(tape-storage): add pluggable storage provider system`

## Notes

- **USE THE WORKTREE**: All shadowcat code changes in `shadowcat-tape-storage-providers`
- Phase 3 integration complete - all APIs working
- Focus on making migration smooth for existing users
- Documentation should be comprehensive for external provider developers
- Consider creating a crate template for external providers
- **When updating this prompt**: Always include the worktree reminder for next session

---

*Remember: 
1. **WORK IN THE WORKTREE** - `cd shadowcat-tape-storage-providers` first!
2. Phase 3 is complete - API integration fully working
3. Migration utilities should handle edge cases gracefully
4. Documentation is key for adoption - make it excellent
5. This is the final phase - aim for production readiness*