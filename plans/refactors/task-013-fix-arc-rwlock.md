# Task 013: Fix Arc RwLock Overuse

## Overview
Review and optimize the excessive use of `Arc<RwLock<T>>` throughout the codebase to reduce lock contention and improve performance.

## Context
From the [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md), performance analysis identified concerns with excessive `Arc<RwLock>` usage that may be causing unnecessary lock contention and performance degradation.

## Scope
- **Files to modify**: Multiple modules using Arc<RwLock>
- **Priority**: MEDIUM - Performance optimization
- **Time estimate**: 1 day

## Current Problems

### Identified Issues
1. **Overuse of RwLock** - Used even for data that's rarely written
2. **Lock contention** - Multiple threads competing for same locks
3. **Nested locking** - Risk of deadlocks
4. **Unnecessary Arc wrapping** - For data that could be immutable

### Common Anti-patterns
```rust
// Anti-pattern 1: RwLock for mostly-read data
struct SessionManager {
    config: Arc<RwLock<Config>>,  // Config rarely changes
}

// Anti-pattern 2: Fine-grained locking
struct ProxyState {
    upstream_url: Arc<RwLock<String>>,
    timeout: Arc<RwLock<Duration>>,
    retry_count: Arc<RwLock<usize>>,
}

// Anti-pattern 3: Nested locks
let outer = state.read().unwrap();
let inner = outer.something.write().unwrap();  // Deadlock risk
```

## Implementation Plan

### Step 1: Audit Current Usage

```bash
# Find all Arc<RwLock> usage
rg "Arc<RwLock" --type rust

# Find nested lock patterns (potential deadlocks)
rg "\.read\(\)|\.write\(\)" --type rust -A 5 -B 5 | grep -E "(read|write)\(\).*\n.*\.(read|write)\(\)"
```

Categories to identify:
1. **Immutable after init** - Can use Arc only
2. **Rarely written** - Can use RwLock without Arc or ArcSwap
3. **Frequently written** - Keep Arc<RwLock> or use different pattern
4. **Single owner** - Don't need Arc at all

### Step 2: Replace Immutable Data with Arc

#### Before - Unnecessary RwLock
```rust
pub struct Config {
    data: Arc<RwLock<ConfigData>>,
}

impl Config {
    pub fn new(data: ConfigData) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
        }
    }
    
    pub fn get_timeout(&self) -> Duration {
        self.data.read().unwrap().timeout  // Lock for read-only data
    }
}
```

#### After - Just Arc for Immutable
```rust
pub struct Config {
    data: Arc<ConfigData>,
}

impl Config {
    pub fn new(data: ConfigData) -> Self {
        Self {
            data: Arc::new(data),
        }
    }
    
    pub fn get_timeout(&self) -> Duration {
        self.data.timeout  // No lock needed
    }
}
```

### Step 3: Use ArcSwap for Rarely Updated Data

```rust
use arc_swap::ArcSwap;

pub struct SessionManager {
    // Config that's updated rarely but read frequently
    config: ArcSwap<SessionConfig>,
    // Sessions that are frequently modified
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionManager {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config: ArcSwap::from_pointee(config),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // Fast read path - no locking
    pub fn get_config(&self) -> Arc<SessionConfig> {
        self.config.load_full()
    }
    
    // Rare update path
    pub fn update_config(&self, config: SessionConfig) {
        self.config.store(Arc::new(config));
    }
}
```

### Step 4: Use DashMap for Concurrent Collections

```rust
use dashmap::DashMap;

// Before - Manual locking
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    pub async fn get(&self, id: &str) -> Option<Session> {
        self.sessions.read().unwrap().get(id).cloned()
    }
    
    pub async fn insert(&self, id: String, session: Session) {
        self.sessions.write().unwrap().insert(id, session);
    }
}

// After - Lock-free concurrent map
pub struct SessionStore {
    sessions: Arc<DashMap<String, Session>>,
}

impl SessionStore {
    pub async fn get(&self, id: &str) -> Option<Session> {
        self.sessions.get(id).map(|r| r.clone())
    }
    
    pub async fn insert(&self, id: String, session: Session) {
        self.sessions.insert(id, session);
    }
}
```

### Step 5: Eliminate Unnecessary Arc

```rust
// Before - Unnecessary Arc
struct RequestHandler {
    validator: Arc<RwLock<Validator>>,  // Only used by one owner
}

// After - Direct ownership
struct RequestHandler {
    validator: Validator,  // If single-threaded
    // OR
    validator: RwLock<Validator>,  // If needs mutation but not sharing
}
```

### Step 6: Use Atomics for Simple Values

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};

// Before - RwLock for simple values
struct Metrics {
    request_count: Arc<RwLock<u64>>,
    is_healthy: Arc<RwLock<bool>>,
}

impl Metrics {
    pub fn increment_requests(&self) {
        let mut count = self.request_count.write().unwrap();
        *count += 1;
    }
}

// After - Atomic operations
struct Metrics {
    request_count: AtomicU64,
    is_healthy: AtomicBool,
}

