# Reverse Proxy Refactoring Strategy

## Overview
The reverse proxy module (src/proxy/reverse.rs) has grown to 3,465 lines and needs to be broken up into manageable submodules. We're using an incremental refactoring approach that keeps the code working throughout the process.

## File Structure Strategy

### Initial Setup (Current)
```
src/proxy/
├── reverse.rs          # 3,465 line monolith
└── reverse/            # New module structure
    ├── mod.rs          # Module coordinator
    └── upstream_response.rs  # First new module
```

### Phase 1: Move to Legacy Pattern
```
src/proxy/
└── reverse/
    ├── mod.rs          # Re-exports from legacy.rs initially
    ├── legacy.rs       # The old reverse.rs moved here
    └── upstream_response.rs  # New clean module
```

### Phase 2: Gradual Extraction
As we refactor, we'll extract functionality from legacy.rs into new modules:
```
src/proxy/
└── reverse/
    ├── mod.rs          # Gradually switching exports to new modules
    ├── legacy.rs       # Shrinking as code moves out
    ├── upstream_response.rs
    ├── sse_streaming.rs
    ├── json_processing.rs
    ├── admin_interface.rs
    ├── session_mapping.rs
    └── interceptor_integration.rs
```

### Phase 3: Completion
When legacy.rs has no more exports needed by mod.rs:
```
src/proxy/
└── reverse/
    ├── mod.rs          # Clean module exports only
    ├── upstream_response.rs
    ├── sse_streaming.rs
    ├── json_processing.rs
    ├── admin_interface.rs
    ├── session_mapping.rs
    └── interceptor_integration.rs
    # legacy.rs deleted!
```

## Implementation Steps

1. **Move reverse.rs to reverse/legacy.rs**
   - Simple file move, no code changes

2. **Create reverse/mod.rs**
   - Initially just re-export everything from legacy.rs
   - Document the refactoring strategy in comments
   - Add new module declarations as we create them

3. **Extract functionality incrementally**
   - Start with UpstreamResponse (Phase C.1)
   - Move SSE streaming logic (Phase C.3)
   - Extract admin interface (Phase D)
   - Continue until legacy.rs is empty

4. **Update imports throughout codebase**
   - Change from `use crate::proxy::reverse::*` 
   - To specific imports from submodules

## Benefits of This Approach

1. **Zero downtime** - Code keeps working throughout
2. **Clear progress** - Can see legacy.rs shrinking
3. **Incremental testing** - Test each extraction
4. **Easy rollback** - Can revert individual extractions
5. **Clear completion** - Done when legacy.rs is deleted

## Tracking Progress

We can track progress by:
- Lines of code in legacy.rs (starts at 3,465)
- Number of public items exported from legacy.rs
- Number of new clean modules created
- Test coverage of new modules

## Success Criteria

The refactoring is complete when:
1. legacy.rs can be deleted
2. All functionality is in appropriate submodules
3. Each module is under 500 lines
4. All tests still pass
5. No clippy warnings

## Current Status

- [x] Strategy documented
- [x] upstream_response.rs created
- [ ] Move reverse.rs to legacy.rs
- [ ] Create proper mod.rs
- [ ] Begin extraction process