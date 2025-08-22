# Task E.3: Extract Remaining Handler Logic

## Objective
Move remaining handler functions from legacy.rs to appropriate modules.

## Functions to Extract
- `handle_mcp_request` (~200 lines)
- `handle_mcp_sse_request` (~150 lines)
- Handler helper functions

## Target Structure
```
handlers/
├── mod.rs
├── mcp.rs (existing, enhance)
├── health.rs (existing)
└── helpers.rs (new, for shared logic)
```

## Steps
1. Move main handler functions to handlers/mcp.rs
2. Create handlers/helpers.rs for shared logic
3. Update all references
4. Remove from legacy.rs

## Success Criteria
- [ ] All handlers in handlers/ directory
- [ ] Legacy.rs under 1,800 lines
- [ ] Clean separation of concerns
- [ ] All tests pass