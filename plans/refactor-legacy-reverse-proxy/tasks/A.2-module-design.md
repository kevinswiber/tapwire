# Task A.2: Module Design

## Objective
Design the target module structure with clear boundaries, interfaces, and responsibilities.

## Context
Based on the analysis from A.0 and A.1, we need to design a clean module architecture that:
- Maintains single responsibility
- Keeps modules under 500 lines
- Enables testing in isolation
- Supports future extensions

## Deliverables

### 1. Module Architecture
Create `analysis/module-architecture.md` with:
- Complete module tree
- Line count estimates per module
- Responsibility definitions
- Public API for each module
- Internal structure

### 2. Module Interfaces
Document in `analysis/module-interfaces.md`:
```rust
// Example for each module
pub trait McpHandler {
    async fn handle_request(&self, req: Request) -> Result<Response>;
}

pub trait UpstreamSelector {
    async fn select(&self, session: &Session) -> Result<Upstream>;
}
```

### 3. Data Flow Diagram
Create `analysis/data-flow.md` showing:
- Request flow through modules
- State management
- Session handling
- Error propagation

### 4. Migration Plan
Document in `analysis/migration-strategy.md`:
- Step-by-step extraction order
- Compatibility layers needed
- Testing strategy at each step
- Rollback points

## Process

### Step 1: Define Module Boundaries
For each identified component:
- Define clear responsibility
- Estimate line count
- List public interface
- Identify dependencies

### Step 2: Design Interfaces
Create trait definitions for:
- Handler abstraction
- Upstream management
- Session operations
- Middleware chain

### Step 3: Plan State Management
Design how state flows:
- Immutable config
- Shared app state
- Per-request context
- Session storage

### Step 4: Design Extension Points
Plan for future needs:
- Plugin system for handlers
- Custom middleware
- Alternative storage backends
- Protocol extensions

## Module Structure

### Config Module (~200 lines)
```rust
// config/mod.rs
pub use upstream::*;
pub use session::*;
pub use middleware::*;

// config/upstream.rs (~150 lines)
pub struct UpstreamConfig { ... }
pub enum LoadBalancing { ... }

// config/session.rs (~50 lines)
pub struct SessionConfig { ... }
```

### Server Module (~300 lines)
```rust
// server/mod.rs
pub struct ReverseProxyServer { ... }
pub use builder::ReverseProxyServerBuilder;

// server/builder.rs (~200 lines)
impl ReverseProxyServerBuilder { ... }

// server/state.rs (~100 lines)
pub(crate) struct AppState { ... }
```

### Handler Module (~400 lines each)
```rust
// handlers/mod.rs
pub trait Handler: Send + Sync {
    async fn handle(&self, ctx: RequestContext) -> Result<Response>;
}

// handlers/mcp.rs
pub struct McpHandler { ... }

// handlers/sse.rs  
pub struct SseHandler { ... }
```

## Success Criteria
- [ ] Every module has clear single responsibility
- [ ] No module exceeds 500 lines
- [ ] Clean interfaces defined
- [ ] No circular dependencies
- [ ] Extension points identified

## Estimated Time
3 hours

## Design Principles

### Single Responsibility
Each module does ONE thing:
- Config: Define configuration
- Server: Manage lifecycle
- Handler: Process requests
- Router: Route requests
- Upstream: Select backends

### Dependency Inversion
- Depend on abstractions (traits)
- Not concrete implementations
- Enable testing with mocks

### Open/Closed Principle
- Open for extension (traits)
- Closed for modification
- Add features without changing core

## Notes
- Consider making admin UI a separate crate
- Handler traits enable testing in isolation
- State management needs careful design to avoid locks
- Migration must be incremental to avoid breaking changes