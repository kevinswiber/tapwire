# Task A.2: Migration Strategy

## Objective
Create a detailed migration strategy based on the analysis, defining the order and approach for fixing each module.

## Key Questions
1. What order should modules be migrated?
2. How do we handle modules with many violations?
3. What patterns should we establish?
4. How do we minimize risk?

## Process

### Step 1: Define Migration Order

Based on dependency analysis, establish order:
1. **Foundation** - Modules with no dependencies
2. **Core** - Low-level infrastructure
3. **Operations** - Business logic modules
4. **API** - High-level interfaces

### Step 2: Define Standard Pattern

Document the standard pattern for module errors:
```rust
// src/module/error.rs or src/module/mod.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // Module-specific errors
    #[error("Something failed: {0}")]
    SomethingFailed(String),
    
    // Dependencies (NOT crate::Error)
    #[error("Transport error")]
    Transport(#[from] transport::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Step 3: Create Migration Checklist

For each module:
- [ ] Create module Error enum
- [ ] Create module Result type alias
- [ ] Update all function signatures
- [ ] Update error construction
- [ ] Add #[from] conversions
- [ ] Update parent module
- [ ] Run tests
- [ ] Fix clippy warnings

### Step 4: Risk Assessment

Identify and plan for risks:
- Breaking changes
- Test failures
- Performance impact
- Merge conflicts

## Deliverables

### `/analysis/migration-strategy.md`
```markdown
# Migration Strategy

## Migration Order

### Phase 1: Foundation (No Dependencies)
1. telemetry - 2 hours
2. process - 2 hours
3. mcp - 3 hours
Rationale: These have no internal dependencies, safe to change

### Phase 2: Core Infrastructure
1. pool - 2 hours (already has Error)
2. transport - 3 hours (already has Error)
3. session - 3 hours (already has Error)
Rationale: Other modules depend on these

### Phase 3: Auth & Config
1. auth - 4 hours (many references)
2. config - 4 hours (many references)
Rationale: Required by proxy modules

### Phase 4: Proxy Modules
1. proxy::forward - 4 hours
2. proxy::reverse - 4 hours
Rationale: High-level operations

### Phase 5: API Layer
1. api - 3 hours
2. main.rs - 2 hours
Rationale: Final cleanup

## Migration Patterns

### Standard Module Structure
```rust
// Every module should have:
mod error;  // or errors in mod.rs
pub use error::{Error, Result};
```

### Conversion Pattern
```rust
// At module boundaries:
impl From<child::Error> for parent::Error {
    fn from(err: child::Error) -> Self {
        parent::Error::Child(err)
    }
}
```

## Testing Strategy
1. Run module tests after each change
2. Run integration tests after each phase
3. Full test suite before merging
```

### `/analysis/risk-assessment.md`
```markdown
# Risk Assessment

## Identified Risks

### High Risk
1. **Circular Dependencies**
   - Probability: Medium
   - Impact: High
   - Mitigation: Careful dependency analysis, incremental changes
   
2. **Breaking Public API**
   - Probability: Low
   - Impact: High  
   - Mitigation: Keep crate::Error for public API

### Medium Risk
1. **Test Failures**
   - Probability: High
   - Impact: Medium
   - Mitigation: Fix tests as we go, comprehensive testing

2. **Merge Conflicts**
   - Probability: Medium
   - Impact: Medium
   - Mitigation: Complete quickly, coordinate with team

### Low Risk
1. **Performance Impact**
   - Probability: Low
   - Impact: Low
   - Mitigation: Error handling not on hot path

## Rollback Plan
- Each phase in separate commit
- Can revert individual modules if needed
- Keep old error handling until fully migrated
```

## Success Criteria
- [ ] Clear migration order established
- [ ] Standard patterns documented
- [ ] Risk mitigation planned
- [ ] Detailed checklist created
- [ ] Timeline estimated

## Time Estimate
2 hours

## Dependencies
- A.0 (Error Usage Analysis) must be complete
- A.1 (Dependency Mapping) must be complete

## Notes
- Consider creating a small proof-of-concept first
- Plan for incremental merging
- Document decisions for future reference