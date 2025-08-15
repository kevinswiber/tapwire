# Task A.1: Map Session ID Usage Across Codebase

## Objective
Create a comprehensive map of all places in the codebase that use session IDs to understand the full impact of implementing dual session tracking.

## Key Questions
1. Which components directly manipulate session IDs?
2. How are session IDs passed between components?
3. What assumptions exist about session ID format/ownership?
4. Which components need to know about proxy vs upstream IDs?

## Process

### 1. Code Search and Inventory
- [ ] Search for `SessionId` type usage
- [ ] Find `session_id` string literals in headers
- [ ] Locate `mcp-session-id` header references
- [ ] Identify session ID generation points

### 2. Component Analysis
For each component using session IDs:
- [ ] Transport layer (HTTP, SSE, stdio)
- [ ] Interceptor chain
- [ ] Rate limiting
- [ ] Recording/replay system
- [ ] Connection pooling
- [ ] Metrics and logging

### 3. Create Usage Matrix
Document for each usage:
- Component/file/function
- Purpose of session ID usage
- Whether it needs proxy or upstream ID
- Impact of dual tracking

### 4. Identify Integration Points
- [ ] Where session mapping lookups are needed
- [ ] Points where both IDs might be required
- [ ] Places needing special handling

## Deliverables
Create `analysis/session-id-usage-map.md` with:
1. Complete inventory of session ID usage
2. Usage matrix table
3. Dependency graph
4. Integration points for mapping layer

## Success Criteria
- [ ] Every session ID usage documented
- [ ] Clear understanding of which ID each component needs
- [ ] Integration points identified
- [ ] No hidden dependencies missed