# Task B.3: Add Forward/Gateway Commands

## Objective

Implement the new command structure using "forward" and "gateway" terminology. Since Shadowcat hasn't been released yet, we're free to use the clearest terminology from the start, while being helpful to users familiar with traditional proxy terms.

## Background

Based on our design (A.2), we're using "forward" and "gateway" because:
- "Forward proxy" is already well-understood
- "Gateway" is much more intuitive than "reverse proxy"
- API gateways are a familiar concept to most developers
- This creates a clearer mental model of what each command does

Since some developers will instinctively type "reverse" (from reverse proxy), we'll catch that and helpfully redirect them to "gateway"

## Key Questions to Answer

1. How do we handle users who type "reverse" out of habit?
2. Should the warning message be colored/styled for visibility?
3. How do we update all help text to use the new terminology?
4. What's the best way to structure the command enums?
5. How do we ensure all tests are updated?

## Step-by-Step Process

### 1. Analysis Phase (30 min)
Understand current command structure

```bash
cd shadowcat
# Find all references to reverse command
grep -r "reverse" src/
grep -r "Reverse" src/
grep -r "Command::Reverse" src/
```

### 2. Design Phase (30 min)

Plan the command structure:
- Keep `Command::Forward` unchanged
- Rename `Command::Reverse` to `Command::Gateway`
- Add migration handler for "reverse" input
- Update all help text and documentation

### 3. Implementation Phase (1.5 hours)

#### 3.1 Update Command Enum
```rust
// src/cli/commands.rs
#[derive(Parser)]
pub enum Command {
    /// Forward proxy to MCP server
    Forward(ForwardArgs),
    
    /// API gateway for MCP clients
    Gateway(GatewayArgs),  // Was: Reverse
    
    /// Record MCP session
    Record(RecordArgs),
    
    /// Replay recorded session
    Replay(ReplayArgs),
}
```

#### 3.2 Add Helpful Redirect for "reverse"
```rust
// src/main.rs or cli parser
fn parse_command(input: &str) -> Command {
    match input {
        "reverse" => {
            // Show educational note
            eprintln!("{}",
                "Note: 'reverse' has been renamed to 'gateway' for clarity."
                    .yellow()
            );
            eprintln!("  Example: shadowcat gateway {}", 
                args.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ")
            );
            eprintln!("{}", "[Continuing as 'gateway'...]\n".dimmed());
            
            // Log for metrics if we want to track this
            log::info!("User typed 'reverse', redirecting to 'gateway'");
            
            // Parse as gateway
            Command::Gateway(parse_gateway_args(remaining_args))
        }
        "gateway" => Command::Gateway(parse_gateway_args(remaining_args)),
        "forward" => Command::Forward(parse_forward_args(remaining_args)),
        // ... other commands
    }
}
```

#### 3.3 Update Help Text
```rust
/// Shadowcat - MCP Developer Proxy
/// 
/// USAGE:
///     shadowcat [TARGET]              Auto-detect and run
///     shadowcat forward [OPTIONS]     Forward proxy to MCP server
///     shadowcat gateway [OPTIONS]     API gateway for MCP clients
///     shadowcat record -- [COMMAND]   Record MCP session
///     shadowcat replay [TAPE]         Replay recorded session
```

### 4. Testing Phase (30 min)
```bash
# Test the helpful redirect
cargo run -- reverse --port 8080

# Test gateway command directly
cargo run -- gateway --port 8080

# Test that forward still works
cargo run -- forward stdio -- echo hello

# Run all tests
cargo test
```

Test cases to implement:
- [ ] "reverse" shows helpful note and works as gateway
- [ ] "gateway" command works directly
- [ ] Help text shows "gateway" not "reverse"
- [ ] All proxy tests work with gateway naming
- [ ] Command parsing handles both inputs correctly

### 5. Documentation Phase (30 min)
- Update README with new command names
- Update all code comments
- Update integration test names
- Update error messages

## Expected Deliverables

### Modified Files
- `src/main.rs` - Command parsing with migration handler
- `src/cli/commands.rs` - Updated command enum
- `src/proxy/reverse.rs` → `src/proxy/gateway.rs` - Renamed file
- `src/proxy/mod.rs` - Updated exports
- `tests/*.rs` - Updated test files

### Documentation Updates
- README.md with new examples
- Help text throughout the codebase
- Error messages mentioning gateway instead of reverse

### Tests
- Helpful redirect test for "reverse"
- Gateway command functionality
- Help text validation
- Command parsing for both inputs

## Success Criteria Checklist

- [ ] "reverse" shows educational note and redirects to gateway
- [ ] "gateway" is the primary command
- [ ] All help text uses "gateway" terminology
- [ ] All tests passing with gateway naming
- [ ] No references to "reverse proxy" in user-facing text (except redirect)
- [ ] Documentation uses gateway terminology
- [ ] No clippy warnings

## Risk Assessment

| Risk | Impact | Mitigation | 
|------|--------|------------|
| Missed references to "reverse" | MEDIUM | Comprehensive grep search |
| Test failures from rename | HIGH | Update tests incrementally |
| Confusing error messages | LOW | Review all error paths |
| User confusion | MEDIUM | Clear warning message |

## Duration Estimate

**Total: 3 hours**
- Analysis: 30 minutes
- Design: 30 minutes
- Implementation: 1.5 hours
- Testing: 30 minutes
- Documentation: 30 minutes

## Dependencies

- B.1: Refactor CLI Module Structure (need modular structure)
- A.2: Design Proposal (command specifications)

## Integration Points

- **main.rs**: Command parsing
- **proxy module**: Gateway implementation
- **help system**: All help text
- **tests**: All integration tests

## Performance Considerations

- Command check is negligible overhead
- Note output to stderr, not stdout
- No performance impact on actual proxy operation

## Notes

- Use colored output for the note (yellow/amber)
- Keep message concise but educational
- Consider telemetry to track how often users type "reverse"
- "gateway" is the primary command going forward

## Commands Reference

```bash
cd shadowcat

# Find all references
grep -rn "reverse" src/ tests/
grep -rn "Reverse" src/ tests/

# Test helpful redirect
cargo run -- reverse --port 8080
cargo run -- gateway --port 8080

# Run tests
cargo test gateway
cargo test cli

# Validation
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Example Implementation

```rust
use colored::*;

// In command parser
pub fn parse_cli() -> Result<Command> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "reverse" {
        // Show helpful redirect message
        eprintln!("{}", 
            "Note: 'reverse' has been renamed to 'gateway' for clarity."
                .yellow()
        );
        eprintln!("  Example: shadowcat gateway {}",
            args[2..].join(" ")
        );
        eprintln!("{}", "[Continuing as 'gateway'...]\n".dimmed());
        
        // Continue as gateway
        args[1] = "gateway".to_string();
    }
    
    // Normal parsing continues
    Command::parse_from(args)
}
```

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-14
**Last Modified**: 2025-01-14
**Author**: Kevin