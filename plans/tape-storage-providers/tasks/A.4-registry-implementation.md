# Task A.4: Registry Implementation

## Overview
Implement the global registry for managing storage providers with thread-safe access.

**Duration**: 2 hours  
**Dependencies**: A.1, A.2, A.3  
**Status**: ⬜ Not Started

## Objectives

1. Implement thread-safe provider registry
2. Create global registry singleton
3. Add provider lifecycle management
4. Implement provider discovery mechanisms
5. Create registry query and management APIs

## Registry Architecture

### Core Registry Implementation

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

/// Global storage provider registry
pub static STORAGE_REGISTRY: Lazy<Arc<StorageProviderRegistry>> = 
    Lazy::new(|| Arc::new(StorageProviderRegistry::new()));

pub struct StorageProviderRegistry {
    providers: RwLock<HashMap<String, RegisteredProvider>>,
    default_provider: RwLock<Option<String>>,
    initialization_hooks: RwLock<Vec<InitHook>>,
}

struct RegisteredProvider {
    factory: Arc<dyn StorageProviderFactory>,
    metadata: ProviderMetadata,
    instances: RwLock<HashMap<String, WeakStorageRef>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub capabilities: HashSet<ProviderCapability>,
    pub registered_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProviderCapability {
    Compression,
    Encryption,
    Streaming,
    Transactions,
    RemoteStorage,
    LocalStorage,
    Search,
    Replication,
}
```

### Registry Methods

```rust
impl StorageProviderRegistry {
    /// Register a storage provider factory
    pub async fn register(
        &self,
        factory: impl StorageProviderFactory + 'static,
    ) -> Result<()> {
        let name = factory.name().to_string();
        let metadata = ProviderMetadata::from_factory(&factory);
        
        let mut providers = self.providers.write().await;
        
        if providers.contains_key(&name) {
            return Err(RegistryError::ProviderExists(name));
        }
        
        providers.insert(name.clone(), RegisteredProvider {
            factory: Arc::new(factory),
            metadata,
            instances: RwLock::new(HashMap::new()),
        });
        
        // Run initialization hooks
        self.run_init_hooks(&name).await?;
        
        Ok(())
    }
    
    /// Create a storage instance
    pub async fn create_storage(
        &self,
        provider: &str,
        config: Value,
        instance_id: Option<String>,
    ) -> Result<Arc<dyn TapeStorageBackend>> {
        let providers = self.providers.read().await;
        
        let registered = providers.get(provider)
            .ok_or_else(|| RegistryError::ProviderNotFound(provider.to_string()))?;
        
        // Validate config
        registered.factory.validate_config(&config)?;
        
        // Create instance
        let storage = registered.factory.create(config).await?;
        let storage_arc = Arc::from(storage);
        
        // Track instance if ID provided
        if let Some(id) = instance_id {
            let mut instances = registered.instances.write().await;
            instances.insert(id, Arc::downgrade(&storage_arc));
        }
        
        Ok(storage_arc)
    }
    
    /// Get or create default storage
    pub async fn get_default_storage(&self) -> Result<Arc<dyn TapeStorageBackend>> {
        let default_name = self.default_provider.read().await
            .clone()
            .unwrap_or_else(|| "filesystem".to_string());
        
        let default_config = self.get_default_config(&default_name).await?;
        self.create_storage(&default_name, default_config, Some("default".to_string())).await
    }
    
    /// List all registered providers
    pub async fn list_providers(&self) -> Vec<ProviderInfo> {
        let providers = self.providers.read().await;
        
        providers.values().map(|p| ProviderInfo {
            name: p.metadata.name.clone(),
            display_name: p.metadata.display_name.clone(),
            version: p.metadata.version.clone(),
            capabilities: p.metadata.capabilities.clone(),
            instance_count: p.instances.blocking_read().len(),
        }).collect()
    }
    
