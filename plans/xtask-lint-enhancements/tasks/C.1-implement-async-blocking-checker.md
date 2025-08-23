# Task C.1: Implement Async Blocking Checker

## Objective
Detect and prevent blocking operations in async contexts that could stall the executor thread pool, causing performance degradation or deadlocks.

## Background
Blocking operations in async functions prevent the executor from running other tasks, effectively reducing the thread pool to synchronous execution. This is particularly critical in shadowcat's proxy operations.

## Requirements

### Blocking Patterns to Detect

#### File System Operations
```rust
// In async fn - BAD
std::fs::read()
std::fs::write()
std::fs::File::open()
std::fs::remove_file()

// Should use tokio::fs instead
```

#### Network Operations
```rust
// In async fn - BAD
std::net::TcpStream::connect()
std::net::UdpSocket::bind()

// Should use tokio::net instead
```

#### Thread Operations
```rust
// In async fn - BAD
std::thread::sleep()
std::thread::spawn().join()

// Should use tokio::time::sleep() or tokio::spawn()
```

#### Synchronous Locks
```rust
// In async fn - BAD
std::sync::Mutex::lock()
std::sync::RwLock::read()

// Should use tokio::sync versions
```

#### Process Operations
```rust
// In async fn - BAD
std::process::Command::output()
std::process::Command::status()

// Should use tokio::process or spawn_blocking
```

### Allowed Patterns

1. **Inside spawn_blocking**:
```rust
async fn read_file(path: &str) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        std::fs::read_to_string(path)  // ✅ OK in spawn_blocking
    }).await?
}
```

2. **Quick, non-blocking operations**:
```rust
async fn process() {
    let now = std::time::Instant::now();  // ✅ OK - doesn't block
    let random = rand::random::<u32>();   // ✅ OK - fast
}
```

3. **Explicitly allowed**:
```rust
async fn legacy_integration() {
    // allow:lint(async_blocking) Legacy API requires sync call
    let result = blocking_api_call();
}
```

## Implementation Steps

### 1. AST Analysis
```rust
pub fn check_async_blocking() -> Result<Vec<LintViolation>> {
    check_async_blocking_in_dir("src")
}

fn is_async_function(item: &syn::Item) -> bool {
    match item {
        Item::Fn(f) => f.sig.asyncness.is_some(),
        _ => false
    }
}

fn find_blocking_calls(block: &syn::Block) -> Vec<BlockingCall> {
    // Walk AST looking for path patterns
    // Match against blocking API list
    // Check if inside spawn_blocking
}
```

### 2. Blocking API Registry
```rust
const BLOCKING_APIS: &[&str] = &[
    // File system
    "std::fs::",
    "std::io::BufReader",
    "std::io::BufWriter",
    
    // Network
    "std::net::TcpStream",
    "std::net::UdpSocket",
    
    // Thread
    "std::thread::sleep",
    "std::thread::spawn",
    
    // Locks
    "std::sync::Mutex",
    "std::sync::RwLock",
    "parking_lot::Mutex",
    
    // Process
    "std::process::Command",
];

const ASYNC_ALTERNATIVES: &[(&str, &str)] = &[
    ("std::fs::read", "tokio::fs::read"),
    ("std::thread::sleep", "tokio::time::sleep"),
    ("std::sync::Mutex", "tokio::sync::Mutex"),
    // ... more mappings
];
```

### 3. Context Awareness
```rust
fn is_in_spawn_blocking(expr: &syn::Expr) -> bool {
    // Check if expression is inside spawn_blocking closure
    // Walk up AST to find spawn_blocking call
}

fn is_in_block_on(expr: &syn::Expr) -> bool {
    // Check if inside runtime.block_on()
    // This is OK for top-level coordination
}
```

### 4. Smart Suggestions
```rust
fn suggest_async_alternative(api: &str) -> Option<String> {
    ASYNC_ALTERNATIVES.iter()
        .find(|(blocking, _)| api.contains(blocking))
        .map(|(_, async_api)| format!("Use {} instead", async_api))
}
```

## Test Cases

### Should Flag
```rust
async fn bad_file_read() {
    let contents = std::fs::read("file.txt").unwrap();  // ❌
}

async fn bad_sleep() {
    std::thread::sleep(Duration::from_secs(1));  // ❌
}

async fn bad_mutex() {
    let guard = std::sync::Mutex::new(0).lock();  // ❌
}
```

### Should Allow
```rust
async fn good_file_read() {
    let contents = tokio::fs::read("file.txt").await.unwrap();  // ✅
}

async fn good_spawn_blocking() {
    tokio::task::spawn_blocking(|| {
        std::fs::read("file.txt")  // ✅ Inside spawn_blocking
    }).await;
}

fn sync_function() {
    std::fs::read("file.txt");  // ✅ Not async
}

async fn with_escape_hatch() {
    // allow:lint(async_blocking) FFI requires sync call
    unsafe { blocking_ffi_call() };  // ✅
}
```

## Deliverables

1. **Lint Implementation**:
   - AST walker for async functions
   - Blocking API detector
   - Context awareness (spawn_blocking)

2. **Alternative Suggestions**:
   - Mapping of blocking → async APIs
   - Clear migration guidance

3. **Tests**:
   - Comprehensive test suite
   - Edge cases (nested async, macros)

4. **Documentation**:
   - Common patterns guide
   - Migration examples

## Success Criteria

- [ ] Detects all listed blocking patterns
- [ ] No false positives in spawn_blocking
- [ ] Provides async alternative suggestions
- [ ] Handles macro-generated code
- [ ] Performance: <2s for full scan
- [ ] Clear, actionable error messages

## Estimated Duration
3 hours

## Dependencies
- AST parsing infrastructure
- Task A.2 (escape hatch system)

## Notes

### Common Violations in Shadowcat
1. File operations in recorders
2. Process spawning in transport
3. Sync mutexes in session management

### Priority Fixes
1. Transport layer (network critical)
2. Session manager (high concurrency)
3. Interceptor chains (performance path)

### Future Enhancements
- Detect custom blocking patterns
- Suggest spawn_blocking wrapper
- Auto-fix simple cases