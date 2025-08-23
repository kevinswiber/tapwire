# Task B.1: Implement no_unwrap Checker

## Objective
Implement a pragmatic unwrap/expect checker that eliminates panic-inducing patterns in production code while allowing legitimate uses with documented rationale.

## Background
- Found 1082+ unwrap() calls in non-test code
- Major production stability risk
- Need escape hatches for legitimate uses (startup, tests)

## Requirements

### Detection Patterns
```rust
// Must detect:
result.unwrap()
option.unwrap()
result.expect("msg")
option.expect("msg")
lock.unwrap()  // Mutex/RwLock
channel.send().unwrap()
unwrap_or_else(|_| panic!())
```

### Allowed Contexts
1. Test modules (`#[cfg(test)]`, `tests/`)
2. Build scripts (`build.rs`)
3. Examples (`examples/`)
4. Benchmarks (`benches/`)
5. Explicitly allowed with rationale

### Escape Hatch Syntax
```rust
// allow:lint(no_unwrap) Config file must exist for app startup
let config = Config::load().expect("config.toml not found");

// allow:lint(no_unwrap) Test fixture setup
let test_data = fs::read("fixtures/data.json").unwrap();
```

## Implementation Steps

### 0. Module Structure
Create proper module organization:
```
xtask/src/
├── lint/
│   ├── mod.rs           # Public API and orchestration
│   ├── violations.rs    # LintViolation type and reporting
│   ├── unwrap.rs        # Unwrap/expect/panic detection
│   ├── debug_output.rs  # Print/dbg detection
│   ├── async_blocking.rs # Blocking ops in async
│   ├── boundaries.rs    # Module boundary checks (existing)
│   ├── clippy.rs        # Clippy integration
│   ├── escape_hatch.rs  # Escape hatch parsing
│   └── test_utils.rs    # Test context detection
└── lint.rs              # Re-export and backward compat

xtask/tests/
├── lint_unwrap.rs       # Tests for unwrap detection
├── lint_escape.rs       # Tests for escape hatches
└── fixtures/            # Test files
    ├── has_unwrap.rs
    ├── test_module.rs
    └── with_escape.rs
```

### 1. Parser Implementation
```rust
// In xtask/src/lint/unwrap.rs

use super::{LintViolation, test_utils::is_test_context, escape_hatch::has_lint_allow};

pub fn check_no_unwrap() -> Result<Vec<LintViolation>> {
    check_no_unwrap_in_dir("src")
}

fn check_no_unwrap_in_dir(base_dir: &str) -> Result<Vec<LintViolation>> {
    let mut violations = Vec::new();
    
    for entry in WalkDir::new(base_dir) {
        let path = entry.path();
        
        // Skip if this is test code
        if is_test_context(path, &content)? {
            continue;
        }
        
        // Parse and check for unwrap patterns
        // Look for escape hatch comments
        // Report violations with context
    }
}
```

### 2. Escape Hatch Parser
```rust
fn has_lint_allow(line_num: usize, content: &str) -> Option<String> {
    // Check previous line for // allow:lint(no_unwrap) Reason
    // Extract and validate reason
    // Return Some(reason) if valid
}
```

### 3. Test Context Detection
```rust
// In xtask/src/lint/test_utils.rs

/// Comprehensive test context detection
pub fn is_test_context(path: &Path, content: &str) -> Result<bool> {
    // Path-based detection
    if is_test_path(path) {
        return Ok(true);
    }
    
    // Content-based detection for inline tests
    if is_test_content(content) {
        return Ok(true);
    }
    
    Ok(false)
}

fn is_test_path(path: &Path) -> bool {
    // Check path components
    path.components().any(|c| {
        let s = c.as_os_str().to_str().unwrap_or("");
        s == "tests" || s == "benches" || s == "examples"
    }) ||
    // Check filename
    path.file_name() == Some(OsStr::new("build.rs")) ||
    path.file_stem().and_then(|s| s.to_str()).map_or(false, |s| {
        s.ends_with("_test") || s.ends_with("_tests") || s == "test" || s == "tests"
    })
}

fn is_test_content(content: &str) -> bool {
    // Parse with syn to check for test attributes
    if let Ok(file) = syn::parse_file(content) {
        has_test_attributes(&file)
    } else {
        // Fallback to string search if parse fails
        content.contains("#[cfg(test)]") || 
        content.contains("#[test]") ||
        content.contains("#[tokio::test]") ||
        content.contains("#[async_std::test]")
    }
}

fn has_test_attributes(file: &syn::File) -> bool {
    use syn::{Item, ItemMod};
    
    for item in &file.items {
        match item {
            Item::Mod(ItemMod { attrs, .. }) => {
                if attrs.iter().any(|attr| {
                    attr.path().is_ident("cfg") && 
                    attr.parse_args::<syn::Ident>().map_or(false, |i| i == "test")
                }) {
                    return true;
                }
            }
            Item::Fn(f) => {
                if f.attrs.iter().any(|attr| {
                    attr.path().is_ident("test") ||
                    attr.path().segments.last().map_or(false, |s| s.ident == "test")
                }) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}
```

