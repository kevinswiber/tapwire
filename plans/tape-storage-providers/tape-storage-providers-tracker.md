# Tape Storage Providers Tracker

## Overview

This tracker coordinates the implementation of a flexible, plugin-based storage backend system for tape recordings in Shadowcat. The system will allow users to provide custom storage implementations with a clean API design (no backwards compatibility needed since Shadowcat is pre-release).

**Last Updated**: 2025-08-14  
**Total Estimated Duration**: 27-32 hours  
**Status**: ✅ PROJECT COMPLETE - All Phases Done
**Git Worktree**: `shadowcat-tape-storage-providers` (branch: `feat/tape-storage-providers`)

### ⚠️ IMPORTANT: Development Environment
**All shadowcat development for this feature MUST happen in the git worktree:**
```bash
cd shadowcat-tape-storage-providers
git status  # Should show: On branch feat/tape-storage-providers
```

**DO NOT modify shadowcat files in the main worktree during this feature development.**

## Working Directory & Branch

- **Worktree Location**: `shadowcat-tape-storage-providers`
- **Branch**: `feat/tape-storage-providers` 
- **Commit Hash**: e57bb75
- **All code changes**: Make in the worktree, not main shadowcat directory
- **Session prompts**: Always include worktree reminder

## Goals

1. **Pluggable Storage** - Allow users to provide custom storage implementations
2. **Registry System** - Enable runtime registration of storage providers
3. **Configuration Flexibility** - Support configuration by provider name in config files
4. **Clean API** - Design optimal API without legacy constraints (pre-release advantage)
5. **Minimal Core** - Ship with filesystem (production) and memory (testing) providers only
6. **Cloud-Ready Design** - Ensure traits support object storage patterns for external providers

## Architecture Vision

```
┌──────────────────────────────────────────────────────────┐
│                     Public API                            │
│  Shadowcat::register_storage_provider()                   │
│  ShadowcatBuilder::with_storage()                        │
└─────────────────┬────────────────────────────────────────┘
                  │
┌─────────────────▼────────────────────────────────────────┐
│                   Storage Registry                        │
│  ┌─────────────────────────────────────────────────┐    │
│  │  HashMap<String, Arc<dyn StorageProviderFactory>>│    │
│  └─────────────────────────────────────────────────┘    │
│           ▲              ▲              ▲                 │
│           │              │              │                 │
│      filesystem      sqlite        custom-provider        │
└──────────────────────────────────────────────────────────┘
                  │
┌─────────────────▼────────────────────────────────────────┐
│                    TapeRecorder                           │
│  backend: Arc<RwLock<Box<dyn TapeStorageBackend>>>       │
└──────────────────────────────────────────────────────────┘
                  │
┌─────────────────▼────────────────────────────────────────┐
│             TapeStorageBackend (trait)                    │
├───────────────────────────────────────────────────────────┤
│  async fn initialize()                                    │
│  async fn save(tape) -> String                           │
│  async fn load(tape_id) -> Tape                          │
│  async fn delete(tape_id)                                │
│  async fn list() -> Vec<TapeIndexEntry>                  │
│  async fn search(criteria) -> Vec<TapeIndexEntry>        │
└──────────────────────────────────────────────────────────┘
         ▲                    ▲                    ▲
         │                    │                    │
┌────────┴─────────┐ ┌───────┴──────────┐ ┌──────┴────────┐
│FilesystemBackend │ │  MemoryBackend   │ │ CustomBackend │
│   (Production)   │ │  (Testing Only)  │ │  (External)   │
└──────────────────┘ └──────────────────┘ └───────────────┘
                                           Examples:
                                           - S3 Provider
                                           - Azure Blob
                                           - GCS Provider
```

## Work Phases

### Phase 0: Analysis & Investigation (Week 1)
Understand the current system and research best practices before implementation.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | **Current State Analysis** | 2h | None | ✅ Complete | 2025-08-14 | [Details](tasks/A.0-current-state-analysis.md) |
| A.1 | **Storage Patterns Research** | 2h | None | ✅ Complete | 2025-08-14 | [Details](tasks/A.1-storage-patterns-research.md) |
| A.2 | **Requirements Gathering** | 1.5h | A.0 | ✅ Complete | 2025-08-14 | [Details](tasks/A.2-requirements-gathering.md) |
| A.3 | **Design Proposal** | 2h | A.0, A.1, A.2 | ✅ Complete | 2025-08-14 | [Details](tasks/A.3-design-proposal.md) |

**Phase 0 Total**: 7.5 hours

### Phase 1: Core Abstractions (Week 1-2)
Define the core traits and registry system that will enable pluggable storage.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.1 | **Define TapeStorageBackend Trait** | 2h | A.3 | ✅ Complete | 2025-08-14 | [Details](tasks/B.1-core-trait-design.md) |
| B.2 | **Create StorageProviderFactory Trait** | 2h | B.1 | ✅ Complete | 2025-08-14 | [Details](tasks/B.2-factory-pattern.md) |
| B.3 | **Create Configuration Types** | 1h | B.2 | ✅ Complete | 2025-08-14 | [Details](tasks/B.3-configuration-system.md) |
| B.4 | **Implement Storage Registry** | 3h | B.2 | ✅ Complete | 2025-08-14 | [Details](tasks/B.4-registry-implementation.md) |

**Phase 1 Total**: 8 hours

### Phase 2: Built-in Providers (Week 2)
Implement filesystem and memory storage providers.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | **Filesystem Provider** | 3h | B.1 | ✅ Complete | 2025-08-14 | [Details](tasks/C.1-filesystem-provider.md) |
| C.2 | **Memory Provider (Testing)** | 2h | B.1 | ✅ Complete | 2025-08-14 | For testing only |
| C.3 | **Provider Testing** | 2h | C.1, C.2 | ✅ Complete | 2025-08-14 | [Details](tasks/C.3-provider-testing.md) |

