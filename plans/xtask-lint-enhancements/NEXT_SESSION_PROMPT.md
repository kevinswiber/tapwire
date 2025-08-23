# Dylint Refactoring Session - Modularize shadowcat_lints

## Context
The `shadowcat_lints/src/lib.rs` file has grown to ~800 lines and contains three complete lint implementations plus utilities. This is becoming difficult to manage and needs to be refactored into a modular structure.

## Primary Task: Refactor shadowcat_lints into modules

### Current Structure Problems
- Single 800-line lib.rs file containing everything
- Mixed concerns: lint definitions, implementations, utilities, registration
- Hard to navigate and maintain
- Difficult to add new lints without increasing complexity

### Proposed Architecture

```
shadowcat_lints/
├── src/
│   ├── lib.rs              # Thin coordinator: exports, registration
│   ├── lints/
│   │   ├── mod.rs          # Re-exports all lints
│   │   ├── error_suffix.rs # NO_ERROR_SUFFIX lint
│   │   ├── panic_in_prod.rs # NO_PANIC_IN_PROD lint
│   │   └── debug_output.rs  # NO_DEBUG_OUTPUT lint
│   ├── utils/
│   │   ├── mod.rs          # Re-exports utilities
│   │   ├── test_detection.rs # is_test_function, is_non_library_path
│   │   └── macro_utils.rs   # macro_ancestry_message, classify_print_call
│   └── combined_pass.rs    # ShadowcatLints LateLintPass impl
```

### Design Considerations

1. **Separation of Concerns**
   - Each lint in its own module with declare_lint! and implementation
   - Shared utilities extracted to utils module
   - Combined pass stays separate but imports from modules

2. **Module Interfaces**
   - Each lint module exports: the lint declaration and any lint-specific helpers
   - Utils module exports all test detection and macro utilities
   - Main lib.rs just coordinates and registers

3. **Key Challenges to Address**
   - LateLintPass trait must be implemented in one place (combined_pass.rs)
   - Need to ensure all symbols are properly imported/exported
   - Must maintain dylint registration compatibility
   - Preserve all existing functionality and test detection logic

4. **Implementation Strategy**
   - Start by extracting utilities (least risky)
   - Then extract one lint at a time
   - Test after each extraction
   - Keep combined pass working throughout

### Specific Refactoring Steps

1. **Extract utilities first**
   ```rust
   // utils/test_detection.rs
   pub fn is_non_library_path(cx: &LateContext<'_>, span: Span) -> bool { ... }
   pub fn is_test_function(cx: &LateContext<'_>, hir_id: HirId) -> bool { ... }
   
   // utils/macro_utils.rs  
   pub fn macro_ancestry_message<'tcx>(...) -> Option<(&'static str, &'static str, bool)> { ... }
   pub fn classify_print_call<'tcx>(...) -> Option<PrintKind> { ... }
   ```

2. **Extract each lint**
   ```rust
   // lints/error_suffix.rs
   declare_lint! { pub NO_ERROR_SUFFIX, ... }
   pub fn check_error_variant(cx: &LateContext<'_>, variant: &Variant<'_>) { ... }
   ```

3. **Update combined pass**
   ```rust
   // combined_pass.rs
   use crate::lints::{error_suffix, panic_in_prod, debug_output};
   use crate::utils::{test_detection, macro_utils};
   
   impl<'tcx> LateLintPass<'tcx> for ShadowcatLints {
       // Delegate to module functions
   }
   ```

4. **Thin lib.rs**
   ```rust
   // lib.rs
   mod lints;
   mod utils;  
   mod combined_pass;
   
   pub use lints::*;
   use combined_pass::ShadowcatLints;
   
   // dylint registration functions
   ```

### Success Criteria
- [ ] All lints continue to work in VS Code
- [ ] `cargo dylint --all` runs without errors
- [ ] Each module is under 250 lines
- [ ] Clear separation of concerns
- [ ] Easy to add new lints

### Testing Plan
1. Run `cargo dylint --all` after each extraction
2. Test specific files that were previously flagged
3. Verify VS Code integration still works
4. Check that test exclusion still functions

## Reference
- Current tracker: `plans/xtask-lint-enhancements/xtask-lint-enhancements-tracker.md`
- Current lib.rs is ~800 lines with 3 lints implemented

## Commands to run
```bash
cd shadowcat_lints
cargo build --release
cd ..
cargo dylint --all
```

## Time Estimate
2-3 hours for complete refactoring with testing