# Task A.1: Dependency Mapping

## Objective
Map dependencies between modules to understand error propagation paths and identify potential circular dependency risks.

## Key Questions
1. What is the dependency hierarchy between modules?
2. How do errors currently flow through the system?
3. Where might circular dependencies occur?
4. What is the correct order for migration?

## Process

### Step 1: Extract Module Dependencies
```bash
# Find all internal crate imports
grep -rn "use crate::" src/ --include="*.rs" | grep -v "use crate::prelude" > /tmp/internal-imports.txt

# Find module-specific imports
for module in auth config transport session pool proxy recorder; do
  echo "=== $module dependencies ===" 
  grep -rn "use crate::" src/$module --include="*.rs" | grep -v "use crate::$module"
done
```

### Step 2: Build Dependency Graph

Create a visual representation of module dependencies:
```
transport (no deps)
    ↓
session (depends on transport)
    ↓
pool (depends on transport)
    ↓
proxy::forward (depends on transport, session, pool)
```

### Step 3: Identify Error Flow Paths

Trace how errors flow from low-level to high-level:
- pool::Error -> forward::Error -> crate::Error
- transport::Error -> session::Error -> forward::Error -> crate::Error

### Step 4: Detect Circular Risks

Look for potential circular dependencies:
- Module A imports from Module B
- Module B imports from Module A
- This would prevent proper error type separation

## Deliverables

### `/analysis/dependency-graph.md`
```markdown
# Module Dependency Graph

## Dependency Hierarchy
```
Level 0 (No internal deps):
- mcp
- telemetry  
- process

Level 1 (Basic deps):
- transport (deps: mcp)
- config (deps: none)
- auth (deps: config)

Level 2 (Core infrastructure):
- pool (deps: transport)
- session (deps: transport, mcp)
- recorder (deps: mcp)

Level 3 (High-level):
- proxy::forward (deps: transport, session, pool)
- proxy::reverse (deps: transport, auth, config)
- interceptor (deps: mcp, session)

Level 4 (API):
- api (deps: all)
```

## Circular Dependency Risks
- None identified / List any found
```

### `/analysis/error-flow.md`
```markdown
# Error Flow Analysis

## Current Error Flows

### Forward Proxy Flow
```
pool::Error -----> crate::Error (VIOLATION)
     ↓
Should be: pool::Error -> forward::Error -> crate::Error
```

### Reverse Proxy Flow
```
auth::Error -----> crate::Error (VIOLATION)
     ↓
Should be: auth::Error -> reverse::Error -> crate::Error
```

## Recommended Error Chains
1. Foundation errors bubble to operation errors
2. Operation errors bubble to crate::Error
3. Never skip levels
```

## Success Criteria
- [ ] Complete dependency graph created
- [ ] Error flow paths documented
- [ ] Circular dependency risks identified
- [ ] Migration order determined
- [ ] Analysis files created

## Time Estimate
2 hours

## Dependencies
- A.0 (Error Usage Analysis) must be complete

## Notes
- Focus on internal module dependencies
- External crate dependencies don't matter for this analysis
- Pay attention to test modules that might have different patterns