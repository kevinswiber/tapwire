# Task 011: Optimize Clone Operations

## Overview
Reduce the 1,338 clone operations identified in the review to improve performance and memory usage.

## Context
The [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md) found excessive cloning throughout the codebase, impacting performance.

## Analysis of Clone Hotspots

### Current Clone Count
```bash
rg '\.clone\(\)' --type rust | wc -l
# Result: 1,338 clones
```

### Top Offenders
1. Session IDs cloned on every operation
2. Messages cloned for each interceptor
3. Configuration cloned per request
4. Strings cloned unnecessarily
5. Large structures cloned in loops

## Optimization Strategies

### Strategy 1: Use Arc for Shared Immutable Data

**Before**:
```rust
pub struct Frame {
    pub session_id: SessionId,
    pub message: TransportMessage,
}

fn process_frame(frame: Frame) {
    let id = frame.session_id.clone();  // Unnecessary clone
    let msg = frame.message.clone();     // Unnecessary clone
}
```

**After**:
```rust
pub struct Frame {
    pub session_id: Arc<SessionId>,
    pub message: Arc<TransportMessage>,
}

fn process_frame(frame: &Frame) {
    let id = Arc::clone(&frame.session_id);  // Cheap Arc clone
    let msg = Arc::clone(&frame.message);     // Cheap Arc clone
}
```

### Strategy 2: Use Cow for Conditional Ownership

**Before**:
```rust
fn format_message(prefix: String, content: String) -> String {
    format!("{}: {}", prefix, content)
}

// Called with:
format_message(config.prefix.clone(), message.clone())
```

**After**:
```rust
use std::borrow::Cow;

fn format_message<'a>(prefix: &'a str, content: Cow<'a, str>) -> String {
    format!("{}: {}", prefix, content)
}

// Called with:
format_message(&config.prefix, Cow::Borrowed(&message))
```

### Strategy 3: Pass References Instead of Values

**Before**:
```rust
fn validate_session(session: Session) -> Result<Session, Error> {
    // Validation logic
    Ok(session)
}

// Usage:
let session = get_session().clone();
let validated = validate_session(session)?;
```

**After**:
```rust
fn validate_session(session: &Session) -> Result<(), Error> {
    // Validation logic
    Ok(())
}

// Usage:
let session = get_session();
validate_session(&session)?;
```

## Implementation Plan

### Phase 1: Identify and Categorize Clones

```rust
// Create a script to analyze clones
use regex::Regex;
use std::collections::HashMap;

fn analyze_clones() -> HashMap<String, Vec<String>> {
    let mut clones_by_type = HashMap::new();
    
    // Categories:
    // 1. String clones
    // 2. Arc clones (already efficient)
    // 3. Small Copy types (inefficient)
    // 4. Large struct clones
    // 5. Collection clones
    
    // ... analysis logic ...
    
    clones_by_type
}
```

### Phase 2: Fix Session ID Clones

**File**: `src/session/store.rs`

**Before**:
```rust
pub async fn add_frame(&self, frame: Frame) -> Result<(), Error> {
    let session_id = frame.session_id.clone();  // Clone #1
    
    let mut frames = self.frames.write().await;
    frames.entry(frame.session_id.clone())      // Clone #2
        .or_insert_with(Vec::new)
        .push(frame);
        
    self.notify_observers(session_id.clone());   // Clone #3
    Ok(())
}
```

**After**:
```rust
pub async fn add_frame(&self, frame: Frame) -> Result<(), Error> {
    let session_id = &frame.session_id;  // Reference
    
    let mut frames = self.frames.write().await;
    frames.entry(frame.session_id)  // Move, not clone
        .or_insert_with(Vec::new)
        .push(frame);
        
    self.notify_observers(session_id);  // Pass reference
    Ok(())
}
```

### Phase 3: Optimize Message Passing

**File**: `src/interceptor/chain.rs`

**Before**:
```rust
pub async fn process(&self, mut message: TransportMessage) -> Result<TransportMessage, Error> {
    for interceptor in &self.interceptors {
        message = interceptor.process(message.clone()).await?;  // Clone for each!
    }
    Ok(message)
}
```

