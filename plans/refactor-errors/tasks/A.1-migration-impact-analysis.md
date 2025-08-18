# Task A.1: Migration Impact Analysis

## Objective

Analyze the impact of migrating to module-local Error and Result types, identifying potential breaking changes, import conflicts, and migration complexity for each module.

## Background

Based on the inventory from A.0, we need to understand:
- What code changes are required
- What might break for consumers
- Where import conflicts might occur
- How to prioritize the migration

## Key Questions to Answer

1. Which modules have the most error usage to migrate?
2. What public APIs would be affected?
3. Where might Result type conflicts occur?
4. What's the optimal migration order?
5. How can we minimize disruption?

## Step-by-Step Process

### 1. Public API Analysis (30 min)

Identify all public functions returning error types:

```bash
# Find public functions with Result returns
rg "pub.*fn.*-> .*Result" --type rust src/
rg "pub.*async fn.*-> .*Result" --type rust src/

# Check trait implementations
rg "impl.*for.*{" --type rust src/ -A 3 | grep Result
```

### 2. Import Conflict Analysis (30 min)

Identify potential naming conflicts:

```bash
# Find files that import multiple Result types
rg "use.*Result" --type rust src/ -g '!error.rs'

# Find unqualified Result usage
rg "\bResult<" --type rust src/ | grep -v "std::result::Result"
```

### 3. Migration Complexity Assessment (45 min)

For each module, assess:
- Number of files to change
- Public API changes required
- Test updates needed
- Documentation impact

Create complexity matrix:
```
Module      | Files | Public APIs | Tests | Complexity
------------|-------|-------------|-------|------------
transport   |   X   |      Y      |   Z   | HIGH/MED/LOW
session     |   X   |      Y      |   Z   | HIGH/MED/LOW
```

### 4. Documentation Phase (15 min)

Document findings in structured format for migration planning.

## Expected Deliverables

### New Files
- `analysis/migration-impact.md` - Detailed impact analysis
- `analysis/migration-order.md` - Recommended migration sequence

### Analysis Output Structure

```markdown
# Migration Impact Analysis

## Public API Impact
### High Impact (Breaking Changes)
- Function X in module Y returns TransportResult
- Trait Z requires SessionResult

### Low Impact (Internal Only)
- Internal function A
- Private module B

## Import Conflict Risk
### High Risk Modules
- module_name: Uses X different Result types
  - Current imports: [list]
  - Potential conflicts: [describe]

## Migration Complexity Matrix
| Module | Files | APIs | Tests | Risk | Priority |
|--------|-------|------|-------|------|----------|
| ...    | ...   | ...  | ...   | ...  | ...      |

## Recommended Migration Order
1. storage (lowest risk, good pilot)
2. config (few dependencies)
3. ...
```

## Success Criteria Checklist

- [ ] All public APIs analyzed for impact
- [ ] Import conflict risks identified
- [ ] Complexity matrix completed
- [ ] Migration order determined
- [ ] Risk assessment documented
- [ ] Mitigation strategies defined

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Underestimating public API changes | HIGH | Double-check with cargo doc |
| Missing indirect dependencies | MEDIUM | Use cargo tree to verify |
| Import conflicts in tests | LOW | Tests can use qualified paths |

## Duration Estimate

**Total: 2 hours**
- Public API analysis: 30 minutes
- Import conflict analysis: 30 minutes  
- Complexity assessment: 45 minutes
- Documentation: 15 minutes

## Dependencies

- A.0: Current State Inventory (must be complete)

## Integration Points

- **All modules**: Need to assess each
- **Public API**: lib.rs exports
- **Tests**: May need updates
- **Examples**: May need migration

## Performance Considerations

Compile-time impact:
- More type imports might slightly increase compile time
- Should measure before/after

## Notes

- Pay attention to generic functions that might need type annotations
- Consider macro-generated code that might be affected
- Note any modules that could benefit from error consolidation

## Commands Reference

```bash
cd shadowcat

# Public API analysis
rg "pub.*fn.*->.*Result" src/ --type rust

# Complexity assessment per module
for module in transport session storage auth config; do
  echo "=== $module ==="
  rg "${module}::.*Result" --type rust -c
  rg "use crate::error::.*${module}" --type rust -c
done

# Test impact
rg "Result<" tests/ --type rust -c
```

## Follow-up Tasks

After completing this task:
- A.2: Design compatibility strategy based on impact analysis
- B.1: Begin implementation with lowest-risk module

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-18
**Last Modified**: 2025-01-18
**Author**: Kevin