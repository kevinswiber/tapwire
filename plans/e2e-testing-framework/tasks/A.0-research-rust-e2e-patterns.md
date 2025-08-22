# Task A.0: Research Rust E2E Testing Patterns

## Objective
Survey how successful Rust projects implement end-to-end testing, particularly those dealing with:
- Multiple process coordination
- Network services
- CLI applications  
- Proxy/middleware systems

## Key Questions
1. How do major Rust projects structure their E2E tests?
2. What libraries and tools are commonly used?
3. How is process lifecycle managed?
4. How are logs captured and analyzed?
5. What patterns ensure deterministic tests?
6. How is test parallelization handled?

## Projects to Study

### Primary References
1. **Linkerd2-proxy** (Kubernetes service mesh proxy)
   - GitHub: `linkerd/linkerd2-proxy`
   - Focus: `tests/` directory structure
   - Key patterns: Traffic generation, proxy testing

2. **Vector** (Observability data pipeline)
   - GitHub: `vectordotdev/vector`
   - Focus: `tests/integration/` 
   - Key patterns: Component testing, event processing

3. **Cargo** (Rust package manager)
   - GitHub: `rust-lang/cargo`
   - Focus: `tests/testsuite/`
   - Key patterns: CLI testing, process spawning

4. **Tokio** (Async runtime)
   - GitHub: `tokio-rs/tokio`
   - Focus: `tests/` and process examples
   - Key patterns: Async process management

### Secondary References
- **Hyper** - HTTP library E2E tests
- **Actix-web** - Web framework integration tests
- **SQLx** - Database integration testing
- **Nushell** - Shell with complex CLI testing

## Research Areas

### 1. Test Organization
- Directory structure patterns
- Test categorization (unit/integration/e2e)
- Shared test utilities
- Fixture management

### 2. Process Management
```rust
// Common patterns to investigate:
- tokio::process::Command
- std::process::Command
- duct library
- assert_cmd crate
- How cleanup is handled
- Health check patterns
```

### 3. Port Management
```rust
// Strategies to evaluate:
- portpicker crate
- OS-assigned ports (bind to 0)
- Port ranges for CI
- Collision avoidance
```

### 4. Log Capture
```rust
// Approaches to study:
- tracing-subscriber with custom layer
- env_logger with capture
- test-log crate
- Custom stdout/stderr capture
```

### 5. Test Helpers
```rust
// Common utilities:
- Test builders
- Assertion helpers
- Timing utilities
- Cleanup guards
```

## Deliverables

### Research Document Structure
```markdown
# Rust E2E Testing Patterns Research

## Executive Summary
- Key findings
- Recommended approach for Shadowcat

## Project Analysis

### Linkerd2-proxy
- Structure: [describe]
- Key patterns: [list]
- Relevant techniques: [list]

### Vector
- Structure: [describe]
- Key patterns: [list]
- Relevant techniques: [list]

[Continue for each project...]

## Common Patterns

### Process Management
- Pattern 1: [describe with code example]
- Pattern 2: [describe with code example]

### Port Allocation
- Pattern 1: [describe with code example]
- Pattern 2: [describe with code example]

### Log Capture
- Pattern 1: [describe with code example]
- Pattern 2: [describe with code example]

## Recommended Libraries
| Library | Purpose | Pros | Cons |
|---------|---------|------|------|
| assert_cmd | CLI testing | ... | ... |
| portpicker | Port allocation | ... | ... |
| [etc...] | | | |

## Best Practices
1. [Practice with rationale]
2. [Practice with rationale]

## Anti-patterns to Avoid
1. [Anti-pattern with explanation]
2. [Anti-pattern with explanation]

## Recommendations for Shadowcat
- Test structure: [specific recommendation]
- Process management: [specific recommendation]
- Port allocation: [specific recommendation]
- Log capture: [specific recommendation]
```

## Research Method
1. Clone repositories
2. Analyze test directory structure
3. Identify common patterns
4. Extract code examples
5. Evaluate pros/cons
6. Document findings

## Time Allocation
- 30 min: Initial repository survey
- 30 min: Linkerd2-proxy deep dive
- 30 min: Vector deep dive
- 30 min: Cargo/Tokio analysis
- 30 min: Pattern synthesis
- 30 min: Documentation

## Success Criteria
- [ ] Analyzed at least 4 major Rust projects
- [ ] Identified 3+ process management patterns
- [ ] Identified 2+ port allocation strategies
- [ ] Identified 2+ log capture approaches
- [ ] Clear recommendation for Shadowcat
- [ ] Code examples for each pattern