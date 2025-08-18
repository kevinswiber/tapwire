# Task E.0: Consolidate Selector Modules

## Objective
Resolve duplicate selector modules and establish clear selection strategy.

## Current State
- `src/proxy/reverse/selector.rs` - Main selector
- `src/proxy/reverse/upstream/selector.rs` - Upstream selector
- Likely duplicate functionality

## Steps
1. Compare both selector modules for overlap
2. Determine which functionality belongs where
3. Merge or clearly separate responsibilities
4. Update all references

## Success Criteria
- [ ] Single source of truth for upstream selection
- [ ] No duplicate code
- [ ] Clear module boundaries
- [ ] All tests pass