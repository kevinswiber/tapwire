# Task 004: Fix Blocking IO in Async Contexts

## Overview
Replace blocking I/O operations in async contexts to prevent blocking the Tokio runtime.

## Context
The [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md) identified blocking operations in async code that can freeze the entire executor.

## Problem Locations

### 1. CLI Tape Recording
**File**: `src/cli/tape.rs:342`
```rust
// BLOCKING - This blocks the async executor
std::io::stdin().read_line(&mut input)
    .map_err(|e| RecorderError::RecordingFailed(format!("Failed to read input: {}", e)))?;
```

### 2. File Operations
**Various locations using `std::fs` instead of `tokio::fs`**

### 3. Process Spawning
**Using `std::process::Command` instead of `tokio::process::Command`**

## Solutions

### Fix 1: Async Stdin Reading

**Before**:
```rust
use std::io::{self, BufRead};

fn read_user_input() -> Result<String, Error> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input)
}
```

**After**:
```rust
use tokio::io::{self, AsyncBufReadExt, BufReader};

async fn read_user_input() -> Result<String, Error> {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut input = String::new();
    reader.read_line(&mut input).await?;
    Ok(input)
}
```

### Fix 2: Async File Operations

**Before**:
```rust
use std::fs;

fn read_config() -> Result<Config, Error> {
    let contents = fs::read_to_string("config.toml")?;
    toml::from_str(&contents)
}
```

**After**:
```rust
use tokio::fs;

async fn read_config() -> Result<Config, Error> {
    let contents = fs::read_to_string("config.toml").await?;
    toml::from_str(&contents)
        .map_err(|e| Error::ParseError(e.to_string()))
}
```

### Fix 3: Async Process Spawning

**Before**:
```rust
use std::process::Command;

fn run_command(cmd: &str) -> Result<String, Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;
    
    String::from_utf8(output.stdout)
        .map_err(|e| Error::Utf8Error(e.to_string()))
}
```

**After**:
```rust
use tokio::process::Command;

async fn run_command(cmd: &str) -> Result<String, Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await?;
    
    String::from_utf8(output.stdout)
        .map_err(|e| Error::Utf8Error(e.to_string()))
}
```

### Fix 4: Using spawn_blocking for Unavoidable Blocking Operations

Some operations don't have async equivalents. Use `tokio::task::spawn_blocking`:

```rust
// For CPU-intensive operations or libraries without async support
async fn compute_hash(data: Vec<u8>) -> Result<String, Error> {
    tokio::task::spawn_blocking(move || {
        // This runs in a separate thread pool
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:x}", hasher.finalize()))
    })
    .await
    .map_err(|e| Error::TaskError(e.to_string()))?
}
```

## Step-by-Step Implementation

### Step 1: Find All Blocking Operations

```bash
# Find stdin usage
rg "std::io::stdin" --type rust

# Find synchronous file operations
rg "std::fs::" --type rust

# Find synchronous process spawning
rg "std::process::Command" --type rust

# Find potential blocking reads/writes
rg "\.read\(|\. read_to_string\(|\.write\(" --type rust | grep -v await
```

### Step 2: Update Dependencies

Ensure Tokio features are enabled in `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
# Specifically need these features:
# - "io-std" for stdin/stdout
# - "fs" for file operations
# - "process" for command execution
```

### Step 3: Fix Each Module

#### Module: cli/tape.rs

```rust
// Old implementation
pub fn record_interactive() -> Result<(), Error> {
    println!("Enter commands (Ctrl-D to finish):");
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        process_command(&line)?;
    }
    Ok(())
}

// New implementation
pub async fn record_interactive() -> Result<(), Error> {
    println!("Enter commands (Ctrl-D to finish):");
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    
    while let Some(line) = lines.next_line().await? {
        process_command(&line).await?;
    }
    Ok(())
}
```

#### Module: config.rs

```rust
// Update all file operations
impl Config {
    pub async fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let contents = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        toml::from_str(&contents)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
    
    pub async fn save_to_file(&self, path: &Path) -> Result<(), ConfigError> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        
        tokio::fs::write(path, contents)
            .await
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }
}
```

### Step 4: Handle Mixed Sync/Async Contexts

Sometimes you need to call async from sync context:

```rust
// When you can't make the parent function async
fn sync_function_that_needs_async() -> Result<(), Error> {
    // Create a runtime for one-off async operations
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        async_operation().await
    })
}

// Better: restructure to be async all the way up
async fn async_function() -> Result<(), Error> {
    async_operation().await
}
```

## Common Patterns

### Pattern 1: Async Reader/Writer Traits

```rust
use tokio::io::{AsyncRead, AsyncWrite, AsyncBufReadExt};

async fn copy_data<R, W>(reader: R, writer: W) -> io::Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    tokio::io::copy(reader, writer).await
}
```

### Pattern 2: Timeout on Blocking Operations

```rust
use tokio::time::{timeout, Duration};

async fn read_with_timeout() -> Result<String, Error> {
    match timeout(Duration::from_secs(5), read_user_input()).await {
        Ok(Ok(input)) => Ok(input),
        Ok(Err(e)) => Err(Error::IoError(e.to_string())),
        Err(_) => Err(Error::Timeout),
    }
}
```

## Testing

```rust
#[tokio::test]
async fn test_async_file_operations() {
    let temp_file = "test_async.txt";
    
    // Write asynchronously
    tokio::fs::write(temp_file, b"test data").await.unwrap();
    
    // Read asynchronously
    let contents = tokio::fs::read_to_string(temp_file).await.unwrap();
    assert_eq!(contents, "test data");
    
    // Clean up
    tokio::fs::remove_file(temp_file).await.unwrap();
}

#[tokio::test]
async fn test_no_blocking_in_async() {
    // This test ensures we don't block the runtime
    let handle = tokio::spawn(async {
        // Our async operation
        read_user_input().await
    });
    
    // Should be able to do other work concurrently
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Cancel if it takes too long (indicates blocking)
    handle.abort();
}
```

## Validation

```bash
# Check for remaining blocking operations
rg "std::io::stdin\(\)" --type rust
rg "std::fs::read" --type rust | grep -v tokio
rg "std::process::Command" --type rust

# Run tests with async runtime metrics
RUST_LOG=tokio=trace cargo test 2>&1 | grep "block_on"

# Profile for blocking detection
cargo install tokio-console
tokio-console  # Run while app is running to detect blocking
```

## Success Criteria

- [ ] No `std::io::stdin()` in async functions
- [ ] All file operations use `tokio::fs`
- [ ] Process spawning uses `tokio::process`
- [ ] No blocking detected by tokio-console
- [ ] Tests pass with async runtime
- [ ] Response latency remains consistent under load