impl Metrics {
    pub fn increment_requests(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_request_count(&self) -> u64 {
        self.request_count.load(Ordering::Relaxed)
    }
}
```

### Step 7: Reduce Lock Scope

```rust
// Before - Holding lock too long
impl SessionManager {
    pub async fn process_message(&self, msg: Message) -> Result<Response> {
        let mut sessions = self.sessions.write().unwrap();
        let session = sessions.get_mut(&msg.session_id).unwrap();
        
        // Long processing while holding write lock
        let result = expensive_operation(&session).await?;
        session.update(result);
        
        Ok(Response::new())
    }
}

// After - Minimize lock scope
impl SessionManager {
    pub async fn process_message(&self, msg: Message) -> Result<Response> {
        // Clone what we need
        let session_data = {
            let sessions = self.sessions.read().unwrap();
            sessions.get(&msg.session_id).cloned()
        };
        
        // Process without holding lock
        let result = expensive_operation(&session_data).await?;
        
        // Quick update
        {
            let mut sessions = self.sessions.write().unwrap();
            if let Some(session) = sessions.get_mut(&msg.session_id) {
                session.update(result);
            }
        }
        
        Ok(Response::new())
    }
}
```

### Step 8: Consider Alternative Patterns

#### Message Passing Instead of Shared State
```rust
use tokio::sync::mpsc;

// Instead of shared mutable state
enum SessionCommand {
    Get { id: String, response: oneshot::Sender<Option<Session>> },
    Insert { id: String, session: Session },
    Remove { id: String },
}

pub struct SessionActor {
    receiver: mpsc::Receiver<SessionCommand>,
    sessions: HashMap<String, Session>,
}

impl SessionActor {
    pub async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                SessionCommand::Get { id, response } => {
                    let _ = response.send(self.sessions.get(&id).cloned());
                }
                SessionCommand::Insert { id, session } => {
                    self.sessions.insert(id, session);
                }
                SessionCommand::Remove { id } => {
                    self.sessions.remove(&id);
                }
            }
        }
    }
}
```

## Testing Strategy

### Lock Contention Test
```rust
#[tokio::test]
async fn test_no_lock_contention() {
    let store = SessionStore::new();
    let mut handles = vec![];
    
    // Spawn many concurrent operations
    for i in 0..100 {
        let store = store.clone();
        let handle = tokio::spawn(async move {
            for j in 0..1000 {
                store.get(&format!("session-{}-{}", i, j)).await;
            }
        });
        handles.push(handle);
    }
    
    // Should complete quickly without contention
    let start = Instant::now();
    for handle in handles {
        handle.await.unwrap();
    }
    let duration = start.elapsed();
    
    assert!(duration < Duration::from_secs(1));
}
```

### Deadlock Detection
```rust
#[tokio::test]
async fn test_no_deadlocks() {
    // Use parking_lot which has deadlock detection in debug mode
    use parking_lot::RwLock;
    
    // Test various lock acquisition patterns
    // Should not hang
}
```

## Validation

### Pre-check
```bash
# Count Arc<RwLock> usage
rg "Arc<RwLock" --type rust | wc -l

# Run benchmark
cargo bench --bench locks > baseline.txt
```

### Post-check
```bash
# Should be reduced
rg "Arc<RwLock" --type rust | wc -l

# Performance should improve
cargo bench --bench locks > optimized.txt
benchcmp baseline.txt optimized.txt

# Check for deadlocks with thread sanitizer
RUSTFLAGS="-Z sanitizer=thread" cargo test --target x86_64-unknown-linux-gnu
```

## Success Criteria

- [ ] Arc<RwLock usage reduced by >50%
- [ ] No deadlock potential identified
- [ ] Lock contention reduced (measured via benchmarks)
- [ ] Read operations 30% faster for frequently accessed data
- [ ] All tests pass
- [ ] No race conditions introduced

## Performance Targets

| Pattern | Before | After |
|---------|--------|-------|
| Config read | 150ns (with lock) | 10ns (Arc only) |
| Session lookup | 500ns (RwLock) | 200ns (DashMap) |
| Metrics update | 100ns (RwLock) | 5ns (Atomic) |
| Concurrent reads | Linear slowdown | No slowdown |

## Recommended Replacements

| Current | Recommended | Use Case |
|---------|-------------|----------|
| Arc<RwLock<Config>> | Arc<Config> | Immutable after init |
| Arc<RwLock<T>> | ArcSwap<T> | Rarely updated |
| Arc<RwLock<HashMap>> | Arc<DashMap> | Concurrent map |
| Arc<RwLock<Vec>> | Arc<Vec> + ArcSwap | Rarely modified list |
| Arc<RwLock<u64>> | AtomicU64 | Simple counters |
| Arc<RwLock<bool>> | AtomicBool | Flags |

## Risks and Mitigations

1. **Race conditions** - Carefully review each change, add tests
2. **Memory ordering** - Use appropriate Ordering for atomics
3. **API changes** - May need to update public interfaces
4. **Complexity** - Some patterns (actors) add complexity

## Dependencies

Consider adding:
- `arc-swap` - For rarely updated shared data
- `dashmap` - For concurrent hashmaps
- `parking_lot` - Better RwLock implementation with deadlock detection

## Notes

- Not all Arc<RwLock> usage is bad - evaluate case by case
- Consider the trade-off between complexity and performance
- Document why specific synchronization primitives were chosen
- Prefer message passing over shared state where appropriate