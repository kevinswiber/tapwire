# Task A.0: Code Analysis

## Objective
Perform a complete analysis of the current `src/proxy/reverse.rs` implementation to understand its structure, dependencies, and pain points before refactoring.

## Key Questions
1. What are all the public functions and their callers?
2. How is state shared between different parts of the code?
3. What are the main execution paths for JSON vs SSE?
4. Where are the synchronization points and potential bottlenecks?
5. What patterns are duplicated that could be extracted?

## Process

### Step 1: Function Inventory (30 min)
- [ ] List all public functions with signatures
- [ ] List all private functions with signatures  
- [ ] Map function dependencies (who calls whom)
- [ ] Identify entry points and exit points

### Step 2: State Analysis (30 min)
- [ ] Document all shared state (AppState, SessionManager, etc.)
- [ ] Identify all Arc/Mutex usage
- [ ] Map data flow through the system
- [ ] Document concurrent access patterns

### Step 3: Code Metrics (15 min)
- [ ] Count lines per logical section
- [ ] Identify largest functions
- [ ] Count external dependencies
- [ ] Measure cyclomatic complexity of key functions

### Step 4: Pattern Identification (30 min)
- [ ] Find duplicated code blocks
- [ ] Identify common error handling patterns
- [ ] Document interceptor integration points
- [ ] List all uses of tokio::spawn

### Step 5: Problem Areas (15 min)
- [ ] Document known bugs (SSE streaming)
- [ ] Identify technical debt
- [ ] List TODO/FIXME comments
- [ ] Note performance concerns

## Commands to Run
```bash
# Get function list
rg "^(pub |async |fn )" src/proxy/reverse.rs

# Count lines by section
wc -l src/proxy/reverse.rs

# Find TODO/FIXME
rg "TODO|FIXME" src/proxy/reverse.rs

# Find duplicated strings (potential duplicate logic)
rg -o '"[^"]{20,}"' src/proxy/reverse.rs | sort | uniq -d

# Find all struct definitions
rg "^(pub )?struct" src/proxy/reverse.rs

# Find all enum definitions  
rg "^(pub )?enum" src/proxy/reverse.rs

# Check for large functions (>50 lines)
awk '/^(async )?fn / {name=$0; count=0} {count++} /^}$/ {if(count>50) print name " - " count " lines"}' src/proxy/reverse.rs
```

## Deliverables

### `/analysis/current-architecture.md`
Structure:
```markdown
# Current Architecture Analysis

## Module Structure
- Public Interface
  - Functions
  - Types
  - Traits

## Internal Organization
- Core Logic (lines X-Y)
- Handler Functions (lines X-Y)
- Helper Functions (lines X-Y)

## Dependencies
- External crates
- Internal modules
- Shared resources

## Execution Flows
- JSON Request Path
- SSE Request Path
- Error Paths
```

### `/analysis/dependencies.md`
Structure:
```markdown
# Dependency Analysis

## External Dependencies
- reqwest - HTTP client
- axum - Web framework
- tokio - Async runtime
- [complete list...]

## Internal Dependencies
- session manager
- interceptor chain
- transport layer
- [complete list...]

## Coupling Points
- Tight coupling areas
- Loose coupling opportunities
```

### `/analysis/state-management.md`
Structure:
```markdown
# State Management Analysis

## Shared State
- AppState structure
- SessionManager
- ConnectionPools
- [complete list...]

## Synchronization
- Mutex usage
- RwLock usage
- Channel usage

## Concurrency Patterns
- Task spawning
- Future joining
- Stream processing
```

## Success Criteria
- [ ] Complete function map with dependencies
- [ ] All shared state documented
- [ ] Clear understanding of JSON vs SSE paths
- [ ] Identified refactoring opportunities
- [ ] Documented problem areas
- [ ] Analysis documents created

## Time Estimate
2 hours

## Notes
- Focus on understanding, not judging the current implementation
- Document both good patterns and problematic ones
- Pay special attention to SSE handling code
- Note any undocumented assumptions or invariants