# Next Session: Continue Phase 3 - Schema Validation and Correlation

## Project Status Update

We are implementing SSE proxy integration with MCP message handling in Shadowcat, following the unified tracker at `plans/proxy-sse-message-tracker.md`.

**Current Status**: Phase 3 in progress  
**Phase 0**: 100% Complete ✅ (F.1-F.5 all done)  
**Phase 1**: 100% Complete ✅ (S.1-S.4 all done)  
**Phase 2**: 100% Complete ✅ (R.1-R.4 all done)  
**Phase 3**: 31% Complete (M.1 ✅, M.2 ✅, M.3-M.5 pending)

## Accomplishments Previous Session

### Phase 3: Full MCP Parser ✅

Successfully implemented P.1 (Create Full MCP Parser):
- Created `src/mcp/parser.rs` with comprehensive MCP message support
- Implemented `McpMessage` enum with Request, Response, and Notification variants
- Added `McpParser` with support for both protocol versions (2025-03-26 and 2025-06-18)
- Implemented batch message handling for 2025-03-26
- Created `MessageMetadata` for correlation tracking
- Added helper functions for streaming detection, session extraction, and correlation
- Wrote 22+ comprehensive unit tests covering all edge cases
- All tests passing, no clippy warnings

The parser provides:
- Full JSON-RPC 2.0 compliance
- Protocol version awareness
- Batch message support (2025-03-26 only)
- Error response handling with standard error codes
- Metadata extraction for correlation
- Helper functions for common operations

## Phase 3: Full MCP Parser and Correlation (Week 3)

Remaining tasks for Phase 3:

| ID | Task | Duration | Status |
|----|------|----------|--------|
| P.2 | **Add Schema Validation** | 4h | ⬜ Not Started |
| P.3 | Implement Correlation Store | 5h | ⬜ Not Started |
| P.4 | Add Request/Response Matching | 4h | ⬜ Not Started |
| P.5 | **Integrate with Proxy** | 5h | ⬜ Not Started |

## Primary Task: P.2 - Add Schema Validation

### Objective
Add schema validation to ensure MCP messages conform to the specification:
1. Validate method names against known MCP methods
2. Validate parameter schemas for each method
3. Validate response schemas
4. Handle protocol version differences in schemas

### Implementation Plan

1. **Create `src/mcp/schema.rs`**
   - Define method schemas for all MCP methods
   - Implement validation logic
   - Handle version-specific differences
   - Provide detailed validation errors

2. **MCP Methods to Validate**
   ```rust
   // Core methods
   - initialize: params { protocolVersion, capabilities, clientInfo }
   - initialized: params { }
   - ping: params { }
   - error: notification with error details
   
   // Tool methods
   - tools/list: params { }
   - tools/call: params { name, arguments }
   
   // Prompt methods
   - prompts/list: params { }
   - prompts/get: params { name }
   
   // Resource methods  
   - resources/list: params { }
   - resources/read: params { uri }
   - resources/subscribe: params { uri }
   - resources/unsubscribe: params { uri }
   
   // Completion methods
   - completion/complete: params { ref, argument }
   
   // Logging methods
   - logging/setLevel: params { level }
   ```

3. **Validation Interface**
   ```rust
   pub struct SchemaValidator {
       version: ProtocolVersion,
   }
   
   impl SchemaValidator {
       pub fn validate(&self, msg: &McpMessage) -> Result<(), ValidationError>;
       pub fn validate_method(&self, method: &str) -> bool;
       pub fn validate_params(&self, method: &str, params: &Value) -> Result<(), ValidationError>;
   }
   ```

4. **Integration Points**
   - Integrate with `McpParser` for optional validation
   - Use in interceptors for method-based filtering
   - Apply in correlation for semantic matching

### Success Criteria

1. ✅ Schema definitions for all MCP methods
2. ✅ Version-aware validation logic
3. ✅ Detailed validation error messages
4. ✅ 15+ unit tests for schema validation
5. ✅ Integration with parser (optional validation)
6. ✅ Documentation with examples

## Secondary Task: P.3 - Implement Correlation Store

If time permits, begin work on the correlation store:
- Track request/response pairs by ID
- Handle timeout and cleanup
- Support concurrent correlations
- Provide correlation statistics

## Commands for Development

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Create the schema module
touch src/mcp/schema.rs

# Run tests as you develop
cargo test mcp::schema

# Check for warnings
cargo clippy --all-targets -- -D warnings

# Run all tests
cargo test
```

## Key Context

### MCP Method Categories
1. **Lifecycle**: initialize, initialized, ping
2. **Tools**: list, call
3. **Prompts**: list, get
4. **Resources**: list, read, subscribe, unsubscribe
5. **Completion**: complete
6. **Logging**: setLevel
7. **Notifications**: error, progress, etc.

### Schema Validation Requirements
- Required fields must be present
- Field types must match specification
- Extra fields are allowed (forward compatibility)
- Version-specific validation (e.g., capabilities differ)
- Clear error messages for debugging

### Available Foundation
From P.1 implementation:
- `McpMessage` enum with all message types
- `McpParser` for parsing messages
- `MessageMetadata` for tracking
- Helper functions for common operations
- Protocol version management

## Notes

- Schema validation is critical for interceptor rules and security
- Focus on the most common methods first
- Consider using a schema definition format (JSON Schema or custom)
- Keep performance in mind - validation should be fast
- This will enable semantic interceptor rules in Phase 4

---

**Next Goal**: Implement P.2 (Schema Validation) with comprehensive method validation and testing.