**Phase 2 Total**: 7 hours

### Phase 3: Integration (Week 3)
Update TapeRecorder and public API to use the new system.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.1 | **API Integration** | 3h | B.4, C.3 | ✅ Complete | 2025-08-14 | [Details](tasks/D.1-api-integration.md) |

**Phase 3 Total**: 3 hours

### Phase 4: Documentation & Examples (Week 3)
Create comprehensive documentation and example providers.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| E.1 | **Documentation** | 2h | D.1 | ✅ Complete | 2025-08-14 | Created docs/storage-providers.md |
| E.2 | **Example Provider** | 2h | D.1 | ✅ Complete | 2025-08-14 | Created S3 provider example |

**Phase 4 Total**: 4 hours

## Phase Summary

| Phase | Description | Duration | Status |
|-------|-------------|----------|--------|
| Phase 0 | Analysis & Investigation | 7.5h | ✅ Complete |
| Phase 1 | Core Abstractions | 8h | ✅ Complete |
| Phase 2 | Built-in Providers | 7h | ✅ Complete |
| Phase 3 | Integration | 3h | ✅ Complete |
| Phase 4 | Documentation & Examples | 4h | ✅ Complete |
| **Total** | | **29.5 hours** | |

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Success Criteria

### Functional Requirements
- ✅ Users can register custom storage providers
- ✅ Configuration supports provider selection by name
- ✅ Built-in filesystem provider maintains backward compatibility
- ✅ SQLite provider offers database-backed storage
- ✅ Registry prevents duplicate provider registration
- ✅ Provider configuration is validated before use

### Performance Requirements
- ✅ < 1ms overhead for provider lookup
- ✅ No performance regression in filesystem storage
- ✅ Support concurrent tape operations
- ✅ Lazy initialization of storage backends

### Quality Requirements
- ✅ 90% test coverage for new code
- ✅ No clippy warnings
- ✅ Full trait documentation with examples
- ✅ Integration tests for all providers

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Breaking existing storage | HIGH | Maintain backward compatibility, extensive testing | Planned |
| Complex migration path | MEDIUM | Provide migration utilities and documentation | Planned |
| Provider naming conflicts | LOW | Registry validates unique names | Planned |
| Performance regression | MEDIUM | Benchmark before/after, optimize critical paths | Planned |

## Critical Implementation Guidelines

### Git Worktree Usage
**MANDATORY for all development:**
- Use `shadowcat-tape-storage-providers` worktree for ALL code changes
- Never modify shadowcat files in the main tapwire directory
- All testing and development happens in the worktree
- Commit to `feat/tape-storage-providers` branch
- When creating next-session-prompt.md updates, always include worktree reminder

### Backward Compatibility
**MUST maintain compatibility with existing filesystem storage:**
- Existing TapeStorage continues to work
- Configuration remains compatible
- No data migration required for filesystem users

### Thread Safety
**All components must be thread-safe:**
- Registry uses RwLock for concurrent access
- Backends implement Send + Sync
- No mutable statics without proper synchronization

### Error Handling
**Consistent error handling across providers:**
- All errors wrapped in RecorderError
- Validation happens early (in factory)
- Graceful degradation on storage failures

## API Design Examples

### Built-in Providers (In Shadowcat)
```rust
// Filesystem provider (default)
let shadowcat = Shadowcat::builder()
    .with_storage("filesystem", json!({
        "path": "./tapes"
    }))
    .build()?;

// Memory provider (testing only)
#[cfg(test)]
let shadowcat = Shadowcat::builder()
    .with_storage("memory", json!({}))
    .build()?;
```

### External Provider Example (Separate Crate)
```rust
// In external crate: shadowcat-s3-provider
struct S3Backend { /* ... */ }
impl TapeStorageBackend for S3Backend { /* ... */ }

struct S3ProviderFactory;
impl StorageProviderFactory for S3ProviderFactory {
    fn provider_name(&self) -> &str { "s3" }
    fn create(&self, config: Value) -> Result<Box<dyn TapeStorageBackend>> {
        // Parse config and create S3Backend
    }
}

// User registers the external provider
use shadowcat_s3_provider::S3ProviderFactory;
Shadowcat::register_storage_provider(Arc::new(S3ProviderFactory)).await?;
```

### Using in Configuration
```toml
[recording.storage]
provider = "s3"  # Uses registered provider
config = { bucket = "my-tapes", region = "us-east-1" }
```

### Programmatic Configuration
```rust
let shadowcat = Shadowcat::builder()
    .with_storage("s3", json!({
        "bucket": "my-tapes",
        "region": "us-east-1"
    }))
    .build()?;
```

## Next Actions

1. **Review and approve this plan**
2. **Create individual task files with detailed specifications**
3. **Begin Phase 1 implementation**
4. **Set up test infrastructure for provider testing**

## Notes

- This is a non-breaking change that enhances existing functionality
- Default behavior (filesystem storage) remains unchanged
- Memory provider is explicitly for testing only - will log warnings if used in production
- Cloud storage providers (S3, Azure, GCS) will be implemented as external crates
- Registry is global but can be overridden per Shadowcat instance if needed
- The trait design explicitly supports object storage patterns (put/get/list/delete)
- External providers can be community-maintained or official Anthropic crates

---

**Document Version**: 1.0  
**Created**: 2025-01-13  
**Last Modified**: 2025-01-13  
**Author**: Development Team

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-01-13 | 1.0 | Initial plan creation | Development Team |