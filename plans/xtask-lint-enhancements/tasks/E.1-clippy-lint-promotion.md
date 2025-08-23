# Task E.1: Clippy Lint Promotion

## Objective
Integrate Clippy lints with xtask to promote specific warnings to errors, providing a unified lint experience that leverages existing Rust tooling rather than reimplementing checks.

## Background
GPT-5's key insight: "Don't reinvent Clippy". Many checks we want already exist in Clippy. We should surface and promote these rather than reimplementing them.

## Requirements

### Clippy Lints to Promote

#### Critical (Deny)
```toml
# Panics
clippy::unwrap_used       # Existing check for unwrap()
clippy::expect_used       # Existing check for expect()
clippy::panic            # Direct panic! calls
clippy::todo            # todo! macros
clippy::unimplemented   # unimplemented! macros

# Debug output
clippy::dbg_macro       # dbg! calls
clippy::print_stdout    # println! in libraries
clippy::print_stderr    # eprintln! in libraries
```

#### Important (Warn → Error in CI)
```toml
# Error handling
clippy::map_err_ignore           # Dropping error context
clippy::result_large_err         # Large error types
clippy::large_enum_variant      # Unbalanced enum sizes

# Code quality
clippy::missing_panics_doc     # Functions that can panic
clippy::missing_errors_doc     # Functions returning Result
clippy::needless_borrow        # Unnecessary references
clippy::useless_vec           # vec![] when array would work
```

#### Advisory (Warn)
```toml
# Performance
clippy::inefficient_to_string   # Inefficient string conversion
clippy::large_stack_arrays     # Stack-heavy arrays
clippy::mutex_atomic          # Mutex for atomic types

# Correctness
clippy::await_holding_lock    # Holding locks across await
clippy::await_holding_refcell_ref  # RefCell across await
```

### Integration Approach

1. **Run Clippy programmatically**:
```rust
// In xtask/src/lint.rs
pub fn run_clippy_checks() -> Result<Vec<LintViolation>> {
    let output = Command::new("cargo")
        .args(&["clippy", "--all-targets", "--", 
                "-W", "clippy::unwrap_used",
                "-W", "clippy::expect_used",
                // ... more lints
        ])
        .output()?;
    
    parse_clippy_output(&output.stdout)
}
```

2. **Parse and enhance output**:
```rust
fn parse_clippy_output(output: &[u8]) -> Vec<LintViolation> {
    // Parse JSON output format
    // Enhance with our own context
    // Add to unified violation list
}
```

3. **Unified reporting**:
```rust
pub fn run_all_lints() -> Result<()> {
    let mut violations = Vec::new();
    
    // Our custom lints
    violations.extend(check_error_variants()?);
    violations.extend(check_module_boundaries()?);
    
    // Clippy lints (promoted)
    violations.extend(run_clippy_checks()?);
    
    report_violations(violations)
}
```

## Implementation Steps

### 1. Clippy Configuration
Create `.clippy.toml`:
```toml
# Clippy configuration
msrv = "1.75.0"
warn-on-all-wildcard-imports = true
allow-expect-in-tests = true
allow-unwrap-in-tests = true
allow-dbg-in-tests = true

# Exclude test code patterns
[lints]
test-pattern = ["#[cfg(test)]", "tests/", "benches/"]
```

### 2. Cargo.toml Lint Configuration
```toml
[workspace.lints.rust]
unsafe_code = "forbid"
rust_2018_idioms = "warn"

[workspace.lints.clippy]
# Promoted to error via xtask
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
todo = "warn"
unimplemented = "warn"
dbg_macro = "warn"
print_stdout = "warn"

# Always error
result_large_err = "deny"
large_enum_variant = "deny"
```

### 3. JSON Output Parser
```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct ClippyMessage {
    reason: String,
    message: ClippyDiagnostic,
}

#[derive(Deserialize)]
struct ClippyDiagnostic {
    level: String,
    spans: Vec<ClippySpan>,
    message: String,
    code: Option<ClippyCode>,
}

fn parse_clippy_json(output: &str) -> Vec<LintViolation> {
    output.lines()
        .filter_map(|line| serde_json::from_str::<ClippyMessage>(line).ok())
        .filter(|msg| msg.reason == "compiler-message")
        .map(|msg| to_lint_violation(msg))
        .collect()
}
```

### 4. Promotion Logic
```rust
const PROMOTE_TO_ERROR: &[&str] = &[
    "clippy::unwrap_used",
    "clippy::expect_used",
    "clippy::panic",
    "clippy::todo",
    "clippy::unimplemented",
];

fn should_promote(code: &str) -> bool {
    PROMOTE_TO_ERROR.contains(&code)
}

fn promote_severity(violation: &mut LintViolation) {
    if should_promote(&violation.code) {
        violation.level = "error";
    }
}
```

## Test Cases

### Clippy Detection
```rust
// Should be caught by clippy::unwrap_used
fn bad_unwrap() {
    let x = Some(5).unwrap();  // ❌ Clippy flags this
}

// Should be caught by clippy::print_stdout
fn bad_print() {
    println!("Debug info");  // ❌ In library code
}

// Should be caught by clippy::result_large_err
type BadResult = Result<(), [u8; 1000]>;  // ❌ Large error
```

### Integration Tests
```rust
#[test]
fn test_clippy_integration() {
    let violations = run_clippy_checks().unwrap();
    
    // Verify Clippy violations are captured
    assert!(violations.iter().any(|v| v.code.contains("unwrap_used")));
    
    // Verify promotion works
    let unwrap_violation = violations.iter()
        .find(|v| v.code == "clippy::unwrap_used")
        .unwrap();
    assert_eq!(unwrap_violation.level, "error");
}
```

## Deliverables

1. **Clippy Integration**:
   - JSON output parsing
   - Promotion logic
   - Unified reporting

2. **Configuration**:
   - `.clippy.toml` file
   - Workspace lint settings
   - CI configuration

3. **Documentation**:
   - List of promoted lints
   - Rationale for each
   - Migration guide

## Success Criteria

- [ ] Clippy runs successfully via xtask
- [ ] JSON output parsed correctly
- [ ] Specified lints promoted to error
- [ ] No duplicate checking with custom lints
- [ ] Performance: <3s for Clippy run
- [ ] Clear unified output format

## Estimated Duration
2 hours

## Dependencies
- Clippy installed (rustup component)
- JSON output support (--message-format=json)

## Notes

### Benefits of This Approach
1. **Maintained by Rust team** - Always up-to-date
2. **Battle-tested** - Used by entire ecosystem
3. **IDE integration** - rust-analyzer understands
4. **Auto-fix support** - `cargo clippy --fix`

### Migration Strategy
1. Start with warnings for all
2. Fix critical issues (unwrap, panic)
3. Promote to error incrementally
4. Document exceptions

### Future Enhancements
- Custom Clippy lint plugins (once stable)
- Per-module lint configuration
- Baseline tracking (allowed violations)