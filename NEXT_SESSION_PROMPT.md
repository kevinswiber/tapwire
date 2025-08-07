# Next Claude Session Prompt - Shadowcat Refactor Task 002

## Context

You are continuing the systematic refactoring of the Shadowcat Rust proxy codebase. **Task 001 (Remove All Unwrap Calls) has been successfully completed** - all 35 production unwrap calls have been eliminated, and the codebase now has zero panic points in production code.

## Your Current Objective

**Continue Phase 1 with Task 002: Fix Duplicate Error Types**

Based on the comprehensive review, there are duplicate error type definitions in `src/error.rs` that need to be consolidated for better maintainability and consistency.

## Essential Context Files

Please read these files to understand your current task:

1. **Task Definition**: `/Users/kevin/src/tapwire/plans/refactors/task-002-fix-duplicate-errors.md` (if it exists, otherwise refer to the review)
2. **Overall Refactor Plan**: `/Users/kevin/src/tapwire/plans/refactors/shadowcat-refactor-tracker.md` - Shows current progress and next steps
3. **Original Review**: `/Users/kevin/src/tapwire/reviews/shadowcat-comprehensive-review-2025-08-06.md` - Section on duplicate error types

## Working Directory

`/Users/kevin/src/tapwire/shadowcat`

## What Task 001 Accomplished

- ✅ **35 production unwraps eliminated** (560 → 525, remaining are test-only)
- ✅ **Added 4 new error variants**: `SystemTime`, `AddressParse`, `RequiredFieldMissing`, `InvalidUpstreamConfig`
- ✅ **All 341 tests passing**
- ✅ **Clean clippy output**
- ✅ **Zero panic points in production code**

## Your Task 002 Objectives

1. **Analyze `src/error.rs`** for duplicate error type definitions
2. **Consolidate duplicate error types** - Remove redundant definitions
3. **Ensure consistent error handling patterns** across the codebase
4. **Update any imports/usage** of the old duplicate types
5. **Verify all tests still pass**
6. **Update documentation** when complete

## Success Criteria for Task 002

- [ ] No duplicate error type definitions in codebase
- [ ] All error types have single, canonical definitions
- [ ] Consistent error handling patterns across modules
- [ ] All tests pass
- [ ] Clean clippy output
- [ ] Update task documentation when complete

## Commands to Use

```bash
# Check for duplicate error definitions
rg "enum.*Error" --type rust src/

# Run tests
cargo test

# Check clippy
cargo clippy -- -D warnings

# Check compilation
cargo check
```

## Current Phase Status

**Phase 1: Critical Safety (1/4 tasks complete)**
- ✅ Task 001: Remove All Unwrap Calls (COMPLETED)
- 🔄 Task 002: Fix Duplicate Error Types (CURRENT)
- ⏳ Task 003: Add Request Size Limits
- ⏳ Task 004: Fix Blocking IO in Async

## Important Notes

- Always use the TodoWrite tool to track your progress through the task
- Test frequently to catch any issues early
- Follow the same systematic approach used in Task 001
- Update the refactor tracker when Task 002 is complete
- The original review mentioned duplicate `ConfigurationError` and `AuthenticationError` definitions

## Model Usage Strategy

Use **OPUS for**:
- Initial analysis of duplicate error types
- Complex error consolidation decisions
- Architecture-level error handling patterns

Use **SONNET for**:
- Mechanical code changes (removing duplicates)
- Running validation commands
- Updating imports and references

Begin by reading the context files and analyzing the current state of error type definitions in `src/error.rs`.