**After**:
```rust
pub async fn process(&self, message: Arc<TransportMessage>) -> Result<Arc<TransportMessage>, Error> {
    let mut current = message;
    
    for interceptor in &self.interceptors {
        current = interceptor.process(current).await?;  // No clone needed
    }
    Ok(current)
}

// Interceptor only clones if it needs to modify:
impl Interceptor {
    async fn process(&self, message: Arc<TransportMessage>) -> Result<Arc<TransportMessage>, Error> {
        if self.needs_modification(&message) {
            let mut modified = (*message).clone();  // Clone only when needed
            self.modify(&mut modified);
            Ok(Arc::new(modified))
        } else {
            Ok(message)  // Pass through without clone
        }
    }
}
```

### Phase 4: Configuration Optimization

**Before**:
```rust
pub struct RequestHandler {
    config: Config,  // Cloned for each handler
}

impl RequestHandler {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub fn handle(&self) -> Result<(), Error> {
        let timeout = self.config.timeout.clone();
        // ...
    }
}
```

**After**:
```rust
pub struct RequestHandler {
    config: Arc<Config>,  // Shared reference
}

impl RequestHandler {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
    
    pub fn handle(&self) -> Result<(), Error> {
        let timeout = self.config.timeout;  // Copy for primitive
        // ...
    }
}
```

### Phase 5: String Optimization

**Before**:
```rust
fn build_path(base: String, segment: String) -> String {
    format!("{}/{}", base, segment)
}

// Usage:
build_path(config.base_path.clone(), request.path.clone())
```

**After**:
```rust
fn build_path(base: &str, segment: &str) -> String {
    format!("{}/{}", base, segment)
}

// Usage:
build_path(&config.base_path, &request.path)
```

## Benchmarking

```rust
#[bench]
fn bench_before_optimization(b: &mut Bencher) {
    let frame = create_test_frame();
    b.iter(|| {
        process_frame_old(frame.clone())
    });
}

#[bench]
fn bench_after_optimization(b: &mut Bencher) {
    let frame = Arc::new(create_test_frame());
    b.iter(|| {
        process_frame_new(Arc::clone(&frame))
    });
}
```

## Validation

### Metrics to Track

```bash
# Before optimization
rg '\.clone\(\)' --type rust | wc -l  # 1,338

# After optimization (target)
rg '\.clone\(\)' --type rust | wc -l  # <600

# Arc clones (cheap) vs deep clones
rg 'Arc::clone' --type rust | wc -l     # Should increase
rg '\.clone\(\)' --type rust | grep -v 'Arc::clone' | wc -l  # Should decrease
```

### Performance Testing

```bash
# Memory usage before
/usr/bin/time -v ./target/release/shadowcat test

# Memory usage after (should be lower)
/usr/bin/time -v ./target/release/shadowcat test

# CPU profiling
cargo flamegraph --bin shadowcat -- test
```

## Common Patterns

### Pattern 1: Clone-on-Write
```rust
use std::sync::Arc;

pub struct SharedData {
    inner: Arc<InnerData>,
}

impl SharedData {
    pub fn modify<F>(&mut self, f: F) 
    where F: FnOnce(&mut InnerData)
    {
        Arc::make_mut(&mut self.inner);  // Clone only if shared
        f(Arc::get_mut(&mut self.inner).unwrap());
    }
}
```

### Pattern 2: Builder Pattern to Avoid Clones
```rust
pub struct RequestBuilder<'a> {
    method: &'a str,
    path: &'a str,
    headers: Vec<(&'a str, &'a str)>,
}

impl<'a> RequestBuilder<'a> {
    pub fn build(self) -> Request {
        Request {
            method: self.method.to_string(),  // Clone only at build
            path: self.path.to_string(),
            headers: self.headers.into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}
```

## Success Criteria

- [ ] Clone count reduced by >50% (target: <600)
- [ ] Memory usage reduced by >10%
- [ ] No performance regression
- [ ] All tests pass
- [ ] Benchmarks show improvement
- [ ] No functional changes

## Cleanup Checklist

- [ ] Replace SessionId clones with Arc
- [ ] Optimize message passing in interceptors
- [ ] Use Arc for shared configuration
- [ ] Replace string clones with references
- [ ] Use Cow for conditional ownership
- [ ] Benchmark critical paths
- [ ] Document remaining necessary clones