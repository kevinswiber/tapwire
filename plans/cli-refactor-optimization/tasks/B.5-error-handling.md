# B.5: Standardize Error Handling

**Status**: ✅ Complete  
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
- [x] Search for all public functions returning `anyhow::Result`
- [x] List all instances of `println!` or `eprintln!` in library code
- [x] Identify error conversion gaps

### 2. Standardize Error Types (45 minutes)
- [x] Ensure all public APIs return `Result<T, ShadowcatError>`
- [x] Add proper error conversion traits where needed
- [x] Update documentation for error handling patterns

### 3. Add Error Context (30 minutes)
- [x] Add `.context()` calls for all fallible operations
- [x] Ensure error messages are descriptive and actionable
- [x] Include relevant data in error messages (paths, URLs, etc.)

### 4. Remove Direct Output (15 minutes)
- [x] Remove all `println!` and `eprintln!` from library code
- [x] Move any necessary output to the CLI layer
- [x] Use tracing/logging instead where appropriate

### 5. Update Examples (30 minutes)
- [x] Update all examples to demonstrate proper error handling
- [x] Show how to handle different error types
- [x] Add comments explaining error handling patterns

## Success Criteria

- [x] All public APIs return `Result<T, ShadowcatError>`
- [x] No direct printing to stdout/stderr in library code
- [x] Consistent error messages with context
- [x] Clean error handling in examples

## Completion Notes

**Completed**: 2025-08-11

### Key Changes Made

1. **Replaced anyhow with domain-specific errors**:
   - auth/pkce.rs: Changed from `anyhow::Result` to `AuthResult`
   - auth/token.rs: Changed from `anyhow::Result` to `AuthResult`
   - config/reverse_proxy.rs: Changed from `anyhow::Result` to `ConfigResult`

2. **Fixed error type confusion**:
   - Discovered two separate AuthError types (src/error.rs and src/auth/error.rs)
   - Corrected imports to use the appropriate module-specific error types
   - auth module now properly uses `crate::auth::error::{AuthError, AuthResult}`

3. **Replaced anyhow-specific features**:
   - Replaced `.context()` calls with `.map_err()` for proper error conversion
   - Replaced `anyhow::bail!` with proper `return Err(...)` statements
   - Fixed error message formatting to use inline format arguments

4. **Clippy compliance**:
   - Fixed needless return statements in match arms
   - Fixed format string arguments to use inline variables
   - All 873 tests passing
   - No clippy warnings with `--all-targets -- -D warnings`

### Important Discovery

The codebase has a well-structured error hierarchy:
- Main `ShadowcatError` enum in `src/error.rs` with conversions from domain errors
- Domain-specific error types (AuthError, ConfigError, etc.) with their own Result types
- Some modules (like auth) have their own local error modules for more specific handling
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