# Task A.0: Error Usage Analysis

## Objective
Analyze and document all current uses of `crate::Error` and `crate::Result` in submodules to understand the scope of boundary violations.

## Key Questions
1. Which modules are directly referencing `crate::Error`?
2. Which modules are using `crate::Result`?
3. What patterns of usage exist?
4. Which modules already have their own Error types?
5. What are the highest-violation modules?

## Process

### Step 1: Collect Raw Data
```bash
# Find all crate::Error references
grep -rn "crate::Error" src/ --include="*.rs" | grep -v "^src/lib.rs" | grep -v "^src/main.rs" > /tmp/crate-error-refs.txt

# Find all crate::Result references  
grep -rn "crate::Result" src/ --include="*.rs" | grep -v "^src/lib.rs" | grep -v "^src/main.rs" > /tmp/crate-result-refs.txt

# Find imports of crate::Error or crate::Result
grep -rn "use crate::{.*Error.*}" src/ --include="*.rs" > /tmp/error-imports.txt
grep -rn "use crate::Result" src/ --include="*.rs" > /tmp/result-imports.txt

# Find functions returning crate::Result
grep -rn "-> crate::Result" src/ --include="*.rs" > /tmp/result-returns.txt

# Find Error:: usage (enum variant construction)
grep -rn "crate::Error::" src/ --include="*.rs" > /tmp/error-construction.txt
```

### Step 2: Analyze Module Status
```bash
# Which modules have their own Error type
grep -rn "pub enum Error" src/ --include="*.rs" > /tmp/modules-with-errors.txt

# Which modules have Result type alias
grep -rn "pub type Result" src/ --include="*.rs" > /tmp/modules-with-result.txt
```

### Step 3: Categorize Violations

Group violations by type:
1. **Import violations** - `use crate::Error` or `use crate::Result`
2. **Return type violations** - Functions returning `crate::Result`
3. **Construction violations** - Creating `crate::Error::SomeVariant`
4. **Propagation violations** - Using `?` to propagate to crate::Error

### Step 4: Document High-Risk Modules

Identify modules with the most violations:
- Count violations per module
- Note which types of violations
- Assess difficulty of fixing

## Deliverables

### `/analysis/current-error-usage.md`
```markdown
# Current Error Usage Analysis

## Summary Statistics
- Total crate::Error references: X
- Total crate::Result references: Y
- Modules affected: Z

## Violations by Module
| Module | Error Refs | Result Refs | Has Own Error? | Priority |
|--------|------------|-------------|----------------|----------|
| ... | ... | ... | ... | ... |

## Violation Patterns
1. Pattern: ...
   - Example: ...
   - Modules affected: ...
   - Fix approach: ...

## High-Risk Modules
1. module_name (X violations)
   - Primary issue: ...
   - Dependencies: ...
   - Suggested approach: ...
```

### `/analysis/module-error-status.md`
```markdown
# Module Error Type Status

## Modules With Error Types
- ✅ auth - Has Error and Result
- ✅ config - Has Error and Result
- ...

## Modules Needing Error Types
- ❌ audit - No Error type, X violations
- ❌ telemetry - No Error type, Y violations
- ...

## Modules With Partial Implementation
- ⚠️ module - Has Error but uses crate::Result
- ...
```

## Success Criteria
- [ ] All violations documented and counted
- [ ] Patterns of misuse identified
- [ ] High-risk modules prioritized
- [ ] Clear data for migration planning
- [ ] Analysis files created and populated

## Time Estimate
3 hours

## Dependencies
None - this is the first task

## Notes
- Be thorough - this analysis drives all subsequent work
- Pay attention to indirect violations (modules that import and re-export)
- Note any surprising or complex patterns