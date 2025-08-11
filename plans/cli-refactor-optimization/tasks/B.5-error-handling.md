# B.5: Standardize Error Handling

**Status**: ⬜ Not Started  
**Duration**: 2 hours  
**Dependencies**: Phase A

## Objective

Ensure consistent error handling throughout the library with proper context and error chaining.

## Current Issues

1. Mix of `Result<T, ShadowcatError>` and `anyhow::Result<T>`
2. Some errors printed to stderr, others returned
3. Inconsistent error context and messages
4. Some library code still uses `println!` or `eprintln!`

## Implementation Plan

### 1. Audit Public APIs (30 minutes)
- [ ] Search for all public functions returning `anyhow::Result`
- [ ] List all instances of `println!` or `eprintln!` in library code
- [ ] Identify error conversion gaps

### 2. Standardize Error Types (45 minutes)
- [ ] Ensure all public APIs return `Result<T, ShadowcatError>`
- [ ] Add proper error conversion traits where needed
- [ ] Update documentation for error handling patterns

### 3. Add Error Context (30 minutes)
- [ ] Add `.context()` calls for all fallible operations
- [ ] Ensure error messages are descriptive and actionable
- [ ] Include relevant data in error messages (paths, URLs, etc.)

### 4. Remove Direct Output (15 minutes)
- [ ] Remove all `println!` and `eprintln!` from library code
- [ ] Move any necessary output to the CLI layer
- [ ] Use tracing/logging instead where appropriate

### 5. Update Examples (30 minutes)
- [ ] Update all examples to demonstrate proper error handling
- [ ] Show how to handle different error types
- [ ] Add comments explaining error handling patterns

## Success Criteria

- [ ] All public APIs return `Result<T, ShadowcatError>`
- [ ] No direct printing to stdout/stderr in library code
- [ ] Consistent error messages with context
- [ ] Clean error handling in examples
- [ ] All tests still passing
- [ ] No clippy warnings

## Commands to Run

```bash
# Audit for anyhow::Result
rg "anyhow::Result" src/ --glob '!main.rs' --glob '!cli/'

# Find println/eprintln in library
rg "println!|eprintln!" src/ --glob '!main.rs' --glob '!cli/'

# Check error types
rg "Result<.*>" src/ --glob '*.rs' | grep -v ShadowcatError

# Run tests after changes
cargo test --quiet
cargo clippy --all-targets -- -D warnings
```

## Notes

- Focus on library code first, CLI can keep using anyhow
- Ensure backwards compatibility where possible
- Consider adding error recovery examples
- Document error handling best practices in lib.rs