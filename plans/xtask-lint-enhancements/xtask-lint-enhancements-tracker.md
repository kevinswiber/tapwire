# Xtask Lint Enhancements - Migration to Dylint

## Overview
Migrating custom linting rules from xtask to dylint for better IDE integration and native Rust compiler support.

## Progress Summary
- ✅ **Phase 1**: Basic dylint setup and initial lints (COMPLETE)
- ✅ **Phase 2**: Core lints ported and working (COMPLETE)
- 🚧 **Phase 3**: Refactor into modules (IN PROGRESS - PRIORITY)
  - ✅ Created module structure (utils/, lints/)
  - ✅ Extracted test_detection utilities
  - ✅ Started extracting NO_ERROR_SUFFIX
  - 🚧 Extracting remaining lints
- ⏳ **Phase 4**: Port remaining structural lints (PENDING)
- ⏳ **Phase 5**: CI Integration (PENDING)
- ⏳ **Phase 6**: Documentation (PENDING)

## Detailed Task Status

| Task | Status | Notes |
|------|--------|-------|
| Setup dylint infrastructure | ✅ Complete | Working with VS Code |
| Port NO_ERROR_SUFFIX lint | ✅ Complete | Detects redundant Error suffix |
| Port NO_PANIC_IN_PROD lint | ✅ Complete | Detects unwrap/expect/panic |
| Fix false positives in NO_PANIC_IN_PROD | ✅ Complete | Fixed HIR traversal for tokio::test |
| Port NO_DEBUG_OUTPUT lint | ✅ Complete | Fixed by user - detecting println!/dbg! |
| Port cross-module error imports | ⏳ Pending | Complex structural lint |
| Port no root Error/Result in submodules | ⏳ Pending | Complex structural lint |
| Refactor into modules when needed | 🚧 In Progress | Extracting to utils/ and lints/ modules |
| Update CI to run dylint | ⏳ Pending | |
| Create documentation | ⏳ Pending | |

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

## Files Modified

### Core Implementation
- `shadowcat_lints/src/lib.rs` - Main lint implementations (~780 lines)
- `shadowcat_lints/Cargo.toml` - Dependencies and configuration
- `shadowcat_lints/ui/` - UI tests for lints

### Documentation
- `shadowcat_lints/examples/attributes.rs` - How to use allow/deny
- `shadowcat_lints/examples/panic_control.rs` - Examples for NO_PANIC_IN_PROD

### Integration
- `.vscode/settings.json` - VS Code rust-analyzer configuration
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

1. **Large lib.rs**: At ~780 lines, definitely needs modularization. Previous attempt had compilation issues with trait implementations. This is now a priority.

## Current Refactoring Status

### Completed
- ✅ Module structure created (utils/, lints/)
- ✅ Test detection utilities extracted to `utils/test_detection.rs`
- ✅ NO_ERROR_SUFFIX partially extracted to `lints/error_suffix.rs`

### In Progress
- 🚧 Completing modular refactoring
- 🚧 Extracting NO_PANIC_IN_PROD and NO_DEBUG_OUTPUT
- 🚧 Creating combined_pass.rs for LateLintPass implementation

### Next Steps
1. Complete lint extractions to separate modules
2. Create thin lib.rs coordinator
3. Test all lints still work
4. Port remaining structural lints
5. Update CI and documentation

## References
- [Dylint Documentation](https://github.com/trailofbits/dylint)
- [rustc Lint API](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/)
- [Clippy print_stdout lint](https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/write.rs)