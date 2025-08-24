# Xtask Lint Enhancements - Migration to Dylint

## Overview
Migrating custom linting rules from xtask to dylint for better IDE integration and native Rust compiler support.

## Progress Summary
- ✅ **Phase 1**: Basic dylint setup and initial lints (COMPLETE)
- ✅ **Phase 2**: Core lints ported and working (COMPLETE)
- ✅ **Phase 3**: Refactor into modules (COMPLETE)
  - ✅ Created module structure (utils/, lints/)
  - ✅ Extracted test_detection utilities
  - ✅ Extracted all three lints to separate modules
  - ✅ Created combined_pass.rs for LateLintPass implementation
  - ✅ Reduced lib.rs to thin coordinator (43 lines)
- ✅ **Phase 4**: Port remaining structural lints (COMPLETE)
  - ✅ Ported CROSS_MODULE_ERROR_IMPORTS
  - ✅ Ported NO_ROOT_ERROR_IMPORTS
  - ✅ Created early_pass.rs for structural lints
- ✅ **Phase 5**: CI Integration (COMPLETE)
  - ✅ Updated .github/workflows/architecture.yml
  - ✅ Runs both dylint and xtask checks
- ✅ **Phase 6**: Documentation (COMPLETE)
  - ✅ Created comprehensive README.md
  - ✅ Documented all lints and usage

## Detailed Task Status

| Task | Status | Notes |
|------|--------|-------|
| Setup dylint infrastructure | ✅ Complete | Working with VS Code |
| Port NO_ERROR_SUFFIX lint | ✅ Complete | Detects redundant Error suffix |
| Port NO_PANIC_IN_PROD lint | ✅ Complete | Detects unwrap/expect/panic |
| Fix false positives in NO_PANIC_IN_PROD | ✅ Complete | Fixed HIR traversal for tokio::test |
| Port NO_DEBUG_OUTPUT lint | ✅ Complete | Fixed by user - detecting println!/dbg! |
| Simplify lints using Clippy patterns | ✅ Complete | Reduced code by 61 lines |
| Port cross-module error imports | ✅ Complete | Early pass lint implemented |
| Port no root Error/Result in submodules | ✅ Complete | Early pass lint implemented |
| Refactor into modules when needed | ✅ Complete | Successfully modularized all lints |
| Update CI to run dylint | ✅ Complete | architecture.yml updated |
| Create documentation | ✅ Complete | Comprehensive README created |

## Current Issues

### All major issues resolved!
- NO_DEBUG_OUTPUT fixed by user with improved macro detection
- NO_PANIC_IN_PROD false positives fixed with proper HIR traversal for tokio::test

## Completed Lints

### 1. NO_ERROR_SUFFIX
- Ensures Error enum variants don't end with "Error"
- Uses `Applicability::Unspecified` for manual refactoring
- Has UI tests

### 2. NO_PANIC_IN_PROD  
- Detects unwrap(), expect(), panic!(), todo!(), unimplemented!()
- Properly excludes:
  - Test files and modules  
  - Example files
  - Benchmark files
  - Functions with #[tokio::main] or #[tokio::test] (fixed HIR traversal)
  - Functions with #[test] or #[bench] attributes
- Respects #[allow(no_panic_in_prod)]
- Uses HIR traversal to find actual enclosing function for accurate detection

### 3. NO_DEBUG_OUTPUT
- Detects println!, print!, eprintln!, eprint!, dbg! macros in production code
- Fixed by user with improved macro detection using multiple strategies
- Properly excludes test/example/bench code
- Respects #[allow(no_debug_output)]

### 4. CROSS_MODULE_ERROR_IMPORTS (NEW)
- Prevents cross-module error type imports that violate module boundaries
- Each module should define its own error types
- Implemented as early lint pass
- Successfully detecting violations in codebase

### 5. NO_ROOT_ERROR_IMPORTS (NEW)
- Prevents submodules from importing root Error/Result types
- Root types are for public API surface only
- Implemented as early lint pass
- Encourages domain-specific error types

## Files Modified

### Core Implementation (After Modularization)
- `shadowcat_lints/src/lib.rs` - Thin coordinator with dylint_library! macro (49 lines)
- `shadowcat_lints/src/combined_pass.rs` - LateLintPass implementation (143 lines)
- `shadowcat_lints/src/early_pass.rs` - EarlyLintPass for structural lints (23 lines)
- `shadowcat_lints/src/lints/error_suffix.rs` - NO_ERROR_SUFFIX lint (47 lines)
- `shadowcat_lints/src/lints/panic_in_prod.rs` - NO_PANIC_IN_PROD lint (101 lines)
- `shadowcat_lints/src/lints/debug_output.rs` - NO_DEBUG_OUTPUT lint (62 lines)
- `shadowcat_lints/src/lints/cross_module_errors.rs` - CROSS_MODULE_ERROR_IMPORTS (149 lines)
- `shadowcat_lints/src/lints/root_imports.rs` - NO_ROOT_ERROR_IMPORTS (130 lines)
- `shadowcat_lints/src/utils/test_detection.rs` - Test detection utilities (211 lines)
- `shadowcat_lints/README.md` - Comprehensive documentation (134 lines)
- `shadowcat_lints/Cargo.toml` - Dependencies and configuration
- `shadowcat_lints/ui/` - UI tests for lints

### Documentation
- `shadowcat_lints/examples/attributes.rs` - How to use allow/deny
- `shadowcat_lints/examples/panic_control.rs` - Examples for NO_PANIC_IN_PROD

### Integration
- `.vscode/settings.json` - VS Code rust-analyzer configuration
- `.github/workflows/architecture.yml` - CI pipeline with dylint and xtask
- Uses: `cargo dylint --all -- --all-targets --message-format=json`

## VS Code Integration
```json
"rust-analyzer.check.overrideCommand": [
    "cargo",
    "dylint",
    "--all",
    "--",
    "--all-targets",
    "--message-format=json"
]
```

## Commands

### Build
```bash
cd shadowcat_lints
cargo build --release
```

### Run Lints
```bash
cargo dylint --all
```

### Run UI Tests
```bash
cargo test -p shadowcat_lints ui
```

## Known Issues

None currently - all major issues have been resolved.

## Module Structure (Completed Refactoring)

### Final Architecture
- `lib.rs` - Thin coordinator with dylint_library! macro
- `combined_pass.rs` - LateLintPass implementation delegating to modules
- `early_pass.rs` - EarlyLintPass for structural lints
- `lints/`
  - `error_suffix.rs` - NO_ERROR_SUFFIX lint
  - `panic_in_prod.rs` - NO_PANIC_IN_PROD lint  
  - `debug_output.rs` - NO_DEBUG_OUTPUT lint
  - `cross_module_errors.rs` - CROSS_MODULE_ERROR_IMPORTS lint
  - `root_imports.rs` - NO_ROOT_ERROR_IMPORTS lint
- `utils/`
  - `test_detection.rs` - Shared test detection utilities

### Achievements
- ✅ All 5 lints successfully ported from xtask to dylint
- ✅ Simplified implementation using Clippy patterns (reduced 61 lines)
- ✅ Clean modular architecture with separation of concerns
- ✅ Full CI integration running both dylint and xtask
- ✅ Comprehensive documentation and examples
- ✅ Follows standard dylint conventions

## References
- [Dylint Documentation](https://github.com/trailofbits/dylint)
- [rustc Lint API](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/)
- [Clippy print_stdout lint](https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/write.rs)