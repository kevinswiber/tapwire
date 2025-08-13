# Task 012: Optimize String Allocations

## Overview
Eliminate unnecessary string allocations in hot paths to improve performance and reduce memory pressure.

## Context
From the [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md), string allocations in hot paths and intermediate JSON Value creation are causing performance issues. With 1,338 clone operations identified, many involve string allocations that could be avoided.

## Scope
- **Files to modify**: Multiple modules, focus on hot paths
- **Priority**: MEDIUM - Performance optimization
- **Time estimate**: 1 day

## Current Problems

### Identified Issues
1. **String allocations in hot paths** - Every request/response cycle
2. **Intermediate JSON Value creation** - Unnecessary serialization/deserialization
3. **Repeated buffer allocations** - Not reusing buffers
4. **Excessive format!() and to_string()** - In loops and frequent operations

### Hotspot Locations
```rust
// Example from transport layer
let json = serde_json::json!({...});  // Creates intermediate Value
serde_json::to_string(&json)  // Then converts to String

// Example from session tracking
let session_id = frame.session_id.clone();  // Clones string on every frame

// Example from error handling
format!("Error: {}", msg)  // In hot error paths
```

## Implementation Plan

### Step 1: Identify Hot Paths

Use profiling to find the most expensive allocations:
```bash
# Profile with flamegraph
cargo flamegraph --bin shadowcat -- forward stdio -- benchmark-script

# Look for:
# - alloc::string::String::from
# - alloc::vec::Vec::from_elem
# - serde_json::to_string
```

Common hot paths:
- Message routing in proxy
- Session ID lookups
- Header processing
- Error formatting
- JSON serialization

### Step 2: Replace String Clones with References

#### Before - Excessive Cloning
```rust
impl Frame {
    pub fn get_session_id(&self) -> String {
        self.session_id.clone()  // Clone on every call
    }
}

impl SessionManager {
    pub async fn process_frame(&self, frame: Frame) {
        let session_id = frame.get_session_id();  // Clone
        let session = self.get_session(&session_id);  // Another clone inside
        // ...
    }
}
```

#### After - Use References
```rust
impl Frame {
    pub fn session_id(&self) -> &str {
        &self.session_id  // Return reference
    }
}

impl SessionManager {
    pub async fn process_frame(&self, frame: &Frame) {
        let session = self.get_session(frame.session_id());  // No clone
        // ...
    }
    
    pub async fn get_session(&self, id: &str) -> Option<Arc<Session>> {
        self.sessions.read().ok()?.get(id).cloned()  // Only clone Arc
    }
}
```

### Step 3: Optimize JSON Serialization

#### Before - Intermediate Values
```rust
pub fn create_response(method: &str, result: Value) -> String {
    let response = json!({
        "jsonrpc": "2.0",
        "method": method,
        "result": result,
        "id": generate_id()
    });
    serde_json::to_string(&response).unwrap()
}
```

#### After - Direct Serialization
```rust
use serde::Serialize;
use std::io::Write;

#[derive(Serialize)]
struct Response<'a> {
    jsonrpc: &'static str,
    method: &'a str,
    result: &'a RawValue,
    id: u64,
}

pub fn create_response<W: Write>(
    writer: &mut W,
    method: &str,
    result: &RawValue,
) -> Result<(), Error> {
    let response = Response {
        jsonrpc: "2.0",
        method,
        result,
        id: generate_id(),
    };
    serde_json::to_writer(writer, &response)?;
    Ok(())
}
```

### Step 4: Use String Interning for Common Strings

```rust
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::RwLock;

pub struct StringInterner {
    strings: RwLock<HashSet<&'static str>>,
}

impl StringInterner {
    pub fn intern(&self, s: &str) -> &'static str {
        // Check if already interned
        if let Ok(strings) = self.strings.read() {
            if let Some(&interned) = strings.get(s) {
                return interned;
            }
        }
        
        // Intern new string
        let leaked = Box::leak(s.to_string().into_boxed_str());
        self.strings.write().unwrap().insert(leaked);
        leaked
    }
}

static INTERNER: Lazy<StringInterner> = Lazy::new(|| StringInterner {
    strings: RwLock::new(HashSet::new()),
});

// Use for common method names, headers, etc.
pub fn intern_method(method: &str) -> &'static str {
    match method {
        "initialize" | "initialized" | "ping" | "pong" 
        | "shutdown" | "tools/list" | "resources/list" => method,
        _ => INTERNER.intern(method),
    }
}
```

### Step 5: Buffer Reuse

