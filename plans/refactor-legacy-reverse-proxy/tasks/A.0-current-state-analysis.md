# Task A.0: Current State Analysis

## Objective
Analyze the current `legacy.rs` file to understand its structure, dependencies, and identify natural module boundaries.

## Context
The `legacy.rs` file is 3,465 lines - a monolithic implementation that needs to be broken down. Before we can refactor, we need to understand what's in there and how it's organized.

## Deliverables

### 1. Code Structure Analysis
Create `analysis/current-structure.md` documenting:
- Major sections and their line counts
- Key structs and their responsibilities
- Public API surface
- Internal helper functions
- Test organization

### 2. Dependency Map
Create `analysis/dependencies.md` showing:
- External crate dependencies
- Internal module dependencies
- Circular dependency risks
- Database/storage interactions
- Network/transport usage

### 3. Complexity Hotspots
Identify in `analysis/complexity-hotspots.md`:
- Functions > 100 lines
- Deeply nested code (> 3 levels)
- High cyclomatic complexity areas
- Code duplication
- Mixed responsibilities

### 4. Integration Points
Document in `analysis/integration-points.md`:
- How it connects to SessionManager
- Interceptor chain integration
- Transport layer usage
- Authentication/authorization hooks
- Recording/replay integration

## Process

### Step 1: Line Count Analysis
```bash
# Get section breakdown
grep -n "^impl\|^pub struct\|^pub fn\|^async fn\|^mod\|^///.*#" legacy.rs

# Count test vs implementation
grep -n "^mod tests" legacy.rs
grep -n "#\[test\]" legacy.rs
```

### Step 2: Structural Analysis
- Identify major structs (ReverseProxyServer, configs, etc.)
- Map public functions and their purposes
- Identify helper functions and utilities
- Note any embedded HTML or large constants

### Step 3: Dependency Analysis
```bash
# External dependencies
rg "use (crate::|super::|std::|tokio::|axum::|hyper::)" legacy.rs

# Count usage of each module
rg "crate::" legacy.rs | cut -d':' -f3 | sort | uniq -c
```

### Step 4: Identify Natural Boundaries
Look for:
- Config-related code (lines 56-336 approx)
- Server setup code
- Request handlers
- Admin UI (HTML strings)
- Helper functions

## Success Criteria
- [ ] Complete structure map with line numbers
- [ ] Dependency graph created
- [ ] Natural module boundaries identified
- [ ] Complexity hotspots documented
- [ ] Integration points mapped

## Estimated Time
2 hours

## Tools & Commands

```bash
# Line count by section
awk '/^impl|^pub struct|^pub enum|^pub fn/ {print NR, $0}' legacy.rs

# Find large functions
awk '/^(pub )?async fn|^(pub )?fn/ {start=NR} /^}$/ {if(start) print NR-start, "lines starting at", start; start=0}' legacy.rs | sort -rn | head -20

# Identify embedded content
rg -n 'r#"' legacy.rs | head -20
```

## Notes
- The admin UI HTML is likely a large chunk that can be easily extracted
- Config types are already somewhat organized together
- Watch for tight coupling between handler functions and server state
- Some code has already been extracted to `streaming/` modules