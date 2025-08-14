# Task A.3: Configuration System

## Overview
Build a flexible configuration system for storage providers that supports multiple formats and validation.

**Duration**: 2 hours  
**Dependencies**: A.1, A.2  
**Status**: ⬜ Not Started

## Objectives

1. Design configuration schema system
2. Support multiple configuration formats (JSON, YAML, TOML)
3. Implement configuration validation
4. Create configuration builders and helpers
5. Support environment variable overrides

## Configuration Architecture

### Base Configuration Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeStorageConfig {
    /// Selected provider name
    pub provider: String,
    
    /// Provider-specific configuration
    #[serde(flatten)]
    pub settings: ProviderSettings,
    
    /// Common options for all providers
    #[serde(default)]
    pub common: CommonStorageOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProviderSettings {
    Filesystem(FilesystemConfig),
    Sqlite(SqliteConfig),
    Custom(Value), // For user-provided backends
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonStorageOptions {
    /// Enable compression for stored tapes
    #[serde(default)]
    pub compression: bool,
    
    /// Enable encryption at rest
    #[serde(default)]
    pub encryption: bool,
    
    /// Maximum storage size in bytes
    pub max_storage_size: Option<u64>,
    
    /// Retention policy
    pub retention: Option<RetentionPolicy>,
    
    /// Performance tuning
    #[serde(default)]
    pub performance: PerformanceOptions,
}
```

### Configuration Loading

```rust
pub struct ConfigLoader {
    sources: Vec<Box<dyn ConfigSource>>,
    validators: Vec<Box<dyn ConfigValidator>>,
}

impl ConfigLoader {
    /// Load configuration from multiple sources
    pub async fn load(&self) -> Result<TapeStorageConfig> {
        let mut config = TapeStorageConfig::default();
        
        // Load from each source in order (later overrides earlier)
        for source in &self.sources {
            if let Some(partial) = source.load().await? {
                config.merge(partial)?;
            }
        }
        
        // Validate final configuration
        for validator in &self.validators {
            validator.validate(&config)?;
        }
        
        Ok(config)
    }
}

#[async_trait]
pub trait ConfigSource: Send + Sync {
    async fn load(&self) -> Result<Option<PartialConfig>>;
}

// Implementations
pub struct FileConfigSource { path: PathBuf }
pub struct EnvConfigSource { prefix: String }
pub struct CliConfigSource { args: Vec<String> }
```

### Schema Validation

```rust
pub struct SchemaValidator {
    schema: Value, // JSON Schema
}

impl SchemaValidator {
    pub fn validate_provider_config(
        &self,
        provider: &str,
        config: &Value,
    ) -> Result<()> {
        // Use jsonschema crate for validation
        let schema = self.get_schema_for_provider(provider)?;
        jsonschema::validate(&schema, config)?;
        Ok(())
    }
}
```

### Configuration Builder

```rust
pub struct StorageConfigBuilder {
    provider: Option<String>,
    settings: HashMap<String, Value>,
    common: CommonStorageOptions,
}

impl StorageConfigBuilder {
    pub fn new() -> Self { ... }
    
    pub fn provider(mut self, name: impl Into<String>) -> Self { ... }
    
    pub fn setting(mut self, key: impl Into<String>, value: impl Serialize) -> Self { ... }
    
    pub fn compression(mut self, enabled: bool) -> Self { ... }
    
    pub fn encryption(mut self, enabled: bool) -> Self { ... }
    
    pub fn retention(mut self, policy: RetentionPolicy) -> Self { ... }
    
    pub fn build(self) -> Result<TapeStorageConfig> { ... }
}
```

## Implementation Steps

1. **Define configuration types** (30 min)
   - Create main config structures
   - Define provider-specific configs
   - Add common options

2. **Implement config sources** (30 min)
   - File source (JSON/YAML/TOML)
   - Environment variable source
   - CLI argument source

3. **Create validation system** (30 min)
   - JSON Schema validation
   - Custom validators
   - Error reporting

4. **Build configuration loader** (20 min)
   - Source priority system
   - Configuration merging
   - Override mechanism

5. **Add builder pattern** (10 min)
   - Fluent API for config creation
   - Validation in builder
   - Convenience methods

## Configuration Examples

### JSON Configuration
```json
{
  "provider": "filesystem",
  "directory": "/var/shadowcat/tapes",
  "index_type": "json",
  "common": {
    "compression": true,
    "retention": {
      "type": "age",
      "max_age_days": 30
    }
  }
}
```

### YAML Configuration
```yaml
provider: sqlite
database_path: /var/shadowcat/tapes.db
connection_pool:
  max_connections: 10
  min_connections: 2
common:
  encryption: true
  performance:
    write_buffer_size: 1048576
    read_cache_size: 10485760
```

### Environment Variables
```bash
SHADOWCAT_STORAGE_PROVIDER=s3
SHADOWCAT_STORAGE_BUCKET=my-tapes
SHADOWCAT_STORAGE_REGION=us-west-2
SHADOWCAT_STORAGE_COMPRESSION=true
```

## Testing Strategy

- Configuration loading from files
- Environment variable overrides
- Configuration validation
- Invalid configuration handling
- Schema validation
- Builder pattern usage

## Success Criteria

- [ ] Support JSON, YAML, TOML formats
- [ ] Environment variable overrides work
- [ ] Validation catches invalid configs
- [ ] Clear error messages
- [ ] Builder pattern is ergonomic
- [ ] Documentation with examples
- [ ] Tests cover all scenarios

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Type mismatch for field {field}: expected {expected}, got {actual}")]
    TypeMismatch {
        field: String,
        expected: String,
        actual: String,
    },
}
```

## Notes

- Consider configuration hot-reloading
- Think about configuration migration
- May need configuration defaults per provider
- Security: sanitize sensitive values in logs
- Consider configuration inheritance/templates

## References

- config-rs crate for configuration management
- JSON Schema specification
- serde documentation for serialization

---

**Next Task**: A.4 - Registry Implementation