```rust
use bytes::{BytesMut, BufMut};

pub struct MessageBuffer {
    buffer: BytesMut,
}

impl MessageBuffer {
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(4096),
        }
    }
    
    pub fn write_message(&mut self, msg: &impl Serialize) -> Result<&[u8], Error> {
        self.buffer.clear();
        
        // Write directly to buffer
        let writer = (&mut self.buffer).writer();
        serde_json::to_writer(writer, msg)?;
        
        Ok(&self.buffer[..])
    }
    
    pub fn reset(&mut self) {
        self.buffer.clear();
        // Keep capacity for reuse
    }
}

// Thread-local buffer pool
thread_local! {
    static BUFFER: RefCell<MessageBuffer> = RefCell::new(MessageBuffer::new());
}

pub fn serialize_message(msg: &impl Serialize) -> Result<Vec<u8>, Error> {
    BUFFER.with(|buf| {
        let mut buf = buf.borrow_mut();
        let bytes = buf.write_message(msg)?;
        Ok(bytes.to_vec())  // Only allocate for return
    })
}
```

### Step 6: Optimize Error Messages

#### Before - Format on Every Error
```rust
impl From<io::Error> for TransportError {
    fn from(err: io::Error) -> Self {
        TransportError::Io(format!("IO error: {}", err))  // Allocation
    }
}
```

#### After - Lazy Formatting
```rust
impl From<io::Error> for TransportError {
    fn from(err: io::Error) -> Self {
        TransportError::Io(err)  // Store original error
    }
}

impl Display for TransportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::Io(err) => write!(f, "IO error: {}", err),
            // Format only when displayed
        }
    }
}
```

### Step 7: Use Cow for Conditional Ownership

```rust
use std::borrow::Cow;

pub struct Header<'a> {
    name: Cow<'a, str>,
    value: Cow<'a, str>,
}

impl<'a> Header<'a> {
    pub fn new_borrowed(name: &'a str, value: &'a str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            value: Cow::Borrowed(value),
        }
    }
    
    pub fn new_owned(name: String, value: String) -> Self {
        Self {
            name: Cow::Owned(name),
            value: Cow::Owned(value),
        }
    }
    
    // Only allocate when needed
    pub fn to_owned(&self) -> Header<'static> {
        Header {
            name: Cow::Owned(self.name.to_string()),
            value: Cow::Owned(self.value.to_string()),
        }
    }
}
```

## Testing Strategy

### Benchmarks
```rust
#[bench]
fn bench_message_serialization(b: &mut Bencher) {
    let msg = create_test_message();
    b.iter(|| {
        serialize_message(&msg)
    });
}

#[bench]
fn bench_session_lookup(b: &mut Bencher) {
    let manager = create_test_manager();
    let session_id = "test-session";
    b.iter(|| {
        manager.get_session(session_id)
    });
}
```

### Memory Profiling
```bash
# Use heaptrack or valgrind
heaptrack ./target/release/shadowcat forward stdio -- benchmark
heaptrack_gui heaptrack.shadowcat.*.gz

# Look for:
# - Total allocations
# - Peak memory usage
# - Allocation hotspots
```

## Validation

### Pre-check
```bash
# Count string allocations
rg '\.to_string\(\)|\.clone\(\)|format!\(' --type rust | wc -l

# Run baseline benchmark
cargo bench --bench strings > baseline.txt
```

### Post-check
```bash
# Should be significantly reduced
rg '\.to_string\(\)|\.clone\(\)|format!\(' --type rust | wc -l

# Compare benchmarks
cargo bench --bench strings > optimized.txt
benchcmp baseline.txt optimized.txt  # Should show improvement

# Memory usage test
/usr/bin/time -v cargo run --release -- forward stdio -- load-test
# Check "Maximum resident set size"
```

## Success Criteria

- [ ] String allocations in hot paths reduced by >50%
- [ ] Message serialization 20% faster
- [ ] Memory usage reduced by >10%
- [ ] No functional regressions
- [ ] All tests pass
- [ ] Benchmarks show improvement

## Performance Targets

| Metric | Before | Target | 
|--------|--------|--------|
| String allocations/request | ~50 | <10 |
| Message serialization | 5μs | <4μs |
| Session lookup | 500ns | <100ns |
| Memory per session | 10KB | <5KB |

## Common Patterns to Apply

1. **Return &str instead of String** where possible
2. **Use &[u8] for byte data** instead of Vec<u8>
3. **Leverage Cow<'_, str>** for conditional ownership
4. **Intern common strings** (method names, headers)
5. **Reuse buffers** with clear() instead of new allocations
6. **Direct serialization** without intermediate Values
7. **Lazy error formatting** with Display trait

## Risks and Mitigations

1. **Lifetime complexity** - Start with simple cases, add lifetimes gradually
2. **API breakage** - Use deprecation for public APIs
3. **Thread safety** - Be careful with thread_local and static buffers
4. **Memory leaks** - Don't intern unbounded user input

## Dependencies

- Consider adding `bytes` crate for efficient buffer management
- May benefit from `string-cache` or similar interning library
- Profile with `flamegraph` and `heaptrack`

## Notes

- Focus on hot paths first for maximum impact
- Some allocations are acceptable in cold paths
- Balance optimization with code readability
- Document why certain optimizations were applied