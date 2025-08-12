# Tape Storage Providers Tracker

## Overview

This tracker coordinates the implementation of a flexible, plugin-based storage backend system for tape recordings in Shadowcat. The system will allow users to provide custom storage implementations with a clean API design (no backwards compatibility needed since Shadowcat is pre-release).

**Last Updated**: 2025-01-13  
**Total Estimated Duration**: 20-25 hours  
**Status**: Planning

## Goals

1. **Pluggable Storage** - Allow users to provide custom storage implementations
2. **Registry System** - Enable runtime registration of storage providers
3. **Configuration Flexibility** - Support configuration by provider name in config files
4. **Clean API** - Design optimal API without legacy constraints (pre-release advantage)
5. **Built-in Providers** - Include filesystem and SQLite implementations

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
│FilesystemBackend │ │  SqliteBackend   │ │ CustomBackend │
└──────────────────┘ └──────────────────┘ └───────────────┘
```

## Work Phases

### Phase 1: Core Abstractions (Week 1)
Define the core traits and registry system that will enable pluggable storage.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.1 | **Define TapeStorageBackend Trait** | 2h | None | ⬜ Not Started | | [Details](tasks/A.1-storage-backend-trait.md) |
| A.2 | **Create StorageProviderFactory Trait** | 2h | A.1 | ⬜ Not Started | | [Details](tasks/A.2-provider-factory-trait.md) |
| A.3 | **Implement Storage Registry** | 3h | A.2 | ⬜ Not Started | | [Details](tasks/A.3-storage-registry.md) |
| A.4 | **Create Configuration Types** | 1h | A.2 | ⬜ Not Started | | [Details](tasks/A.4-configuration-types.md) |

**Phase 1 Total**: 8 hours

### Phase 2: Filesystem Provider (Week 1-2)
Refactor existing filesystem storage to use the new trait system.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.1 | **Extract FilesystemBackend** | 3h | A.1 | ⬜ Not Started | | [Details](tasks/B.1-filesystem-backend.md) |
| B.2 | **Implement FilesystemProviderFactory** | 2h | A.2, B.1 | ⬜ Not Started | | [Details](tasks/B.2-filesystem-factory.md) |
| B.3 | **Migrate Existing Storage Code** | 2h | B.1 | ⬜ Not Started | | [Details](tasks/B.3-migrate-storage.md) |

**Phase 2 Total**: 7 hours

### Phase 3: SQLite Provider (Week 2)
Implement SQLite as an alternative storage backend.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | **Design SQLite Schema** | 1h | A.1 | ⬜ Not Started | | [Details](tasks/C.1-sqlite-schema.md) |
| C.2 | **Implement SqliteBackend** | 4h | C.1 | ⬜ Not Started | | [Details](tasks/C.2-sqlite-backend.md) |
| C.3 | **Create SqliteProviderFactory** | 1h | A.2, C.2 | ⬜ Not Started | | [Details](tasks/C.3-sqlite-factory.md) |

**Phase 3 Total**: 6 hours

### Phase 4: Integration (Week 2)
Update TapeRecorder and public API to use the new system.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.1 | **Update TapeRecorder** | 2h | A.3, B.2 | ⬜ Not Started | | [Details](tasks/D.1-update-recorder.md) |
| D.2 | **Add Registry to Public API** | 2h | A.3, D.1 | ⬜ Not Started | | [Details](tasks/D.2-public-api.md) |
| D.3 | **Update Configuration Loading** | 2h | D.2 | ⬜ Not Started | | [Details](tasks/D.3-config-loading.md) |

**Phase 4 Total**: 6 hours

### Phase 5: Testing & Documentation (Week 3)
Comprehensive testing and documentation of the new system.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| E.1 | **Unit Tests for Registry** | 1h | A.3 | ⬜ Not Started | | [Details](tasks/E.1-registry-tests.md) |
| E.2 | **Integration Tests** | 2h | D.3 | ⬜ Not Started | | [Details](tasks/E.2-integration-tests.md) |
| E.3 | **Documentation & Examples** | 1h | D.3 | ⬜ Not Started | | [Details](tasks/E.3-documentation.md) |

**Phase 5 Total**: 4 hours

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

### Registering a Custom Provider
```rust
// Define custom backend
struct S3Backend { /* ... */ }
impl TapeStorageBackend for S3Backend { /* ... */ }

// Define factory
struct S3ProviderFactory;
impl StorageProviderFactory for S3ProviderFactory {
    fn provider_name(&self) -> &str { "s3" }
    fn create(&self, config: Value) -> Result<Box<dyn TapeStorageBackend>> {
        // Parse config and create S3Backend
    }
}

// Register with Shadowcat
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
- SQLite provider will use SQLx for async database operations
- Registry is global but can be overridden per Shadowcat instance if needed
- Consider adding provider capabilities/features discovery in future

---

**Document Version**: 1.0  
**Created**: 2025-01-13  
**Last Modified**: 2025-01-13  
**Author**: Development Team

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-01-13 | 1.0 | Initial plan creation | Development Team |