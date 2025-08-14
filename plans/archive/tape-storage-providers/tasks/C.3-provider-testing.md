# Task B.3: Provider Testing Framework

## Overview
Create comprehensive testing framework for storage providers to ensure consistency across implementations.

**Duration**: 2 hours  
**Dependencies**: B.1, B.2  
**Status**: ⬜ Not Started

## Objectives

1. Create provider-agnostic test suite
2. Build conformance tests for trait implementation
3. Add performance benchmarks
4. Create stress tests for reliability
5. Implement migration testing between providers

## Test Framework Architecture

### Provider Test Trait

```rust
/// Trait that all storage providers must pass tests for
pub trait StorageProviderTests {
    type Provider: TapeStorageBackend;
    
    /// Create a test instance of the provider
    async fn create_test_provider() -> Result<Self::Provider>;
    
    /// Clean up after tests
    async fn cleanup(provider: Self::Provider) -> Result<()>;
    
    /// Get test configuration
    fn test_config() -> Value;
}

/// Macro to generate test suite for a provider
#[macro_export]
macro_rules! storage_provider_tests {
    ($provider:ty) => {
        mod storage_provider_conformance {
            use super::*;
            use $crate::testing::*;
            
            #[tokio::test]
            async fn test_initialize() {
                conformance::test_initialize::<$provider>().await;
            }
            
            #[tokio::test]
            async fn test_save_and_load() {
                conformance::test_save_and_load::<$provider>().await;
            }
            
            #[tokio::test]
            async fn test_delete() {
                conformance::test_delete::<$provider>().await;
            }
            
            #[tokio::test]
            async fn test_list() {
                conformance::test_list::<$provider>().await;
            }
            
            #[tokio::test]
            async fn test_concurrent_access() {
                conformance::test_concurrent_access::<$provider>().await;
            }
            
            #[tokio::test]
            async fn test_large_tapes() {
                conformance::test_large_tapes::<$provider>().await;
            }
        }
    };
}
```

### Conformance Test Suite

```rust
pub mod conformance {
    use super::*;
    
    /// Test provider initialization
    pub async fn test_initialize<T: StorageProviderTests>() {
        let provider = T::create_test_provider().await
            .expect("Failed to create provider");
        
        // Verify provider is initialized
        assert_eq!(provider.storage_type(), T::EXPECTED_TYPE);
        
        // Test double initialization (should be idempotent or error)
        let config = T::test_config();
        let result = provider.initialize(config).await;
        assert!(result.is_ok() || matches!(result, Err(StorageError::AlreadyInitialized)));
        
        T::cleanup(provider).await.expect("Cleanup failed");
    }
    
    /// Test save and load operations
    pub async fn test_save_and_load<T: StorageProviderTests>() {
        let mut provider = T::create_test_provider().await.unwrap();
        
        // Create test tape
        let tape = create_test_tape("test-session-1", 10);
        let tape_id = tape.id.clone();
        
        // Save tape
        let saved_id = provider.save_tape(&tape).await
            .expect("Failed to save tape");
        assert_eq!(saved_id, tape_id);
        
        // Load tape
        let loaded = provider.load_tape(&tape_id).await
            .expect("Failed to load tape");
        
        // Verify tape contents
        assert_eq!(loaded.id, tape.id);
        assert_eq!(loaded.session_id, tape.session_id);
        assert_eq!(loaded.frames.len(), tape.frames.len());
        
        // Verify frame contents
        for (original, loaded) in tape.frames.iter().zip(loaded.frames.iter()) {
            assert_eq!(original.timestamp, loaded.timestamp);
            assert_eq!(original.direction, loaded.direction);
            assert_eq!(original.message, loaded.message);
        }
        
        T::cleanup(provider).await.unwrap();
    }
    
    /// Test concurrent access patterns
    pub async fn test_concurrent_access<T: StorageProviderTests>() {
        let provider = Arc::new(RwLock::new(
            T::create_test_provider().await.unwrap()
        ));
        
        let mut handles = Vec::new();
        
        // Spawn multiple writers
        for i in 0..10 {
            let provider_clone = Arc::clone(&provider);
            handles.push(tokio::spawn(async move {
                let tape = create_test_tape(&format!("session-{i}"), 5);
                let mut provider = provider_clone.write().await;
                provider.save_tape(&tape).await
            }));
        }
        
        // Spawn multiple readers
        for i in 0..10 {
            let provider_clone = Arc::clone(&provider);
            handles.push(tokio::spawn(async move {
                let provider = provider_clone.read().await;
                provider.list_tapes().await
            }));
        }
        
        // Wait for all operations
        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }
        
        // Verify all tapes were saved
        let provider = provider.read().await;
        let tapes = provider.list_tapes().await.unwrap();
        assert_eq!(tapes.len(), 10);
        
        T::cleanup(Arc::try_unwrap(provider).unwrap().into_inner()).await.unwrap();
    }
}
```

### Performance Benchmarks

```rust
pub mod benchmarks {
    use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
    
    fn bench_save_tape<T: StorageProviderTests>(c: &mut Criterion) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        let mut group = c.benchmark_group("save_tape");
        
        for size in [10, 100, 1000, 10000] {
            group.bench_with_input(
                BenchmarkId::new(T::NAME, size),
                &size,
                |b, &size| {
                    b.to_async(&runtime).iter(|| async {
                        let mut provider = T::create_test_provider().await.unwrap();
                        let tape = create_test_tape("bench", size);
                        provider.save_tape(&tape).await.unwrap();
                        T::cleanup(provider).await.unwrap();
                    });
                },
            );
        }
        
        group.finish();
    }
    
    fn bench_load_tape<T: StorageProviderTests>(c: &mut Criterion) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        c.bench_function(&format!("load_tape_{}", T::NAME), |b| {
            b.to_async(&runtime).iter(|| async {
                let mut provider = T::create_test_provider().await.unwrap();
                let tape = create_test_tape("bench", 100);
                let tape_id = provider.save_tape(&tape).await.unwrap();
                
                // Benchmark the load operation
                let _ = provider.load_tape(&tape_id).await.unwrap();
                
                T::cleanup(provider).await.unwrap();
            });
        });
    }
}
```

