# Task A.1: Core Trait Design

## Overview
Design the core `TapeStorageBackend` trait that all storage providers must implement.

**Duration**: 2 hours  
**Dependencies**: None  
**Status**: ⬜ Not Started

## Objectives

1. Define the core storage backend trait with all required methods
2. Establish error handling patterns for storage operations
3. Define associated types for configuration and metadata
4. Create async-safe trait design with Send + Sync bounds
5. Document trait requirements and invariants

## Key Design Decisions

### Trait Methods to Define

```rust
#[async_trait]
pub trait TapeStorageBackend: Send + Sync + 'static {
    type Config: DeserializeOwned + Send + Sync;
    type Metadata: Serialize + DeserializeOwned;
    
    // Lifecycle
    async fn initialize(&mut self, config: Self::Config) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    
    // CRUD Operations
    async fn save_tape(&mut self, tape: &Tape) -> Result<TapeId>;
    async fn load_tape(&self, tape_id: &TapeId) -> Result<Tape>;
    async fn delete_tape(&mut self, tape_id: &TapeId) -> Result<()>;
    async fn list_tapes(&self) -> Result<Vec<TapeMetadata>>;
    
    // Import/Export
    async fn export_tape(&self, tape_id: &TapeId, writer: &mut dyn Write) -> Result<()>;
    async fn import_tape(&mut self, reader: &mut dyn Read) -> Result<TapeId>;
    
    // Metadata
    async fn get_metadata(&self, tape_id: &TapeId) -> Result<Self::Metadata>;
    async fn update_metadata(&mut self, tape_id: &TapeId, metadata: Self::Metadata) -> Result<()>;
    
    // Search/Query (optional)
    async fn find_by_session(&self, session_id: &SessionId) -> Result<Vec<TapeId>>;
    async fn find_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<TapeId>>;
    
    // Storage info
    fn storage_type(&self) -> &str;
    async fn get_storage_stats(&self) -> Result<StorageStats>;
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Storage not initialized")]
    NotInitialized,
    
    #[error("Tape not found: {0}")]
    TapeNotFound(TapeId),
    
    #[error("Storage operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Backend-specific error: {0}")]
    Backend(Box<dyn std::error::Error + Send + Sync>),
}
```

## Implementation Steps

1. **Create trait module** (30 min)
   - Create `src/recorder/storage/backend.rs`
   - Define the main trait with async_trait
   - Add Send + Sync + 'static bounds

2. **Define associated types** (20 min)
   - Config type for backend-specific configuration
   - Metadata type for backend-specific metadata
   - Consider using defaults where appropriate

3. **Design error types** (30 min)
   - Create comprehensive StorageError enum
   - Add conversion traits for common errors
   - Ensure errors are Send + Sync

4. **Add helper types** (20 min)
   - StorageStats struct for storage metrics
   - TapeMetadata for listing operations
   - QueryOptions for search operations

5. **Documentation** (20 min)
   - Document each trait method
   - Provide implementation guidance
   - Add examples in doc comments

## Testing Approach

### Mock Implementation
```rust
pub struct MockStorageBackend {
    tapes: HashMap<TapeId, Tape>,
    initialized: bool,
}

impl TapeStorageBackend for MockStorageBackend {
    // Implement all methods for testing
}
```

### Test Cases
- Trait object creation and usage
- Error propagation
- Async operation safety
- Send + Sync constraints

## Success Criteria

- [ ] Trait compiles with all methods defined
- [ ] Associated types are properly constrained
- [ ] Error types cover all failure modes
- [ ] Trait is object-safe if needed
- [ ] Documentation is complete
- [ ] Mock implementation works
- [ ] No clippy warnings

## Dependencies and Imports

```rust
use async_trait::async_trait;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::io::{Read, Write};
use chrono::{DateTime, Utc};
use crate::recorder::{Tape, TapeId};
use crate::session::SessionId;
```

## Notes

- Consider whether trait should be object-safe
- Think about streaming for large tapes
- Consider pagination for list operations
- May need separate read/write traits
- Consider transaction support for backends that support it

## References

- Current TapeStorage implementation: `src/recorder/storage.rs`
- Tape types: `src/recorder/tape.rs`
- Similar patterns in sqlx, redis-rs

---

**Next Task**: A.2 - Factory Pattern Implementation