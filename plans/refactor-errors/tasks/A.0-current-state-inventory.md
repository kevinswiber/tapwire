# Task A.0: Current State Inventory

## Objective

Create a comprehensive inventory of all error types, Result aliases, and their usage patterns across the shadowcat codebase to inform the migration strategy.

## Background

The current error handling uses a centralized error.rs file containing all error enums and Result type aliases. Before modularizing, we need to understand:
- What error types exist
- Where they are used
- How they interconnect via From implementations
- What public APIs expose them

## Key Questions to Answer

1. What error enums are defined in error.rs?
2. What Result type aliases exist and how are they used?
3. Which errors have From implementations for ShadowcatError?
4. What is the public API surface for each error type?
5. Are there any circular dependencies between error types?

## Step-by-Step Process

### 1. Analysis Phase (30 min)
Examine the current error.rs structure

```bash
cd shadowcat
# Count error definitions
rg "pub enum \w+Error" src/error.rs
# Count Result aliases
rg "pub type \w+Result" src/error.rs
# Find From implementations
rg "impl From<" src/error.rs
```

### 2. Usage Mapping (1 hour)

Map where each error type is used:

```bash
# For each error type, find usage
rg "TransportError" --type rust -g '!target'
rg "SessionError" --type rust -g '!target'
rg "StorageError" --type rust -g '!target'
# ... continue for all error types

# Find Result alias usage
rg "TransportResult<" --type rust -g '!target'
rg "SessionResult<" --type rust -g '!target'
# ... continue for all Result types
```

### 3. Documentation Phase (30 min)

Create inventory document with:
- Error enum definitions and variants
- Result type aliases
- Usage counts per module
- From implementation graph
- Public API exposure analysis

## Expected Deliverables

### New Files
- `analysis/error-inventory.md` - Complete inventory of error types and usage
- `analysis/migration-map.md` - Mapping of what needs to migrate where

### Analysis Output Structure

```markdown
# Error Inventory

## Error Enums
- ShadowcatError: Top-level, X variants, used in Y files
- TransportError: Z variants, used in W files
  - Variants: [list]
  - Public API exposure: [functions]
- SessionError: ...
[continue for all]

## Result Type Aliases
- TransportResult<T>: Used in X locations
  - Internal: Y files
  - Public API: Z functions
[continue for all]

## From Implementation Graph
ShadowcatError <- TransportError
ShadowcatError <- SessionError
[show all relationships]

## Module Usage Patterns
transport/: Uses TransportError, TransportResult
session/: Uses SessionError, SessionResult
[continue for all modules]
```

## Success Criteria Checklist

- [ ] All error enums documented with variant counts
- [ ] All Result aliases documented with usage counts
- [ ] From implementation graph created
- [ ] Public API surface identified
- [ ] Module usage patterns mapped
- [ ] No circular dependencies found
- [ ] Analysis document created in analysis/

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Missing error types in inventory | MEDIUM | Use multiple search patterns, verify with compiler |
| Underestimating public API surface | HIGH | Check docs.rs, examples, and tests |

## Duration Estimate

**Total: 2 hours**
- Analysis: 30 minutes
- Usage mapping: 1 hour
- Documentation: 30 minutes

## Dependencies

None - this is the first task

## Integration Points

- **error.rs**: Primary analysis target
- **All modules**: Need to scan for usage
- **Public API**: lib.rs and module exports

## Performance Considerations

N/A - Analysis task only

## Notes

- Pay special attention to generic error variants that might need special handling
- Note any error types that seem incorrectly placed
- Identify opportunities for consolidation

## Commands Reference

```bash
cd shadowcat

# Quick inventory
rg "pub enum \w+Error" src/error.rs -A 5
rg "pub type \w+Result" src/error.rs

# Usage analysis
for error in Transport Session Storage Auth Config Intercept Recorder Proxy ReverseProxy; do
  echo "=== ${error}Error ==="
  rg "${error}Error" --type rust -g '!target' -c
done

# Public API check
rg "pub.*Result<" --type rust -g '!target'
```

## Follow-up Tasks

After completing this task:
- A.1: Analyze migration impact based on inventory
- A.2: Design compatibility strategy

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-18
**Last Modified**: 2025-01-18
**Author**: Kevin