### Stress Testing

```rust
pub mod stress {
    /// Test provider under sustained load
    pub async fn test_sustained_load<T: StorageProviderTests>() {
        let provider = Arc::new(RwLock::new(
            T::create_test_provider().await.unwrap()
        ));
        
        let start = Instant::now();
        let duration = Duration::from_secs(60); // Run for 1 minute
        
        let mut handles = Vec::new();
        
        // Continuous writes
        let writer_provider = Arc::clone(&provider);
        handles.push(tokio::spawn(async move {
            let mut count = 0;
            while start.elapsed() < duration {
                let tape = create_test_tape(&format!("stress-{count}"), 50);
                let mut provider = writer_provider.write().await;
                if provider.save_tape(&tape).await.is_ok() {
                    count += 1;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            count
        }));
        
        // Continuous reads
        let reader_provider = Arc::clone(&provider);
        handles.push(tokio::spawn(async move {
            let mut count = 0;
            while start.elapsed() < duration {
                let provider = reader_provider.read().await;
                if provider.list_tapes().await.is_ok() {
                    count += 1;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            count
        }));
        
        // Wait for completion
        let results = futures::future::join_all(handles).await;
        
        for (i, result) in results.iter().enumerate() {
            let count = result.as_ref().unwrap();
            println!("Task {} completed {} operations", i, count);
        }
        
        // Verify data integrity
        let provider = provider.read().await;
        let tapes = provider.list_tapes().await.unwrap();
        println!("Total tapes after stress test: {}", tapes.len());
        
        // Randomly verify some tapes
        for tape_meta in tapes.iter().take(10) {
            let tape = provider.load_tape(&tape_meta.id).await.unwrap();
            assert_eq!(tape.id, tape_meta.id);
        }
    }
}
```

### Migration Testing

```rust
pub async fn test_migration<From: StorageProviderTests, To: StorageProviderTests>() {
    // Create source provider with data
    let mut from_provider = From::create_test_provider().await.unwrap();
    
    let mut tape_ids = Vec::new();
    for i in 0..100 {
        let tape = create_test_tape(&format!("migrate-{i}"), 20);
        let id = from_provider.save_tape(&tape).await.unwrap();
        tape_ids.push(id);
    }
    
    // Create destination provider
    let mut to_provider = To::create_test_provider().await.unwrap();
    
    // Migrate all tapes
    for tape_id in &tape_ids {
        let tape = from_provider.load_tape(tape_id).await.unwrap();
        let new_id = to_provider.save_tape(&tape).await.unwrap();
        assert_eq!(new_id, *tape_id);
    }
    
    // Verify migration
    let from_list = from_provider.list_tapes().await.unwrap();
    let to_list = to_provider.list_tapes().await.unwrap();
    assert_eq!(from_list.len(), to_list.len());
    
    // Verify content integrity
    for tape_id in tape_ids.iter().take(10) {
        let from_tape = from_provider.load_tape(tape_id).await.unwrap();
        let to_tape = to_provider.load_tape(tape_id).await.unwrap();
        assert_eq!(from_tape, to_tape);
    }
    
    From::cleanup(from_provider).await.unwrap();
    To::cleanup(to_provider).await.unwrap();
}
```

## Test Data Generators

```rust
/// Create a test tape with specified number of frames
pub fn create_test_tape(session_id: &str, frame_count: usize) -> Tape {
    let mut tape = Tape {
        id: TapeId::new(),
        session_id: SessionId::from(session_id),
        transport_type: TransportType::Http,
        started_at: Utc::now(),
        ended_at: None,
        frames: Vec::with_capacity(frame_count),
        total_bytes: 0,
        metadata: HashMap::new(),
    };
    
    for i in 0..frame_count {
        let envelope = MessageEnvelope {
            timestamp: Utc::now(),
            direction: if i % 2 == 0 { Direction::Incoming } else { Direction::Outgoing },
            message: create_test_message(i),
            context: MessageContext::default(),
        };
        
        tape.total_bytes += envelope.size_hint();
        tape.frames.push(envelope);
    }
    
    tape.ended_at = Some(Utc::now());
    tape
}
```

## Running Tests

```bash
# Run conformance tests for all providers
cargo test storage_provider_conformance

# Run benchmarks
cargo bench --bench storage_providers

# Run stress tests
cargo test stress -- --ignored --test-threads=1

# Run specific provider tests
cargo test filesystem_provider::
cargo test sqlite_provider::
```

## Success Criteria

- [ ] Conformance tests pass for all providers
- [ ] Performance benchmarks established
- [ ] Stress tests show stability
- [ ] Migration between providers works
- [ ] Test coverage > 90%
- [ ] Documentation complete
- [ ] CI integration configured

## Notes

- Consider property-based testing with proptest
- Add fuzzing for robustness
- Test edge cases (empty tapes, huge tapes)
- Consider chaos testing for reliability
- Mock external dependencies

## References

- criterion.rs for benchmarking
- proptest for property testing
- tokio::test for async testing

---

**Next Task**: C.1 - API Integration