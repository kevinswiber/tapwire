# Task B.2: Implement Smart Detection

## Objective

Implement the smart auto-detection logic that allows Shadowcat to intelligently determine the user's intent from minimal input, making the common cases "just work" without requiring explicit flags or subcommands.

## Background

Based on the design proposal (A.2), we need to implement heuristics that can detect:
- Executable files → forward proxy (stdio)
- Port notation → gateway mode
- URLs → forward proxy (HTTP)
- Tape files → replay mode
- Ambiguous cases → helpful error messages

This is the core "magic" that will make Shadowcat intuitive for new users.

## Key Questions to Answer

1. How do we reliably detect executable files across platforms?
2. What's the best way to parse port notation (`:8080`, `localhost:8080`, etc.)?
3. How do we handle ambiguous inputs gracefully?
4. Should we support combined operations (e.g., record while proxying)?
5. How do we make the detection logic testable and maintainable?

## Step-by-Step Process

### 1. Analysis Phase (30 min)
Understand the current argument parsing

```bash
cd shadowcat
# Examine current CLI parsing
grep -n "parse" src/main.rs
grep -n "from_args" src/main.rs
```

### 2. Design Phase (30 min)

Define detection modules:
- Input analyzer module
- Pattern matchers for each type
- Ambiguity resolver
- Error message generator

### 3. Implementation Phase (2.5 hours)

#### 3.1 Create Input Analyzer
```rust
// src/cli/detector.rs
pub enum DetectedMode {
    ForwardStdio { command: String },
    ForwardHttp { url: String },
    Gateway { port: u16 },
    Replay { file: PathBuf },
    Ambiguous { suggestions: Vec<String> },
}

pub fn detect_mode(input: &str) -> DetectedMode {
    // Implementation
}
```

#### 3.2 Implement Pattern Matchers
```rust
// Port detection: :8080, localhost:8080, 0.0.0.0:8080
fn is_port_notation(input: &str) -> Option<u16>

// URL detection: http://, https://, ws://, wss://
fn is_url(input: &str) -> Option<Url>

// File detection: .tape, .json, executable files
fn detect_file_type(input: &str) -> Option<FileType>
```

#### 3.3 Integrate with Main CLI
```rust
// Update main.rs to use detector
match args.target {
    Some(target) => {
        match detect_mode(&target) {
            DetectedMode::ForwardStdio { command } => // ...
            DetectedMode::Gateway { port } => // ...
            // etc
        }
    }
    None => // Show help
}
```

### 4. Testing Phase (45 min)
```bash
# Test detection logic
cargo test detector
cargo test cli_smart

# Manual testing
cargo run -- my-server
cargo run -- :8080
cargo run -- http://example.com
cargo run -- session.tape
```

Test cases to implement:
- [ ] Executable file detection
- [ ] Port notation variations
- [ ] URL detection
- [ ] Tape file detection
- [ ] Ambiguous input handling
- [ ] Platform-specific paths

### 5. Documentation Phase (15 min)
- Add module documentation
- Update CLI help text
- Add examples to README

## Expected Deliverables

### New Files
- `src/cli/detector.rs` - Smart detection logic
- `src/cli/patterns.rs` - Pattern matching utilities
- `tests/cli_detection.rs` - Comprehensive tests

### Modified Files
- `src/main.rs` - Integration with detector
- `src/cli/mod.rs` - Module exports
- `src/lib.rs` - Public API updates

### Tests
- Unit tests for each pattern matcher
- Integration tests for full detection flow
- Edge case tests for ambiguous inputs
- Platform-specific tests

## Success Criteria Checklist

- [ ] Common cases work without flags
- [ ] Ambiguous cases show helpful errors
- [ ] All pattern matchers have tests
- [ ] No performance regression
- [ ] Cross-platform compatibility
- [ ] Documentation complete
- [ ] No clippy warnings

## Risk Assessment

| Risk | Impact | Mitigation | 
|------|--------|------------|
| Platform differences | HIGH | Test on Linux, macOS, Windows |
| Ambiguous inputs | MEDIUM | Clear error messages with suggestions |
| Performance overhead | LOW | Cache detection results |
| Complex regex | MEDIUM | Use simple string operations where possible |

## Duration Estimate

**Total: 4 hours**
- Analysis: 30 minutes
- Design: 30 minutes
- Implementation: 2.5 hours
- Testing: 45 minutes
- Documentation: 15 minutes

## Dependencies

- B.1: Refactor CLI Module Structure (need modular structure)
- A.2: Design Proposal (detection logic specification)

## Integration Points

- **main.rs**: Primary integration point
- **CLI commands**: Must work with existing command structure
- **Error handling**: Consistent with project patterns
- **Help system**: Must update help text

## Performance Considerations

- Detection should be < 1ms for common cases
- Avoid expensive file system operations
- Cache results if detection is called multiple times
- Use lazy evaluation where possible

## Notes

- Start with simple heuristics, enhance later
- Make detection deterministic and predictable
- Provide escape hatch (--mode flag) for edge cases
- Consider how this works with future transport types

## Commands Reference

```bash
cd shadowcat

# Development
cargo build
cargo check

# Testing detection
cargo run -- my-server           # Should detect forward proxy
cargo run -- :8080              # Should detect gateway
cargo run -- http://localhost   # Should detect forward HTTP
cargo run -- recording.tape     # Should detect replay

# Run tests
cargo test detector
cargo test --test cli_integration

# Validation
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Example Implementation

```rust
use std::path::Path;
use url::Url;

pub fn detect_mode(input: &str) -> DetectedMode {
    // Check for port notation first (most specific)
    if let Some(port) = parse_port_notation(input) {
        return DetectedMode::Gateway { port };
    }
    
    // Check for URL
    if let Ok(url) = Url::parse(input) {
        if url.scheme() == "http" || url.scheme() == "https" {
            return DetectedMode::ForwardHttp { url: input.to_string() };
        }
    }
    
    // Check for tape file
    if input.ends_with(".tape") || input.ends_with(".json") {
        if Path::new(input).exists() {
            return DetectedMode::Replay { file: input.into() };
        }
    }
    
    // Check for executable
    if is_executable(input) {
        return DetectedMode::ForwardStdio { command: input.to_string() };
    }
    
    // Ambiguous - provide suggestions
    DetectedMode::Ambiguous {
        suggestions: vec![
            format!("shadowcat forward {input} - for forward proxy"),
            format!("shadowcat gateway --port {input} - for API gateway"),
        ],
    }
}

fn parse_port_notation(input: &str) -> Option<u16> {
    // Handle :8080, localhost:8080, 0.0.0.0:8080
    if input.starts_with(':') {
        input[1..].parse().ok()
    } else if let Some(pos) = input.rfind(':') {
        input[pos + 1..].parse().ok()
    } else {
        None
    }
}
```

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-14
**Last Modified**: 2025-01-14
**Author**: Kevin