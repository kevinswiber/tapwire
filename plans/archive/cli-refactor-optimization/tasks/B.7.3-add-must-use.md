# Task B.7.3: Add Must-Use Attributes

**Status**: ⬜ Not Started  
**Estimated Duration**: 0.5 hours  
**Dependencies**: B.3 (High-Level API)  
**Priority**: MEDIUM  

## Context

From the [Comprehensive Review](../../../reviews/cli-refactor-optimization/comprehensive-review.md#92-medium-priority), handle types should have `#[must_use]` attributes to prevent accidental drops.

## Problem

Handle types like `ForwardProxyHandle`, `ReverseProxyHandle`, etc. can be accidentally dropped without being awaited, leading to:
- Silent failures
- Resource leaks
- Unexpected behavior

## Solution

Add `#[must_use]` attributes to all handle types with descriptive messages:

```rust
#[must_use = "Handle must be awaited or explicitly shut down"]
pub struct ForwardProxyHandle {
    // ...
}

#[must_use = "Server handle must be awaited or explicitly shut down"]
pub struct ReverseProxyHandle {
    // ...
}

#[must_use = "Recording handle must be stopped to save the tape"]
pub struct RecordingHandle {
    // ...
}

#[must_use = "Replay handle must be awaited for completion"]
pub struct ReplayHandle {
    // ...
}
```

## Implementation Steps

1. [ ] Open `src/api.rs`
2. [ ] Add `#[must_use]` to `ForwardProxyHandle` (line ~424)
3. [ ] Add `#[must_use]` to `ReverseProxyHandle` (line ~464)
4. [ ] Add `#[must_use]` to `RecordingHandle` (line ~500)
5. [ ] Add `#[must_use]` to `ReplayHandle` (line ~530)
6. [ ] Check for any other handle types that need the attribute
7. [ ] Verify compilation with the new attributes
8. [ ] Fix any warnings that appear in existing code

## Files to Modify

- `src/api.rs` - All handle structs

## Testing

- [ ] Compile with `cargo build`
- [ ] Run `cargo clippy -- -D warnings`
- [ ] Create a test that drops a handle without awaiting (should get warning)
- [ ] Verify existing code properly uses handles

## Success Criteria

- [ ] All handle types have `#[must_use]` attributes
- [ ] Descriptive messages explain what to do with each handle
- [ ] No new clippy warnings in existing code
- [ ] Compilation succeeds