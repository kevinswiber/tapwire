# Next Session: Dylint Proof of Concept

## Context
We're exploring migrating our custom lint rules from xtask to dylint for better IDE integration. Currently on branch `refactor/dylint`.

## Objective
Implement a proof of concept by setting up dylint and porting the "error variant naming" rule.

## Tasks to Complete

### 1. Setup Dylint Library (Task B.1)
Reference: `plans/xtask-lint-enhancements/tasks/B.1-setup-dylint-library.md`

- Install dylint tools: `cargo install cargo-dylint dylint-link`
- Create shadowcat_lints library with `cargo dylint new`
- Configure workspace integration
- Verify basic compilation

### 2. Implement Error Variant Rule (Task B.2)
Reference: `plans/xtask-lint-enhancements/tasks/B.2-implement-error-variant-rule.md`

- Port the "no Error suffix in enum Error" rule
- Create as LateLintPass
- Add UI tests
- Verify it finds same violations as xtask version

### 3. VS Code Integration
- Configure rust-analyzer to use dylint
- Test that squiggles appear in editor
- Verify `#[allow(shadowcat::no_error_suffix)]` works

## Success Criteria
- [ ] `cargo dylint list` shows shadowcat_lints
- [ ] Running `cargo dylint` finds error variant violations
- [ ] VS Code shows squiggles for violations
- [ ] Can suppress with `#[allow(...)]` annotations

## Resources
- Dylint repo: https://github.com/trailofbits/dylint
- Plan tracker: `plans/xtask-lint-enhancements/xtask-lint-enhancements-tracker.md`
- Current xtask implementation: `xtask/src/lint/boundaries.rs::check_error_variants()`

## Notes
- We're on branch `refactor/dylint`
- Focus on proof of concept - just get one rule working
- If successful, we'll migrate remaining rules in Phase C