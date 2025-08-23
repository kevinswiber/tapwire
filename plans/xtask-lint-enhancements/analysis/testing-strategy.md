# Testing Strategy for XTask Lint Enhancements

## Critical Testing Requirements

### The Problem with False Positives
Our analysis shows **1082+ unwrap() calls**, but we suspect **80-90% are in test code**. False positives would make the lint unusable, so robust test detection is critical.

## Test Context Detection Strategy

### 1. Path-Based Detection (Fast, Reliable)
These paths should ALWAYS be considered test context:
```
tests/           # Integration tests
benches/         # Benchmarks
examples/        # Example code
build.rs         # Build scripts
*_test.rs        # Test files
*_tests.rs       # Test files
```

### 2. Content-Based Detection (For Inline Tests)
Look for these patterns in file content:
```rust
#[cfg(test)]     # Test module marker
#[test]          # Test function
#[tokio::test]   # Async test
#[async_std::test]
#[bench]         # Benchmark function
```

### 3. AST-Based Detection (Most Accurate)
Parse with `syn` to find:
- Modules with `#[cfg(test)]` attribute
- Functions with test attributes
- Nested test modules

## Test Matrix

### Files to Test

| Test Case | File Path | Content | Should Flag? |
|-----------|-----------|---------|--------------|
| Production code | `src/lib.rs` | `fn foo() { x.unwrap() }` | ✅ Yes |
| Test module | `src/lib.rs` | `#[cfg(test)] mod tests { ... unwrap() }` | ❌ No |
| Test file | `tests/integration.rs` | `fn test() { x.unwrap() }` | ❌ No |
| Test suffix | `src/foo_test.rs` | `fn helper() { x.unwrap() }` | ❌ No |
| Build script | `build.rs` | `fn main() { x.unwrap() }` | ❌ No |
| Example | `examples/demo.rs` | `fn main() { x.unwrap() }` | ❌ No |
| Benchmark | `benches/perf.rs` | `fn bench() { x.unwrap() }` | ❌ No |
| Doc test | `src/lib.rs` | `/// ```\n/// x.unwrap()\n/// ```` | ❌ No |

### Edge Cases

| Edge Case | Description | Expected Behavior |
|-----------|-------------|-------------------|
| Nested test module | Test module inside regular module | Should not flag |
| Test helper function | Non-test function in test module | Should not flag |
| Production code in tests/ | Shared utilities in tests/common/ | Should not flag |
| Generated code | Code with `#[automatically_derived]` | Should not flag |
| Macro-generated tests | Tests created by macros | Should not flag |

## Test Implementation

### Unit Tests Structure
```
xtask/tests/
├── lint_unwrap.rs         # Main unwrap detection tests
├── lint_test_context.rs   # Test context detection
├── lint_escape_hatch.rs   # Escape hatch functionality
└── fixtures/
    ├── production/
    │   ├── has_unwrap.rs
    │   └── with_escape.rs
    └── test_code/
        ├── test_module.rs
        ├── test_file.rs
        └── build.rs
```

### Key Test Functions

```rust
#[test]
fn test_ignores_cfg_test_module() {
    // CRITICAL: This is where most false positives would occur
    let code = r#"
        #[cfg(test)]
        mod tests {
            use super::*;
            
            #[test]
            fn test_something() {
                let result = do_something().unwrap();
                assert_eq!(result, 42);
            }
        }
    "#;
    
    assert_eq!(find_unwraps(code, "src/lib.rs").len(), 0);
}

#[test]
fn test_detects_production_unwrap() {
    let code = r#"
        pub fn process_data(input: &str) -> Result<Data> {
            let parsed = parse(input).unwrap();  // This should be flagged
            Ok(parsed)
        }
    "#;
    
    let violations = find_unwraps(code, "src/processor.rs");
    assert_eq!(violations.len(), 1);
}

#[test]
fn test_respects_escape_hatch() {
    let code = r#"
        fn main() {
            // allow:lint(no_unwrap) Config required for startup
            let config = Config::load().expect("config.toml not found");
        }
    "#;
    
    assert_eq!(find_unwraps(code, "src/main.rs").len(), 0);
}
```

## Real-World Test Data

### From Shadowcat Codebase
Test against actual shadowcat files to verify:

1. **session/manager.rs**: Has 93 unwraps total
   - How many are in `#[cfg(test)]` modules?
   - How many are in production code?

2. **Common patterns to handle**:
```rust
// Pattern 1: Test utilities
#[cfg(test)]
pub fn test_helper() -> TestData {
    TestData::default().unwrap()  // Should not flag
}

// Pattern 2: Test assertions
#[test]
fn test_parse() {
    let result = parse("data");
    assert_eq!(result.unwrap(), expected);  // Should not flag
}

// Pattern 3: Production code
pub fn get_header(req: &Request) -> String {
    req.headers().get("content-type").unwrap().to_string()  // SHOULD FLAG!
}
```

## Success Metrics

### Must Have
- [ ] Zero false positives in test code
- [ ] Detect all production unwraps
- [ ] Handle nested test modules
- [ ] Support all test frameworks (std, tokio, async-std)

### Nice to Have
- [ ] Detect test helper functions
- [ ] Handle macro-generated tests
- [ ] Support custom test attributes

## Testing Commands

```bash
# Run all lint tests
cargo test -p xtask lint

# Run with verbose output
cargo test -p xtask lint -- --nocapture

# Test against real shadowcat code
cargo xtask lint --check unwrap --stats

# Generate test report
cargo xtask lint --check unwrap --output test-report.json
```

## Validation Against Real Code

### Phase 1: Count Analysis
```bash
# Count actual unwraps in production vs test
rg "unwrap\(\)" src/ --type rust | wc -l
rg "unwrap\(\)" src/ --type rust -g "!*test*" | wc -l
rg "#\[cfg\(test\)\]" src/ --type rust | wc -l
```

### Phase 2: Manual Sampling
- Pick 10 random files with unwraps
- Manually verify test vs production classification
- Adjust detection logic if needed

### Phase 3: Full Run
- Run on entire shadowcat codebase
- Review all flagged violations
- Should be ~100-200, not 1000+

## Common Pitfalls to Avoid

1. **String matching `#[cfg(test)]`**: Could be in comments
2. **Only checking file names**: Inline test modules are common
3. **Not handling nested modules**: Tests can be deeply nested
4. **Missing async test frameworks**: tokio::test, async_std::test
5. **Not checking parent module**: Test function in test module

## Conclusion

The success of this lint depends entirely on accurately distinguishing test code from production code. With ~90% of unwraps likely in tests, even a 5% false positive rate would double the violations to review. Our multi-layered detection strategy (path, content, AST) should achieve near-zero false positives.