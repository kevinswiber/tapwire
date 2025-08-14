# Task A.1: Inventory Current Batch-Related Code

## Objective
Create a comprehensive inventory of all batch-related code in the Shadowcat codebase to understand the current state and identify all locations that need modification.

## Key Areas to Investigate

### 1. Explicit Batch Handling
- Code that explicitly handles or rejects batches
- Error messages mentioning batches
- Batch-specific data structures or functions

### 2. Array Message Processing
- Code that checks for JSON arrays
- Array rejection logic
- Array parsing attempts

### 3. Protocol Layer
- Serialization/deserialization of arrays
- Message validation for arrays
- Protocol negotiation mentioning batches

### 4. Test Coverage
- Existing batch-related tests
- Tests that explicitly check batch rejection
- Protocol tests with batch scenarios

## Process

### 1. Search for Batch References
```bash
# Find all batch-related code
rg -i "batch" src/ --type rust

# Find array checking in JSON processing
rg "is_array|as_array" src/ --type rust

# Find array rejection patterns
rg "Array.*not.*supported|not.*support.*array" src/ --type rust -i
```

### 2. Analyze Key Files
Review and document the following files:
- `src/interceptor/batch_handler.rs` - Main batch handler
- `src/proxy/handlers.rs` - Proxy-level batch handling
- `src/transport/protocol/mod.rs` - New batch support
- All transport implementations (stdio, http, sse)

### 3. Trace Message Flow
- Document how messages flow through the system
- Identify all points where batches could be processed
- Note where batches are currently blocked

### 4. Test Analysis
- List all batch-related tests
- Identify test coverage gaps
- Note tests that would need updates

## Expected Deliverables

Create `analysis/batch-code-inventory.md` with:

### Current State Section
- List of all files with batch-related code
- Code snippets showing batch handling
- Current behavior at each point

### Impact Analysis Section
- Components that would need modification
- Estimated complexity for each component
- Dependencies between components

### Test Coverage Section
- Existing batch tests
- Tests that reject batches
- Coverage gaps

## Success Criteria
- [ ] All batch-related code identified
- [ ] Clear understanding of current behavior
- [ ] Impact assessment completed
- [ ] Test coverage mapped

## Commands to Run
```bash
# Generate file list with batch references
rg -l "batch" src/ > /tmp/batch-files.txt

# Count batch-related lines
rg -c "batch" src/ --type rust

# Find TODOs related to batches
rg "TODO.*batch|FIXME.*batch" src/ -i

# Check for batch in comments
rg "//.*batch|/\*.*batch" src/ -i
```

## Time Estimate
3 hours

## Dependencies
None

## Notes
- Be thorough - missing code could cause bugs later
- Document the "why" behind current batch rejection
- Look for implicit assumptions about single messages
- Consider session management implications