    /// Query providers by capability
    pub async fn find_by_capability(
        &self,
        capability: ProviderCapability,
    ) -> Vec<String> {
        let providers = self.providers.read().await;
        
        providers.values()
            .filter(|p| p.metadata.capabilities.contains(&capability))
            .map(|p| p.metadata.name.clone())
            .collect()
    }
}
```

### Provider Lifecycle Management

```rust
impl StorageProviderRegistry {
    /// Unregister a provider
    pub async fn unregister(&self, name: &str) -> Result<()> {
        let mut providers = self.providers.write().await;
        
        if let Some(provider) = providers.get(name) {
            // Check for active instances
            let instances = provider.instances.read().await;
            let active_count = instances.values()
                .filter(|weak| weak.strong_count() > 0)
                .count();
            
            if active_count > 0 {
                return Err(RegistryError::ProviderInUse(name.to_string(), active_count));
            }
        }
        
        providers.remove(name)
            .ok_or_else(|| RegistryError::ProviderNotFound(name.to_string()))?;
        
        Ok(())
    }
    
    /// Add initialization hook
    pub async fn add_init_hook<F>(&self, hook: F)
    where
        F: Fn(&str) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static,
    {
        let mut hooks = self.initialization_hooks.write().await;
        hooks.push(Box::new(hook));
    }
    
    /// Get provider info
    pub async fn get_provider_info(&self, name: &str) -> Result<ProviderInfo> {
        let providers = self.providers.read().await;
        
        providers.get(name)
            .map(|p| ProviderInfo::from(&p.metadata))
            .ok_or_else(|| RegistryError::ProviderNotFound(name.to_string()))
    }
}
```

### Built-in Provider Registration

```rust
pub async fn register_builtin_providers() -> Result<()> {
    let registry = &*STORAGE_REGISTRY;
    
    // Register filesystem provider
    registry.register(FilesystemProviderFactory::new()).await?;
    
    // Register SQLite provider
    registry.register(SqliteProviderFactory::new()).await?;
    
    // Set default
    registry.set_default("filesystem").await?;
    
    Ok(())
}

/// Initialize storage subsystem
pub async fn initialize_storage() -> Result<()> {
    register_builtin_providers().await?;
    
    // Load additional providers from plugins
    if let Ok(plugin_dir) = std::env::var("SHADOWCAT_PLUGIN_DIR") {
        load_plugin_providers(&plugin_dir).await?;
    }
    
    Ok(())
}
```

## Implementation Steps

1. **Core registry structure** (30 min)
   - Create registry with RwLock fields
   - Implement provider storage
   - Add instance tracking

2. **Registration methods** (30 min)
   - Implement register/unregister
   - Add duplicate detection
   - Create metadata extraction

3. **Storage creation** (30 min)
   - Implement create_storage
   - Add configuration validation
   - Handle instance tracking

4. **Query methods** (20 min)
   - List providers
   - Find by capability
   - Get provider info

5. **Lifecycle management** (10 min)
   - Add initialization hooks
   - Handle provider cleanup
   - Manage active instances

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_provider_registration() {
    let registry = StorageProviderRegistry::new();
    let factory = MockProviderFactory::new("test");
    
    assert!(registry.register(factory.clone()).await.is_ok());
    assert!(registry.register(factory).await.is_err()); // Duplicate
}

#[tokio::test]
async fn test_storage_creation() {
    let registry = StorageProviderRegistry::new();
    registry.register(MockProviderFactory::new("test")).await.unwrap();
    
    let storage = registry.create_storage("test", json!({}), None).await.unwrap();
    assert!(storage.storage_type() == "test");
}
```

### Integration Tests
- Multiple provider registration
- Concurrent access
- Instance lifecycle
- Default provider handling

## Success Criteria

- [ ] Thread-safe registry implementation
- [ ] Provider registration/unregistration works
- [ ] Storage creation is functional
- [ ] Instance tracking works
- [ ] Query methods return correct results
- [ ] Initialization hooks execute
- [ ] No race conditions
- [ ] Tests pass

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Provider already registered: {0}")]
    ProviderExists(String),
    
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    
    #[error("Provider {0} has {1} active instances")]
    ProviderInUse(String, usize),
    
    #[error("Storage creation failed: {0}")]
    CreationFailed(String),
    
    #[error("Configuration invalid: {0}")]
    InvalidConfig(String),
}
```

## Notes

- Consider provider versioning for upgrades
- Think about plugin loading mechanism
- May need provider dependencies
- Consider provider health checks
- Security: validate provider sources

## References

- once_cell for lazy statics
- Arc/Weak for instance tracking
- Similar: diesel's connection manager registry

---

**Next Task**: B.1 - Filesystem Provider