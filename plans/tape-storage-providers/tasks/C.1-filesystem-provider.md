# Task B.1: Filesystem Provider Implementation

## Overview
Refactor the existing filesystem storage to implement the new TapeStorageBackend trait.

**Duration**: 3 hours  
**Dependencies**: A.1, A.2, A.3, A.4  
**Status**: ⬜ Not Started

## Objectives

1. Refactor existing TapeStorage to implement TapeStorageBackend
2. Create FilesystemProviderFactory
3. Add filesystem-specific configuration
4. Optimize file operations for performance
5. Implement atomic operations for consistency

## Current Implementation Analysis

The existing `TapeStorage` in `src/recorder/storage.rs` provides:
- Directory-based storage
- JSON index file
- Tape serialization to JSON files
- Basic CRUD operations

Needs enhancement:
- Implement new trait interface
- Add compression support
- Improve atomic operations
- Add file locking
- Better error handling

## Refactored Implementation

### Filesystem Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    /// Base directory for tape storage
    pub directory: PathBuf,
    
    /// Index file type
    #[serde(default = "default_index_type")]
    pub index_type: IndexType,
    
    /// Enable compression
    #[serde(default)]
    pub compression: CompressionType,
    
    /// File permissions (Unix)
    #[serde(default = "default_permissions")]
    pub file_permissions: u32,
    
    /// Use atomic writes
    #[serde(default = "default_atomic_writes")]
    pub atomic_writes: bool,
    
    /// Directory structure
    #[serde(default)]
    pub structure: DirectoryStructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexType {
    Json,
    Sqlite,  // Embedded SQLite for index
    None,    // No index, scan directory
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
    Lz4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectoryStructure {
    Flat,           // All tapes in one directory
    DateBased,      // YYYY/MM/DD/tape-id.json
    SessionBased,   // session-id/tape-id.json
    Sharded(u32),   // Distribute across N subdirectories
}
```

### Filesystem Provider Implementation

```rust
pub struct FilesystemProvider {
    config: FilesystemConfig,
    base_dir: PathBuf,
    index: Box<dyn IndexBackend>,
    compression: Box<dyn Compressor>,
    lock_manager: FileLockManager,
}

#[async_trait]
impl TapeStorageBackend for FilesystemProvider {
    type Config = FilesystemConfig;
    type Metadata = FilesystemMetadata;
    
    async fn initialize(&mut self, config: Self::Config) -> Result<()> {
        self.config = config;
        
        // Create directory structure
        tokio::fs::create_dir_all(&self.base_dir).await?;
        
        // Initialize index
        self.index.initialize(&self.base_dir).await?;
        
        // Set up compression
        self.compression = create_compressor(self.config.compression)?;
        
        // Verify permissions
        self.verify_permissions().await?;
        
        Ok(())
    }
    
    async fn save_tape(&mut self, tape: &Tape) -> Result<TapeId> {
        let tape_id = tape.id.clone();
        let file_path = self.get_tape_path(&tape_id)?;
        
        // Acquire file lock
        let _lock = self.lock_manager.lock_exclusive(&file_path).await?;
        
        // Serialize tape
        let data = serde_json::to_vec_pretty(tape)?;
        
        // Compress if enabled
        let data = self.compression.compress(&data).await?;
        
        // Write atomically
        if self.config.atomic_writes {
            self.atomic_write(&file_path, &data).await?;
        } else {
            tokio::fs::write(&file_path, &data).await?;
        }
        
        // Update index
        self.index.add_tape(&tape_id, &self.create_metadata(tape)).await?;
        
        Ok(tape_id)
    }
    
    async fn load_tape(&self, tape_id: &TapeId) -> Result<Tape> {
        let file_path = self.get_tape_path(tape_id)?;
        
        // Acquire shared lock
        let _lock = self.lock_manager.lock_shared(&file_path).await?;
        
        // Read file
        let data = tokio::fs::read(&file_path).await
            .map_err(|_| StorageError::TapeNotFound(tape_id.clone()))?;
        
        // Decompress if needed
        let data = self.compression.decompress(&data).await?;
        
        // Deserialize
        let tape = serde_json::from_slice(&data)?;
        
        Ok(tape)
    }
    
    async fn delete_tape(&mut self, tape_id: &TapeId) -> Result<()> {
        let file_path = self.get_tape_path(tape_id)?;
        
        // Acquire exclusive lock
        let _lock = self.lock_manager.lock_exclusive(&file_path).await?;
        
        // Remove from index first
        self.index.remove_tape(tape_id).await?;
        
        // Delete file
        tokio::fs::remove_file(&file_path).await?;
        
        // Clean up empty directories if using structured layout
        self.cleanup_empty_dirs(&file_path).await?;
        
        Ok(())
    }
    
    async fn list_tapes(&self) -> Result<Vec<TapeMetadata>> {
        self.index.list_all().await
    }
    
    fn storage_type(&self) -> &str {
        "filesystem"
    }
}
```

### Atomic Operations

```rust
impl FilesystemProvider {
    async fn atomic_write(&self, path: &Path, data: &[u8]) -> Result<()> {
        let temp_path = path.with_extension(".tmp");
        
        // Write to temp file
        tokio::fs::write(&temp_path, data).await?;
        
        // Sync to disk
        let file = tokio::fs::File::open(&temp_path).await?;
        file.sync_all().await?;
        drop(file);
        
        // Atomic rename
        tokio::fs::rename(&temp_path, path).await?;
        
        // Sync directory
        if let Some(parent) = path.parent() {
            let dir = tokio::fs::File::open(parent).await?;
            dir.sync_all().await?;
        }
        
        Ok(())
    }
}
```

### Factory Implementation

```rust
pub struct FilesystemProviderFactory;

#[async_trait]
impl StorageProviderFactory for FilesystemProviderFactory {
    fn name(&self) -> &str {
        "filesystem"
    }
    
    fn display_name(&self) -> &str {
        "Filesystem Storage"
    }
    
    fn description(&self) -> &str {
        "Store tapes as files on the local filesystem with optional compression"
    }
    
    async fn create(&self, config: Value) -> Result<Box<dyn TapeStorageBackend>> {
        let fs_config: FilesystemConfig = serde_json::from_value(config)?;
        
        let mut provider = FilesystemProvider::new();
        provider.initialize(fs_config).await?;
        
        Ok(Box::new(provider))
    }
    
    fn validate_config(&self, config: &Value) -> Result<()> {
        let _: FilesystemConfig = serde_json::from_value(config.clone())?;
        Ok(())
    }
    
    fn config_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "directory": {
                    "type": "string",
                    "description": "Base directory for tape storage"
                },
                "compression": {
                    "type": "string",
                    "enum": ["none", "gzip", "zstd", "lz4"],
                    "default": "none"
                },
                "atomic_writes": {
                    "type": "boolean",
                    "default": true
                }
            },
            "required": ["directory"]
        }))
    }
}
```

## Performance Optimizations

1. **Directory sharding**: Distribute tapes across subdirectories to avoid filesystem limits
2. **Async I/O**: Use tokio::fs for all file operations
3. **Compression**: Optional compression to reduce disk usage
4. **Caching**: Cache frequently accessed tape metadata
5. **Batch operations**: Support batch reads/writes

## Testing Strategy

### Unit Tests
- Atomic write operations
- Directory structure creation
- Compression/decompression
- File locking
- Error handling

### Integration Tests
- Large tape storage
- Concurrent access
- Power failure simulation (atomic writes)
- Permission errors
- Disk full scenarios

## Migration Path

1. Keep existing TapeStorage temporarily
2. Create adapter to new interface
3. Gradually migrate code to use new provider
4. Remove old implementation

## Success Criteria

- [ ] Implements TapeStorageBackend trait
- [ ] Factory registration works
- [ ] Atomic operations are reliable
- [ ] Compression works correctly
- [ ] File locking prevents corruption
- [ ] Performance meets targets
- [ ] Migration path documented
- [ ] Tests pass

## Notes

- Consider fsync strategies for durability
- Think about backup/restore operations
- May need file watching for external changes
- Consider symbolic link handling
- Platform-specific considerations (Windows vs Unix)

## References

- Current implementation: `src/recorder/storage.rs`
- tokio::fs documentation
- atomicwrites crate for atomic file operations

---

**Next Task**: B.2 - SQLite Provider