# Task A.2: Remove Exit() Calls

## Objective
Replace all direct `std::process::exit()` calls with proper error propagation using `Result` types, enabling the codebase to be used as a library where errors can be caught and handled by the caller.

## Background
The current CLI implementation contains multiple direct `exit()` calls:
- `src/cli/forward.rs`: Uses `exit(1)` when command args are empty
- `src/cli/reverse.rs`: Similar exit patterns
- Various error paths that terminate the process

This prevents:
- Library usage (libraries shouldn't exit the process)
- Proper error recovery
- Testing (can't test code that exits)
- Clean resource cleanup

## Key Questions to Answer
1. Where are all the exit() calls located?
2. What error types should replace them?
3. How do we maintain the same CLI behavior while enabling library usage?

## Step-by-Step Process

### 1. Find All Exit Calls
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor
rg "exit\(" --type rust
rg "std::process::exit" --type rust
rg "process::exit" --type rust
```

### 2. Update Error Types
Ensure appropriate error variants exist:
```rust
// src/error.rs or src/cli/error.rs
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Missing required argument: {0}")]
    MissingArgument(String),
    
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    #[error("Command execution failed")]
    ExecutionFailed(#[from] ShadowcatError),
}
```

### 3. Replace Exit Calls in forward.rs
```rust
// Before:
if command_args.is_empty() {
    eprintln!("Error: No command specified for stdio transport");
    exit(1);
}

// After:
if command_args.is_empty() {
    return Err(CliError::MissingArgument(
        "command for stdio transport".to_string()
    ).into());
}
```

### 4. Update main.rs Error Handling
```rust
// src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    init_logging(cli.log_level, cli.verbose);
    
    // Execute command and handle errors
    match cli.command {
        Commands::Forward(cmd) => cmd.execute().await,
        Commands::Reverse(cmd) => cmd.execute().await,
        Commands::Record(cmd) => cmd.execute().await,
        Commands::Replay(cmd) => cmd.execute().await,
        // ...
    }?;  // The ? operator now handles errors properly
    
    Ok(())
}

// The runtime will exit with code 1 if main returns Err
```

### 5. Fix All Unwrap() Calls
While we're at it, replace dangerous unwrap() calls:
```rust
// Find them:
rg "\.unwrap\(\)" --type rust src/cli/

// Replace with proper error handling:
// Before:
let config = ProxyConfig::from_args(&args).unwrap();

// After:
let config = ProxyConfig::from_args(&args)
    .context("Failed to parse proxy configuration")?;
```

### 6. Add Context to Errors
Use anyhow's context for better error messages:
```rust
use anyhow::Context;

// Add context to errors for better debugging
transport.connect()
    .await
    .context("Failed to connect to MCP server")?;
```

### 7. Test Error Propagation
```rust
// tests/cli_errors.rs
#[test]
fn test_missing_command_returns_error() {
    let result = forward_stdio(vec![]).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("command"));
}
```

## Expected Deliverables

### Modified Files
- `shadowcat/src/cli/forward.rs` - No exit() calls
- `shadowcat/src/cli/reverse.rs` - No exit() calls  
- `shadowcat/src/cli/record.rs` - No exit() calls
- `shadowcat/src/cli/replay.rs` - No exit() calls
- `shadowcat/src/main.rs` - Returns Result from main()
- `shadowcat/src/error.rs` or `shadowcat/src/cli/error.rs` - New error types

### Verification Commands
```bash
# Verify no exit calls remain
rg "exit\(" --type rust src/

# Verify error handling works
cargo run -- forward stdio  # Should show nice error about missing command
echo $?  # Should be 1

# Run with valid command
cargo run -- forward stdio -- echo test
echo $?  # Should be 0
```

## Success Criteria Checklist
- [ ] No direct exit() calls in codebase
- [ ] main() returns Result type
- [ ] All errors propagate correctly
- [ ] CLI still exits with code 1 on error
- [ ] Error messages are helpful and contextual
- [ ] Library functions return Result types
- [ ] Tests can verify error conditions

## Risk Assessment
- **Risk**: Changed error messages might confuse users
  - **Mitigation**: Preserve error message content
  - **Mitigation**: Add context for clarity

- **Risk**: Some errors might not propagate correctly
  - **Mitigation**: Test each command path
  - **Mitigation**: Add integration tests

## Duration Estimate
**2 hours**
- 30 min: Find and analyze all exit() calls
- 1 hour: Replace with proper error handling
- 20 min: Test all command paths
- 10 min: Update documentation

## Dependencies
- A.1: Make CLI Module Private (clean separation first)

## Notes
- This is a mechanical refactor but crucial for library usage
- Take the opportunity to improve error messages
- Consider adding error codes for different failure types
- Make sure to test both success and failure paths

## Commands Reference
```bash
# Find all problematic patterns
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor
rg "exit\(|\.unwrap\(\)|\.expect\(" --type rust src/cli/

# Test error handling
cargo run -- forward stdio 2>&1 | head -20
cargo run -- reverse --bind invalid-address

# Verify library usage
cargo build --lib
cargo test --lib

# Check for clippy warnings about error handling
cargo clippy -- -W clippy::unwrap_used -W clippy::expect_used
```