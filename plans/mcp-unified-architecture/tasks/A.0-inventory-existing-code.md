# A.0: Inventory Session & Interceptor Code

## Objective
Comprehensively analyze existing session management and interceptor implementations in shadowcat to understand what needs to be integrated into the MCP crate.

## Key Questions
1. What session management features exist in shadowcat that MCP needs?
2. How do interceptors currently work and what's their interface?
3. What's already been ported vs what's missing?
4. How do sessions and interceptors interact?

## Process

### 1. Session Management Analysis
- [ ] Review `src/session/` in shadowcat
  - [ ] Document SessionManager interface
  - [ ] Map session store implementations
  - [ ] Understand SSE integration points
  - [ ] Note persistence worker pattern
- [ ] Compare with `crates/mcp/src/` current state
  - [ ] Identify gaps in session handling
  - [ ] Note any conflicting implementations

### 2. Interceptor Analysis
- [ ] Review `src/interceptor/` in shadowcat
  - [ ] Document Interceptor trait
  - [ ] Map existing interceptor types
  - [ ] Understand rules engine integration
  - [ ] Note HTTP policy handling
- [ ] Check `crates/mcp/src/interceptor.rs`
  - [ ] Identify what's been ported
  - [ ] Note missing functionality

### 3. Integration Points
- [ ] Map how sessions flow through interceptors
- [ ] Identify shared dependencies
- [ ] Document configuration requirements
- [ ] Note any circular dependencies to resolve

## Deliverables

### 1. Session Inventory Document
Location: `analysis/session-inventory.md`

Structure:
```markdown
# Session Management Inventory

## Shadowcat Implementation
- Core components and responsibilities
- Store implementations (SQLite, Memory, Redis planned)
- Persistence patterns
- SSE integration approach

## Current MCP State
- What exists
- What's missing
- Conflicts to resolve

## Integration Requirements
- Required changes
- API compatibility needs
- Migration approach
```

### 2. Interceptor Inventory Document
Location: `analysis/interceptor-inventory.md`

Structure:
```markdown
# Interceptor Inventory

## Shadowcat Implementation
- Interceptor trait and engine
- Existing interceptor types
- Rules engine integration
- Action system

## Current MCP State
- Basic interceptor.rs analysis
- Missing components

## Integration Strategy
- Port order
- Interface changes needed
- Testing approach
```

### 3. Dependency Map
Location: `analysis/dependency-map.md`

Visual and textual representation of:
- Module dependencies
- Circular dependency issues
- Integration order recommendations

## Commands to Run
```bash
# Analyze shadowcat session code
rg "pub struct.*Session" ~/src/tapwire/shadowcat-mcp-compliance/src/session/
rg "pub trait" ~/src/tapwire/shadowcat-mcp-compliance/src/session/

# Analyze interceptor code
rg "pub trait.*Interceptor" ~/src/tapwire/shadowcat-mcp-compliance/src/interceptor/
rg "impl.*Interceptor" ~/src/tapwire/shadowcat-mcp-compliance/src/interceptor/

# Check what's in MCP already
ls -la crates/mcp/src/
grep -r "session\|Session" crates/mcp/src/
```

## Success Criteria
- [ ] Complete inventory of session management code
- [ ] Complete inventory of interceptor code
- [ ] Clear dependency map created
- [ ] Integration order determined
- [ ] No critical blockers identified

## Duration
4 hours

## Dependencies
None (first task)

## Notes
- Focus on understanding interfaces, not implementation details
- Document any shadowcat-specific assumptions that need addressing
- Note performance characteristics if mentioned in code