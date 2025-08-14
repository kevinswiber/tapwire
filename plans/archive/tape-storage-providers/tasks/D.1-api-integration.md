# Task C.1: API Integration

## Overview
Integrate the storage provider system into Shadowcat's public API, allowing users to configure and use custom storage backends.

**Duration**: 3 hours  
**Dependencies**: A.1-A.4, B.1-B.3  
**Status**: ⬜ Not Started

## Objectives

1. Expose storage configuration in public API
2. Add provider registration methods
3. Update CLI to support storage configuration
4. Integrate with existing TapeRecorder
5. Add runtime provider switching capability

## Public API Design

### Core API Extensions

```rust
// In src/lib.rs or src/api/mod.rs

use crate::recorder::storage::{StorageProviderRegistry, StorageProviderFactory, TapeStorageBackend};

/// Main Shadowcat API
pub struct Shadowcat {
    // ... existing fields ...
    storage_registry: Arc<StorageProviderRegistry>,
    active_storage: Arc<RwLock<Option<Arc<dyn TapeStorageBackend>>>>,
}

impl Shadowcat {
    /// Register a custom storage provider
    pub async fn register_storage_provider<F>(&self, factory: F) -> Result<()>
    where
        F: StorageProviderFactory + 'static,
    {
        self.storage_registry.register(factory).await
    }
    
    /// Configure storage backend
    pub async fn configure_storage(
        &self,
        provider_name: &str,
        config: Value,
    ) -> Result<()> {
        // Create storage instance
        let storage = self.storage_registry
            .create_storage(provider_name, config, Some("main".to_string()))
            .await?;
        
        // Update active storage
        let mut active = self.active_storage.write().await;
        *active = Some(storage.clone());
        
        // Update tape recorder if it exists
        if let Some(recorder) = &self.tape_recorder {
            recorder.set_storage(storage).await?;
        }
        
        Ok(())
    }
    
    /// Get current storage backend
    pub async fn get_storage(&self) -> Result<Arc<dyn TapeStorageBackend>> {
        let active = self.active_storage.read().await;
        active.clone()
            .ok_or_else(|| ShadowcatError::StorageNotConfigured)
    }
    
    /// List available storage providers
    pub async fn list_storage_providers(&self) -> Vec<ProviderInfo> {
        self.storage_registry.list_providers().await
    }
}
```

### Builder Pattern Integration

```rust
pub struct ShadowcatBuilder {
    // ... existing fields ...
    storage_config: Option<StorageConfig>,
    custom_providers: Vec<Box<dyn StorageProviderFactory>>,
}

impl ShadowcatBuilder {
    /// Configure storage backend
    pub fn storage(mut self, provider: &str, config: Value) -> Self {
        self.storage_config = Some(StorageConfig {
            provider: provider.to_string(),
            config,
        });
        self
    }
    
    /// Add custom storage provider
    pub fn add_storage_provider<F>(mut self, factory: F) -> Self
    where
        F: StorageProviderFactory + 'static,
    {
        self.custom_providers.push(Box::new(factory));
        self
    }
    
    pub async fn build(self) -> Result<Shadowcat> {
        let mut shadowcat = Shadowcat::new();
        
        // Register custom providers
        for provider in self.custom_providers {
            shadowcat.register_storage_provider(provider).await?;
        }
        
        // Configure storage if specified
        if let Some(config) = self.storage_config {
            shadowcat.configure_storage(&config.provider, config.config).await?;
        } else {
            // Use default filesystem storage
            shadowcat.configure_storage(
                "filesystem",
                json!({"directory": "./recordings"}),
            ).await?;
        }
        
        Ok(shadowcat)
    }
}
```

