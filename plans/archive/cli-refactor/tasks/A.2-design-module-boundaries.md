# Task A.2: Design Module Boundaries

## Objective
Design clear module boundaries and interfaces for the refactored CLI structure, ensuring clean separation of concerns and reusability.

## Key Questions to Answer
1. What should each module be responsible for?
2. How should modules communicate with each other?
3. What should be in the public API vs internal?
4. How to handle shared configuration?
5. Where should helper functions live?

## Process

### Step 1: Define Module Responsibilities
- [ ] Define scope for each command module
- [ ] Identify shared vs unique functionality
- [ ] Determine public interfaces
- [ ] Plan internal structure

### Step 2: Design Common Module
- [ ] Identify all shared components
- [ ] Design configuration management
- [ ] Plan utility functions
- [ ] Define error handling

### Step 3: Create Interface Contracts
- [ ] Define module public APIs
- [ ] Plan data flow between modules
- [ ] Design configuration passing
- [ ] Handle dependency injection

### Step 4: Plan Testing Strategy
- [ ] Identify testable boundaries
- [ ] Plan mock interfaces
- [ ] Design integration points
- [ ] Consider test helpers

## Expected Deliverables

### 1. Module Architecture (analysis/module-architecture.md)
```markdown
# Module Architecture Design

## Module Structure
cli/
├── mod.rs           # Public API and re-exports
├── common.rs        # Shared utilities
├── forward.rs       # Forward proxy
├── reverse.rs       # Reverse proxy
├── record.rs        # Recording
├── replay.rs        # Replay
├── handlers.rs      # HTTP handlers
└── [existing modules]

## Module Responsibilities
### common.rs
- ProxyConfig management
- Rate limiter creation
- Session manager factory
- Shared error types

### forward.rs
- StdioForward implementation
- HttpForward implementation
- Forward-specific configuration
```

### 2. Interface Design (analysis/interfaces.md)
```markdown
# Module Interfaces

## Public APIs
### Common Module
```rust
pub struct ProxyConfig { ... }
pub fn create_rate_limiter(config: &ProxyConfig) -> Result<Arc<RateLimiter>>
pub fn create_session_manager(config: &ProxyConfig) -> Arc<SessionManager>
```

### Forward Module
```rust
pub async fn execute_stdio(args: StdioArgs, config: ProxyConfig) -> Result<()>
pub async fn execute_http(args: HttpArgs, config: ProxyConfig) -> Result<()>
```
```

### 3. Data Flow Diagram (analysis/data-flow.md)
```markdown
# Data Flow Design

## Configuration Flow
main.rs → parse CLI → create ProxyConfig → pass to module → execute

## Module Communication
- Modules communicate through common types
- No direct module-to-module dependencies
- Shared state through Arc<T>
```

## Commands to Run
```bash
# Examine existing module structure
ls -la src/cli/

# Check existing module interfaces
grep "^pub" src/cli/*.rs

# Analyze import patterns
grep "^use" src/main.rs | sort | uniq
```

## Success Criteria
- [ ] Clear module boundaries defined
- [ ] No circular dependencies
- [ ] Minimal public API surface
- [ ] Testable interfaces
- [ ] Consistent patterns across modules

## Time Estimate
1 hour

## Notes
- Follow existing patterns from tape/intercept/session modules
- Keep interfaces simple and focused
- Consider future extensibility
- Maintain backward compatibility