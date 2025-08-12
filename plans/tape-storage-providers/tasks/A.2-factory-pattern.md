# Task A.2: Factory Pattern Implementation

## Overview
Implement the factory pattern for creating storage backend instances dynamically.

**Duration**: 3 hours  
**Dependencies**: A.1 (Core Trait Design)  
**Status**: ⬜ Not Started

## Objectives

1. Design `StorageProviderFactory` trait for backend creation
2. Implement registration mechanism for providers
3. Create type-erased storage for heterogeneous backends
4. Design configuration parsing system
5. Implement provider discovery and loading

## Key Design Components

### Factory Trait

```rust
#[async_trait]
pub trait StorageProviderFactory: Send + Sync {
    /// Unique name for this provider (e.g., "filesystem", "sqlite", "s3")
    fn name(&self) -> &str;
    
    /// Display name for UI/logging
    fn display_name(&self) -> &str;
    
    /// Description of the provider
    fn description(&self) -> &str;
    
    /// Create a new instance from configuration
    async fn create(&self, config: Value) -> Result<Box<dyn TapeStorageBackend>>;
    
    /// Validate configuration without creating instance
    fn validate_config(&self, config: &Value) -> Result<()>;
    
    /// Get schema for configuration validation
    fn config_schema(&self) -> Option<Value>;
}
```

### Provider Registry

```rust
pub struct StorageProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn StorageProviderFactory>>>,
    default_provider: RwLock<Option<String>>,
}

impl StorageProviderRegistry {
    pub fn new() -> Self { ... }
    
    /// Register a new storage provider
    pub async fn register<F>(&self, factory: F) -> Result<()>
    where
        F: StorageProviderFactory + 'static;
    
    /// Create a storage backend by name
    pub async fn create(&self, name: &str, config: Value) -> Result<Box<dyn TapeStorageBackend>>;
    
    /// List all registered providers
    pub async fn list_providers(&self) -> Vec<ProviderInfo>;
    
    /// Set the default provider
    pub async fn set_default(&self, name: &str) -> Result<()>;
}
```

### Configuration System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Provider name (e.g., "filesystem", "sqlite")
    pub provider: String,
    
    /// Provider-specific configuration
    pub config: Value,
    
    /// Optional provider-specific options
    #[serde(default)]
    pub options: HashMap<String, Value>,
}

impl StorageConfig {
    pub fn from_str(s: &str) -> Result<Self> { ... }
    pub fn from_file(path: &Path) -> Result<Self> { ... }
    pub fn validate(&self, registry: &StorageProviderRegistry) -> Result<()> { ... }
}
```

## Implementation Steps

1. **Create factory trait** (30 min)
   - Define StorageProviderFactory trait
   - Add configuration validation methods
   - Include metadata methods

2. **Implement registry** (45 min)
   - Create thread-safe provider storage
   - Implement registration logic
   - Add provider lookup and creation

3. **Type erasure wrapper** (45 min)
   - Create Box<dyn TapeStorageBackend> wrapper
   - Handle different Config/Metadata types
   - Ensure Send + Sync propagation

4. **Configuration parsing** (30 min)
   - Design flexible config format
   - Support JSON/YAML/TOML
   - Add validation logic

5. **Provider discovery** (30 min)
   - Implement provider listing
   - Add metadata queries
   - Create provider info structures

## Usage Example

```rust
// In application initialization
let registry = StorageProviderRegistry::new();

// Register built-in providers
registry.register(FilesystemProviderFactory::new()).await?;
registry.register(SqliteProviderFactory::new()).await?;

// User registers custom provider
registry.register(CustomS3ProviderFactory::new()).await?;

// Create storage from config
let config = StorageConfig {
    provider: "s3".to_string(),
    config: json!({
        "bucket": "my-tapes",
        "region": "us-west-2",
        "prefix": "shadowcat/"
    }),
    options: HashMap::new(),
};

let storage = registry.create(&config.provider, config.config).await?;
```

## Testing Strategy

### Unit Tests
- Factory registration and retrieval
- Configuration validation
- Provider name conflicts
- Default provider selection

### Integration Tests
- Multiple provider registration
- Dynamic backend creation
- Configuration from files
- Error handling

## Success Criteria

- [ ] Factory trait is well-defined
- [ ] Registry supports multiple providers
- [ ] Configuration system is flexible
- [ ] Type erasure works correctly
- [ ] Thread-safe registration
- [ ] Clear error messages
- [ ] Documentation complete
- [ ] Tests pass

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum FactoryError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    
    #[error("Provider already registered: {0}")]
    ProviderExists(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Provider creation failed: {0}")]
    CreationFailed(String),
}
```

## Notes

- Consider lazy loading of providers
- Think about provider versioning
- May need provider capabilities/features
- Consider hot-reloading of providers
- Security: validate provider sources

## References

- Factory pattern in Rust: https://refactoring.guru/design-patterns/factory-method
- Similar: diesel's connection manager
- tokio's runtime builder pattern

---

**Next Task**: A.3 - Configuration System