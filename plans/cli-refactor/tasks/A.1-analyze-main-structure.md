# Task A.1: Analyze main.rs Structure

## Objective
Perform a comprehensive analysis of the current main.rs implementation to understand all functionality that needs to be extracted and modularized.

## Key Questions to Answer
1. What are all the command handlers and their dependencies?
2. Which functions are tightly coupled to main.rs?
3. What configuration is shared across commands?
4. How is error handling currently implemented?
5. What helper functions could be shared utilities?

## Process

### Step 1: Catalog All Components
- [ ] List all command enums and their variants
- [ ] Identify all handler functions (run_* functions)
- [ ] Document helper functions and utilities
- [ ] Map configuration structures and their usage

### Step 2: Analyze Dependencies
- [ ] Track which commands share configuration (ProxyConfig)
- [ ] Identify shared HTTP handlers
- [ ] Note rate limiter initialization patterns
- [ ] Document session manager usage

### Step 3: Identify Duplication
- [ ] Find repeated code blocks
- [ ] Note similar patterns across handlers
- [ ] Identify configuration duplication
- [ ] Document error handling patterns

### Step 4: Measure Complexity
- [ ] Count lines per command handler
- [ ] Identify most complex functions
- [ ] Note testing challenges
- [ ] Document coupling issues

## Expected Deliverables

### 1. Component Inventory (analysis/main-components.md)
```markdown
# Main.rs Component Inventory

## Command Structure
- Commands enum (X variants)
- ForwardTransport enum (X variants)
- RecordTransport enum (X variants)

## Handler Functions
- run_stdio_forward() - X lines
- run_http_forward_proxy() - X lines
- ...

## Helper Functions
- json_to_transport_message() - X lines
- ...

## Configuration
- ProxyConfig struct
- Usage in X commands
```

### 2. Dependency Map (analysis/dependencies.md)
```markdown
# Dependency Analysis

## Shared Dependencies
- ProxyConfig: used by [commands]
- SessionManager: used by [commands]
- RateLimiter: used by [commands]

## Command-Specific Dependencies
- Forward: [list]
- Reverse: [list]
- ...
```

### 3. Refactoring Opportunities (analysis/opportunities.md)
```markdown
# Refactoring Opportunities

## High Priority
1. Extract ProxyConfig to common module
2. Consolidate rate limiter initialization
3. ...

## Medium Priority
1. Share HTTP handler utilities
2. ...

## Low Priority
1. ...
```

## Commands to Run
```bash
# Count lines in main.rs
wc -l src/main.rs

# Check for TODO/FIXME comments
grep -n "TODO\|FIXME" src/main.rs

# Analyze function sizes
grep -n "^async fn\|^fn" src/main.rs

# Check test coverage
cargo test --bin shadowcat
```

## Success Criteria
- [ ] Complete inventory of all main.rs components
- [ ] Clear understanding of shared vs unique functionality
- [ ] Identified all refactoring opportunities
- [ ] Documented current pain points
- [ ] Created priority list for extraction

## Time Estimate
1 hour

## Notes
- Focus on understanding, not changing code yet
- Document everything for future reference
- Pay special attention to error handling patterns
- Note any potential breaking changes