### 4. Smart Suggestions
```rust
fn suggest_alternative(context: &str) -> String {
    match detect_pattern(context) {
        "lock.unwrap()" => "Consider using `lock.expect(\"mutex poisoned\")`",
        "parse().unwrap()" => "Use `parse().context(\"failed to parse\")?`",
        "channel.send().unwrap()" => "Handle channel disconnection explicitly",
        _ => "Replace with `?` operator or `.expect()` with message"
    }
}
```

## Test Cases

### Should Flag (Production Code)
```rust
// In src/session/manager.rs
fn process() {
    let value = data.unwrap();  // ❌ Violation - production code
}

// In src/transport/http.rs  
fn parse_header(headers: &HeaderMap) {
    let content_type = headers.get("content-type").unwrap();  // ❌ 
}

// Missing rationale
// allow:lint(no_unwrap)
let x = y.unwrap();  // ❌ Need reason after escape hatch
```

### Should Allow (Test/Build/Escaped)
```rust
// In src/session/mod.rs with #[cfg(test)]
#[cfg(test)]
mod tests {
    #[test]
    fn test_foo() {
        assert_eq!(result.unwrap(), 42);  // ✅ Inside test module
    }
}

// In tests/integration_test.rs
fn test_integration() {
    let server = Server::start().unwrap();  // ✅ In tests/ directory
}

// In src/main.rs with escape hatch
fn main() {
    // allow:lint(no_unwrap) Config must exist for app to start
    let config = Config::load().expect("config.toml required");  // ✅
}

// In build.rs
fn main() {
    let version = env::var("VERSION").unwrap();  // ✅ Build script
}

// In examples/demo.rs
fn main() {
    let data = load_demo_data().unwrap();  // ✅ Example code
}

// In benches/benchmark.rs
fn bench_parse(b: &mut Bencher) {
    let input = generate_input().unwrap();  // ✅ Benchmark code
}
```

### Integration Tests
```rust
// xtask/tests/lint_unwrap.rs

#[test]
fn test_detects_unwrap_in_production() {
    let fixture = r#"
        fn process() {
            let value = data.unwrap();
        }
    "#;
    
    let violations = check_unwrap_in_string(fixture, "src/lib.rs").unwrap();
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("unwrap"));
}

#[test]
fn test_ignores_unwrap_in_test_module() {
    let fixture = r#"
        #[cfg(test)]
        mod tests {
            #[test]
            fn test_something() {
                let value = data.unwrap();
            }
        }
    "#;
    
    let violations = check_unwrap_in_string(fixture, "src/lib.rs").unwrap();
    assert_eq!(violations.len(), 0);  // Should not flag test code
}

#[test]
fn test_ignores_unwrap_in_test_directory() {
    let fixture = r#"
        fn test_helper() {
            let value = data.unwrap();
        }
    "#;
    
    let violations = check_unwrap_in_string(fixture, "tests/helper.rs").unwrap();
    assert_eq!(violations.len(), 0);  // Should not flag tests/ directory
}

#[test]
fn test_respects_escape_hatch() {
    let fixture = r#"
        fn main() {
            // allow:lint(no_unwrap) App requires config to start
            let config = Config::load().expect("config required");
        }
    "#;
    
    let violations = check_unwrap_in_string(fixture, "src/main.rs").unwrap();
    assert_eq!(violations.len(), 0);  // Escape hatch should allow
}

#[test]
fn test_requires_escape_hatch_reason() {
    let fixture = r#"
        fn main() {
            // allow:lint(no_unwrap)
            let config = Config::load().unwrap();
        }
    "#;
    
    let violations = check_unwrap_in_string(fixture, "src/main.rs").unwrap();
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("reason"));
}
```

## Deliverables

1. **Lint Implementation**:
   - `check_no_unwrap()` function
   - Escape hatch parser
   - Context detection

2. **Tests**:
   - Unit tests for detection
   - Integration tests with fixtures
   - Escape hatch validation

3. **Documentation**:
   - Usage guide in lint.rs
   - Examples in custom-lint-rules.md
   - Migration guide for existing code

## Success Criteria

- [ ] Detects all unwrap/expect patterns
- [ ] Zero false positives in test code
- [ ] Escape hatch system working
- [ ] Clear, actionable error messages
- [ ] Performance <1s for full scan
- [ ] Integration with existing lint command

## Estimated Duration
3 hours

## Dependencies
- Task A.2 (escape hatch design)
- syn crate for AST parsing

## Notes

Priority patterns to fix first:
1. Session lifecycle (93 instances)
2. Channel operations (high crash risk)
3. Lock unwrapping (mutex poisoning)
4. Header/string conversions

Consider providing auto-fix for simple cases:
- `unwrap()` → `expect("TODO: Add message")`
- In Result context: `unwrap()` → `?`