### Configuration File Support

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ShadowcatConfig {
    // ... existing fields ...
    
    /// Storage configuration
    #[serde(default)]
    pub storage: StorageSection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageSection {
    /// Selected provider
    #[serde(default = "default_provider")]
    pub provider: String,
    
    /// Provider-specific settings
    #[serde(flatten)]
    pub settings: Value,
    
    /// Common options
    #[serde(default)]
    pub options: StorageOptions,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StorageOptions {
    pub compression: bool,
    pub encryption: bool,
    pub retention_days: Option<u32>,
}

fn default_provider() -> String {
    "filesystem".to_string()
}
```

## CLI Integration

### Updated CLI Commands

```rust
// In src/cli/mod.rs

#[derive(Parser)]
pub struct Cli {
    // ... existing commands ...
    
    /// Storage configuration
    #[clap(long, value_name = "PROVIDER")]
    storage_provider: Option<String>,
    
    /// Storage configuration file
    #[clap(long, value_name = "FILE")]
    storage_config: Option<PathBuf>,
    
    /// Storage-specific options (key=value)
    #[clap(long = "storage-opt", value_name = "KEY=VALUE")]
    storage_options: Vec<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...
    
    /// Manage storage providers
    Storage(StorageCommands),
}

#[derive(Parser)]
pub struct StorageCommands {
    #[clap(subcommand)]
    command: StorageSubcommand,
}

#[derive(Subcommand)]
pub enum StorageSubcommand {
    /// List available storage providers
    List,
    
    /// Show storage provider information
    Info {
        #[clap(value_name = "PROVIDER")]
        provider: String,
    },
    
    /// Test storage configuration
    Test {
        #[clap(value_name = "PROVIDER")]
        provider: String,
        
        #[clap(long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    
    /// Show storage statistics
    Stats,
}
```

### CLI Implementation

```rust
pub async fn handle_storage_command(
    shadowcat: &Shadowcat,
    cmd: StorageCommands,
) -> Result<()> {
    match cmd.command {
        StorageSubcommand::List => {
            let providers = shadowcat.list_storage_providers().await;
            
            println!("Available storage providers:");
            for provider in providers {
                println!("  {} - {}", provider.name, provider.display_name);
                println!("    Version: {}", provider.version);
                println!("    Capabilities: {:?}", provider.capabilities);
            }
        }
        
        StorageSubcommand::Info { provider } => {
            let info = shadowcat.get_provider_info(&provider).await?;
            println!("Provider: {}", info.name);
            println!("Display Name: {}", info.display_name);
            println!("Description: {}", info.description);
            
            if let Some(schema) = info.config_schema {
                println!("\nConfiguration Schema:");
                println!("{}", serde_json::to_string_pretty(&schema)?);
            }
        }
        
        StorageSubcommand::Test { provider, config } => {
            let config = if let Some(path) = config {
                let content = tokio::fs::read_to_string(path).await?;
                serde_json::from_str(&content)?
            } else {
                json!({})
            };
            
            println!("Testing storage provider '{}' ...", provider);
            
            // Create test instance
            let storage = shadowcat.storage_registry
                .create_storage(&provider, config, None)
                .await?;
            
            // Run basic tests
            let test_tape = create_test_tape();
            let id = storage.save_tape(&test_tape).await?;
            println!("✓ Save tape: {}", id);
            
            let loaded = storage.load_tape(&id).await?;
            println!("✓ Load tape: {} frames", loaded.frames.len());
            
            let list = storage.list_tapes().await?;
            println!("✓ List tapes: {} found", list.len());
            
            storage.delete_tape(&id).await?;
            println!("✓ Delete tape");
            
            println!("\nStorage provider test successful!");
        }
        
        StorageSubcommand::Stats => {
            let storage = shadowcat.get_storage().await?;
            let stats = storage.get_storage_stats().await?;
            
            println!("Storage Statistics:");
            println!("  Provider: {}", storage.storage_type());
            println!("  Total tapes: {}", stats.tape_count);
            println!("  Total size: {} bytes", stats.total_bytes);
            println!("  Earliest: {:?}", stats.earliest_tape);
            println!("  Latest: {:?}", stats.latest_tape);
        }
    }
    
    Ok(())
}
```

## TapeRecorder Integration

```rust
// Update src/recorder/tape.rs

impl TapeRecorder {
    /// Set storage backend
    pub async fn set_storage(
        &self,
        storage: Arc<dyn TapeStorageBackend>,
    ) -> Result<()> {
        let mut current = self.storage.write().await;
        
        // Flush any pending frames before switching
        self.flush_all().await?;
        
        // Switch storage backend
        *current = storage;
        
        Ok(())
    }
    
    /// Get current storage backend
    pub async fn get_storage(&self) -> Arc<dyn TapeStorageBackend> {
        self.storage.read().await.clone()
    }
}
```

## Usage Examples

### Programmatic Usage

```rust
use shadowcat::{Shadowcat, ShadowcatBuilder};
use my_storage::S3StorageProviderFactory;

#[tokio::main]
async fn main() -> Result<()> {
    // Create Shadowcat with custom storage
    let shadowcat = ShadowcatBuilder::new()
        .add_storage_provider(S3StorageProviderFactory::new())
        .storage("s3", json!({
            "bucket": "my-tapes",
            "region": "us-west-2",
            "prefix": "shadowcat/"
        }))
        .build()
        .await?;
    
    // Use shadowcat normally
    shadowcat.start_proxy().await?;
    
    Ok(())
}
```

### CLI Usage

```bash
# List available providers
shadowcat storage list

# Get provider info
shadowcat storage info sqlite

# Test configuration
shadowcat storage test sqlite --config sqlite.json

# Run with custom storage
shadowcat forward stdio \
  --storage-provider sqlite \
  --storage-config sqlite.json \
  -- mcp-server

# With inline options
shadowcat forward stdio \
  --storage-provider filesystem \
  --storage-opt directory=/var/shadowcat/tapes \
  --storage-opt compression=gzip \
  -- mcp-server
```

### Configuration File

```yaml
# shadowcat.yaml
storage:
  provider: sqlite
  database_path: /var/shadowcat/tapes.db
  options:
    compression: true
    retention_days: 30
  connection_pool:
    max_connections: 10
```

## Testing Strategy

- API registration and configuration
- CLI command parsing and execution
- Configuration file loading
- Provider switching at runtime
- Integration with TapeRecorder
- Error handling for invalid providers

## Success Criteria

- [ ] API methods exposed and documented
- [ ] CLI commands implemented
- [ ] Configuration file support
- [ ] Runtime provider switching works
- [ ] TapeRecorder integration complete
- [ ] Examples and documentation
- [ ] Tests pass

## Notes

- Consider backward compatibility
- Think about provider discovery plugins
- May need provider capabilities negotiation
- Consider async provider initialization
- Security: validate provider configurations

## References

- Current API structure in src/lib.rs
- CLI implementation in src/cli/
- TapeRecorder in src/recorder/tape.rs

---

**Next Task**: C.2 - Forward Proxy Integration