# Task D.1: Migration Strategy

## Overview
Clean replacement of existing TapeStorage with the new provider-based system. Since Shadowcat is pre-release, we can make breaking changes for a better API.

**Duration**: 2 hours  
**Dependencies**: A.1-A.4, B.1, C.1  
**Status**: ⬜ Not Started

## Objectives

1. Remove old TapeStorage implementation completely
2. Replace with new provider-based system
3. Provide data migration tool for development/testing data
4. Update all existing code to use new API
5. Document the new storage system

## Clean Replacement Strategy

### Remove Old Implementation

```rust
// DELETE these files:
// - src/recorder/storage.rs (old TapeStorage)
// - Any legacy storage code

// REPLACE with:
// - src/recorder/storage/mod.rs (new provider system)
// - src/recorder/storage/backend.rs (trait definition)
// - src/recorder/storage/providers/ (provider implementations)
```

### New Clean API

```rust
// The new TapeRecorder will directly use the provider system
pub struct TapeRecorder {
    storage: Arc<dyn TapeStorageBackend>,
    active_tapes: Arc<RwLock<HashMap<SessionId, Tape>>>,
    frame_buffer: Arc<RwLock<HashMap<SessionId, Vec<MessageEnvelope>>>>,
}

impl TapeRecorder {
    /// Create with a specific storage provider
    pub fn new(storage: Arc<dyn TapeStorageBackend>) -> Self {
        Self {
            storage,
            active_tapes: Arc::new(RwLock::new(HashMap::new())),
            frame_buffer: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create with default filesystem storage
    pub fn new_with_default(directory: PathBuf) -> Result<Self> {
        let storage = STORAGE_REGISTRY
            .create_storage("filesystem", json!({"directory": directory}), None)
            .await?;
        Ok(Self::new(storage))
    }
}
```

### Development Data Migration Tool

For developers who have existing test data, we'll provide a simple migration tool:

```rust
/// One-time migration tool for development data only
/// This will be removed before v1.0 release
pub struct DevDataMigrator {
    source_dir: PathBuf,
    target_storage: Arc<dyn TapeStorageBackend>,
}

impl DevDataMigrator {
    pub async fn migrate_dev_data(
        source_dir: PathBuf,
        target_provider: &str,
        target_config: Value,
    ) -> Result<()> {
        info!("Migrating development data from {:?}", source_dir);
        info!("NOTE: This is for development data only!");
        
        let target = STORAGE_REGISTRY
            .create_storage(target_provider, target_config, None)
            .await?;
        
        let migrator = Self {
            source_dir,
            target_storage: target,
        };
        
        migrator.run().await
    }
    
    async fn run(&self) -> Result<()> {
        // Read old index.json if it exists
        let index_path = self.source_dir.join("index.json");
        if !index_path.exists() {
            info!("No index.json found, scanning directory for tape files");
        }
        
        // Find all .json files that look like tapes
        let mut entries = tokio::fs::read_dir(&self.source_dir).await?;
        let mut count = 0;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension() == Some("json".as_ref()) 
                && path.file_name() != Some("index.json".as_ref()) {
                
                match self.migrate_single_file(&path).await {
                    Ok(tape_id) => {
                        info!("Migrated tape: {}", tape_id);
                        count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to migrate {:?}: {}", path, e);
                    }
                }
            }
        }
        
        info!("Migration complete: {} tapes migrated", count);
        Ok(())
    }
    
    async fn migrate_single_file(&self, path: &Path) -> Result<TapeId> {
        let content = tokio::fs::read_to_string(path).await?;
        let tape: Tape = serde_json::from_str(&content)?;
        self.target_storage.save_tape(&tape).await
    }
}
```

### Code Update Locations

Areas that need updating to use the new storage API:

```rust
// 1. src/recorder/tape.rs
// OLD:
use crate::recorder::storage::TapeStorage;

// NEW:
use crate::recorder::storage::{TapeStorageBackend, STORAGE_REGISTRY};

// 2. src/proxy/forward.rs and src/proxy/reverse.rs
// OLD:
let recorder = TapeRecorder::new(PathBuf::from("./recordings"));

// NEW:
let storage = STORAGE_REGISTRY
    .create_storage("filesystem", json!({"directory": "./recordings"}), None)
    .await?;
let recorder = TapeRecorder::new(storage);

// 3. src/cli/commands/tape.rs
// Update all tape commands to use the new storage API

// 4. Configuration files
// OLD:
recording_dir: "./recordings"

// NEW:
storage:
  provider: "filesystem"
  directory: "./recordings"
  compression: true
```

### Benefits of Clean Break

Since we're not maintaining backwards compatibility:

1. **Cleaner API**: No legacy adapters or compatibility layers
2. **Better Performance**: Direct use of new provider system
3. **Simpler Testing**: Only test the new implementation
4. **Less Code**: Remove all legacy code paths
5. **Clear Documentation**: Only document the new way

### Removal Checklist

```markdown
- [ ] Delete src/recorder/storage.rs (old implementation)
- [ ] Remove TapeStorage struct completely
- [ ] Update TapeRecorder to use new provider system
- [ ] Update all proxy code to use new API
- [ ] Update CLI commands for new storage configuration
- [ ] Update all tests to use new provider system
- [ ] Remove any legacy configuration fields
- [ ] Update all documentation
```

## Development Migration Command

Simple command for developers to migrate test data:

```rust
#[derive(Parser)]
pub struct DevMigrateCommand {
    /// Source directory with old tape files
    #[clap(value_name = "DIR")]
    source_dir: PathBuf,
    
    /// Target storage provider (default: filesystem)
    #[clap(long, default_value = "filesystem")]
    provider: String,
    
    /// Target directory for filesystem provider
    #[clap(long)]
    target_dir: Option<PathBuf>,
}

pub async fn handle_dev_migrate(cmd: DevMigrateCommand) -> Result<()> {
    println!("⚠️  Development data migration tool");
    println!("This tool will be removed before v1.0 release");
    
    let config = match cmd.provider.as_str() {
        "filesystem" => {
            let dir = cmd.target_dir.unwrap_or_else(|| PathBuf::from("./recordings_new"));
            json!({"directory": dir})
        }
        "sqlite" => {
            let path = cmd.target_dir.unwrap_or_else(|| PathBuf::from("./tapes.db"));
            json!({"database_path": path})
        }
        _ => {
            return Err(anyhow!("Unknown provider: {}", cmd.provider));
        }
    };
    
    DevDataMigrator::migrate_dev_data(cmd.source_dir, &cmd.provider, config).await?;
    
    println!("✅ Migration complete!");
    Ok(())
}
```

## Testing Strategy

- Test the new provider system thoroughly
- Ensure all existing functionality works with new API
- Test provider switching at runtime
- Verify configuration loading
- Test the dev migration tool (if needed)

## Success Criteria

- [ ] Old TapeStorage code removed completely
- [ ] New provider system fully integrated
- [ ] All tests updated and passing
- [ ] CLI uses new storage configuration
- [ ] Documentation updated
- [ ] Dev migration tool works (optional)
- [ ] No references to old storage system remain

## Notes

- This is a one-time breaking change before release
- Focus on making the new API as clean as possible
- The dev migration tool is temporary and optional
- Prioritize the new implementation over migration

## References

- Current TapeStorage in src/recorder/storage.rs
- Migration patterns in sqlx
- Database migration tools

---

**Next Task**: D.2 - Backward Compatibility