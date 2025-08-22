# Task E.1: Rename Hyper Modules

## Objective
Remove hyper_ prefix from module names for cleaner organization.

## Files to Rename
- `hyper_raw_streaming.rs` → `streaming/raw.rs` or `sse/raw_stream.rs`
- `hyper_sse_intercepted.rs` → `streaming/intercepted.rs` or `sse/intercepted_stream.rs`

## Steps
1. Create new module structure (streaming/ or sse/)
2. Move and rename files
3. Update all imports and references
4. Update module exports in mod.rs

## Success Criteria
- [ ] No hyper_ prefixed files
- [ ] Clear module organization
- [ ] All imports updated
- [ ] All tests pass