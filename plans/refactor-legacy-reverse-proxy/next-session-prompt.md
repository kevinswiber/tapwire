# Next Session Prompt: Phase G - Final SSE Extraction

## Current Status
- **legacy.rs**: 903 lines (74% reduction from original 3,465)
- **Branch**: `refactor/legacy-reverse-proxy` in shadowcat submodule
- **Compilation**: ✅ All code compiles successfully

## Primary Objective: Extract SSE Handler (Final Phase)

Extract the remaining `handle_mcp_sse_request` function from legacy.rs to complete the refactoring.

### Tasks:
1. **Extract SSE Handler** (~150 lines)
   - Move `handle_mcp_sse_request` to `handlers/sse.rs`
   - Update all imports and references
   - Ensure mcp.rs handler properly delegates

2. **Clean Up Tests**
   - Determine which tests should stay vs be deleted
   - Move integration tests to appropriate test files
   - Remove obsolete test code

3. **Final Verification**
   - Ensure all tests pass
   - Verify no functionality lost
   - Check that legacy.rs can be deleted (or is under 500 lines)

## Context
- Most functionality has been successfully extracted
- Clean module structure established
- Only SSE handler and tests remain in legacy.rs

## Success Criteria
- [ ] SSE handler extracted to handlers/sse.rs
- [ ] All tests passing
- [ ] legacy.rs under 500 lines OR completely removed
- [ ] No duplicate code or unnecessary imports

## Reference
Full tracker: `/Users/kevin/src/tapwire/plans/refactor-legacy-reverse-proxy/feature-tracker.md`