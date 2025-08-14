# Task D.2: Add Integration Tests

## Objective

Create comprehensive integration tests to ensure the documentation generation feature works correctly and maintains quality over time.

## Background

Integration tests should verify:
- Flag parsing and handling
- Format generation
- Output correctness
- Error handling
- Performance requirements

## Key Questions to Answer

1. What test coverage do we need?
2. How do we validate output correctness?
3. What edge cases should we test?
4. How do we test performance?

## Step-by-Step Process

### 1. Implementation Phase (20 min)

#### Integration Tests
```rust
// tests/doc_generation.rs
#[test]
fn test_markdown_generation() {
    let output = Command::cargo_bin("shadowcat")
        .arg("--help-doc")
        .output()
        .expect("Failed to execute");
    
    assert!(output.status.success());
    let docs = String::from_utf8(output.stdout).unwrap();
    
    // Verify structure
    assert!(docs.contains("# shadowcat"));
    assert!(docs.contains("## Commands"));
    assert!(docs.contains("forward"));
    assert!(docs.contains("reverse"));
}

#[test]
fn test_json_generation() {
    let output = Command::cargo_bin("shadowcat")
        .arg("--help-doc=json")
        .output()
        .expect("Failed to execute");
    
    let json: serde_json::Value = 
        serde_json::from_slice(&output.stdout).unwrap();
    
    assert!(json["commands"].is_array());
}

#[test]
fn test_performance() {
    let start = Instant::now();
    // Run generation
    let duration = start.elapsed();
    assert!(duration < Duration::from_millis(100));
}
```

### 2. Validation Phase (10 min)

Run and verify:
```bash
cargo test doc_generation
cargo test --release # Performance tests
```

## Expected Deliverables

### Test Files
- `tests/doc_generation.rs` - Integration tests
- Unit tests in module files

### Test Coverage
- All formats tested
- Error cases covered
- Performance validated
- Edge cases handled

## Success Criteria Checklist

- [ ] Integration tests created
- [ ] All formats tested
- [ ] Performance tests added
- [ ] Error handling tested
- [ ] Tests passing
- [ ] Coverage adequate

## Duration Estimate

**Total: 30 minutes**
- Implementation: 20 minutes
- Validation: 10 minutes

## Dependencies

- C.1-C.3: Implementation complete

## Notes

- Use assert_cmd for CLI testing
- Test with minimal and full CLIs
- Verify backward compatibility

---

**Task Status**: ⬜ Not Started
**Created**: 2025-08-14
**Author**: Shadowcat Team