# Task B.2: Implement Error Variant Rule

## Objective
Port the "error variant naming" rule to dylint as a LateLintPass.

## Key Questions
- How do we detect enum Error specifically?
- How do we check variant names efficiently?
- How do we emit proper diagnostics with suggestions?

## Process

### 1. Define the lint
```rust
declare_lint! {
    pub NO_ERROR_SUFFIX,
    Warn,
    "enum Error variants should not end with 'Error'"
}
```

### 2. Implement LateLintPass
```rust
impl<'tcx> LateLintPass<'tcx> for NoErrorSuffix {
    fn check_variant(&mut self, cx: &LateContext<'tcx>, var: &'tcx Variant<'_>) {
        // 1. Check if parent is enum Error
        // 2. Check if variant name ends with "Error"
        // 3. Emit diagnostic with suggestion
    }
}
```

### 3. Key implementation points
- Use `cx.tcx.parent()` to get enum from variant
- Use `var.ident.name.as_str()` for variant name
- Provide fix suggestion to remove "Error" suffix
- Skip if variant would become empty after removal

### 4. Add tests
- Create UI tests in tests/ui/
- Test positive cases (violations)
- Test negative cases (valid code)
- Test edge cases (single word "Error")

### 5. Verify in editor
- Build the lint library
- Run `cargo dylint` on shadowcat codebase
- Check that violations appear
- Verify diagnostics are clear

## Deliverables
- [ ] no_error_suffix.rs implementation
- [ ] UI tests with expected output
- [ ] Working lint that finds violations in shadowcat

## Success Criteria
- [ ] Finds same violations as xtask version
- [ ] Provides clear diagnostic messages
- [ ] Suggests fixes (remove "Error" suffix)
- [ ] Can be suppressed with `#[allow(shadowcat::no_error_suffix)]`

## Example Code to Detect
```rust
// Should trigger:
enum Error {
    ConfigError,  // ❌ ends with Error
    ParseError,   // ❌ ends with Error
}

// Should not trigger:
enum Error {
    Config,       // ✅ no Error suffix
    Parse,        // ✅ no Error suffix
}
```

## Notes
- Reference clippy's enum_variant_names for similar logic
- Use `span_lint_and_sugg` for fix suggestions
- Consider using `applicability::MachineApplicable` for auto-fix