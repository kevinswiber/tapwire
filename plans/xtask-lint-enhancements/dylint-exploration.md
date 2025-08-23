# Dylint Exploration Plan

**Status**: Phase 1 Complete, Phase 2 In Progress  
**Last Updated**: 2025-08-23

## Overview
Explore migrating custom lint rules from xtask to dylint for better IDE integration and standardization.

## Current State
We have 6 custom lint rules implemented in xtask:
1. **Error variant naming** - Check Error enum variants don't end with "Error"
2. **Config validation boundaries** - Only config module may impl Validate trait
3. **Cross-module error imports** - Modules shouldn't import sibling Error types  
4. **No root Error/Result in submodules** - Submodules shouldn't import root types
5. **No unwrap/expect/panic** - Detect panic-prone calls in production code
6. **No debug output** - Detect println!/eprintln!/dbg! in production code

## Proposed Architecture

### Move to Dylint (AST/HIR-based):
- Error variant naming
- Cross-module error imports
- No root Error/Result in submodules
- No unwrap/expect/panic (or leverage clippy's with custom policy)
- No debug output (or leverage clippy's with custom policy)

### Keep in xtask (Architecture/Policy):
- Config validation boundaries
- Future cross-module architectural checks
- Integration/orchestration of dylint + clippy + custom checks

## Benefits of Migration
1. **Native IDE Integration**: Real squiggles in VS Code via rust-analyzer
2. **Standard Annotations**: Use `#[allow(shadowcat::lint_name)]` 
3. **Fix Suggestions**: Machine-applicable fixes in editor
4. **Performance**: Compiled lints vs interpreted AST walking
5. **Reuse**: Leverage rustc/clippy infrastructure

## Implementation Steps

### Phase 1: Setup & Proof of Concept ✅
1. ✅ Create dylint library structure
2. ✅ Implement simplest rule (error variant naming) as LateLintPass
3. ✅ Configure VS Code integration
4. ✅ Verify squiggles and diagnostics work

### Phase 2: Migration
1. Port remaining AST-based rules to dylint
   - [ ] Cross-module error imports
   - [ ] No root Error/Result in submodules
   - [ ] No unwrap/expect/panic
   - [ ] No debug output
2. Keep config validation in xtask
3. Update CI to run both dylint and xtask checks

### Phase 3: Optimization
1. Tune performance and diagnostic quality
2. Add fix suggestions where applicable
3. Document escape hatches and configuration

## Technical Notes

### Dylint Setup
```toml
# workspace Cargo.toml
[workspace.metadata.dylint]
libraries = [{ path = "shadowcat_lints" }]
```

### VS Code Config
```json
{
  "rust-analyzer.check.overrideCommand": [
    "cargo", "dylint", "--all", "--", 
    "--all-targets", "--message-format=json"
  ]
}
```

### Example LateLintPass Structure
```rust
declare_lint! {
    pub NO_ERROR_SUFFIX,
    Warn,
    "enum Error variants should not end with 'Error'"
}

impl<'tcx> LateLintPass<'tcx> for NoErrorSuffix {
    fn check_variant(&mut self, cx: &LateContext<'tcx>, var: &'tcx Variant<'_>) {
        // Check if in enum Error and variant ends with "Error"
    }
}
```

## Implementation Details

### NO_ERROR_SUFFIX Lint (Completed)
- **Location**: `shadowcat_lints/src/lib.rs`
- **Features**:
  - Detects Error enum variants ending with "Error" suffix
  - Supports enum-level and variant-level `#[allow(...)]` attributes
  - Supports `#[deny(...)]` with proper escalation to compilation errors
  - Uses `Applicability::Unspecified` for suggestions (requires symbol rename)
- **Key Decisions**:
  - Not auto-applicable because renaming requires updating all references
  - Uses `span_lint_hir_and_then` for proper lint level handling
  - Workspace separation to avoid panic strategy conflicts

### Workspace Configuration
- **Challenge**: panic=abort conflict with cdylib requirements
- **Solution**: shadowcat_lints as standalone workspace (not member)
- **Benefits**: Clean separation, no panic strategy conflicts

## Risks & Mitigations
- **Learning Curve**: rustc lint API is complex → Start simple, iterate ✅
- **Build Times**: dylint adds compilation → Cache aggressively
- **Debugging**: Harder to debug compiled lints → Good logging, tests
- **Panic Strategy Conflicts**: cdylib requires panic=unwind → Separate workspace ✅

## Success Criteria
- [x] Dylint rules show squiggles in VS Code
- [ ] Performance comparable or better than xtask
- [x] Developers can use `#[allow(...)]` annotations  
- [x] Support for `#[deny(...)]` annotations with proper escalation
- [ ] CI runs both dylint and remaining xtask checks