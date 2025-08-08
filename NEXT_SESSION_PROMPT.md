# Next Session: Phase 0 - Task F.3: Implement Batch Handler

## Context

We are implementing SSE proxy integration with MCP message handling capabilities in Shadowcat. The unified tracker (`plans/proxy-sse-message-tracker.md`) coordinates this work across 7 phases with 120-140 hours of effort.

### Current Status
- **Phase**: Phase 0 - Foundation Components (Week 1)
- **Task**: F.3 - Implement Batch Handler
- **Duration**: 3 hours
- **Dependencies**: F.1 (✅ Completed), F.2 (✅ Completed)

### What Has Been Completed
- **F.1: Protocol Version Manager** (✅ Completed 2025-08-08)
  - Created `src/mcp/protocol.rs` with type-safe enum-based version management
  - Supports both MCP versions (2025-03-26 with batching, 2025-06-18 without)
  - Includes version negotiation, capability detection, and conversion traits
  - All tests passing, no clippy warnings

- **F.2: Build Minimal MCP Parser** (✅ Completed 2025-08-08)
  - Created `src/mcp/early_parser.rs` with lightweight MCP message parser
  - Extracts message type (Request/Response/Notification), method names, and IDs
  - Handles batch message parsing for 2025-03-26 version only
  - Validates JSON-RPC 2.0 format
  - Returns `ParsedInfo` with message count and individual messages
  - 16 comprehensive tests, all passing

## Objective

Implement a Batch Handler that provides shared logic for handling MCP batch messages. This component will handle splitting batch messages into individual messages and combining responses into batches when needed, specifically for the 2025-03-26 protocol version which supports batching.

## Essential Context Files to Read

1. **Primary Tracker**: `plans/proxy-sse-message-tracker.md`
2. **Task Details**: `plans/integration-tasks/foundation-tasks.md` (Task F.3 section)
3. **Protocol Version Manager**: `shadowcat/src/mcp/protocol.rs` (completed in F.1)
4. **Minimal Parser**: `shadowcat/src/mcp/early_parser.rs` (completed in F.2)
5. **Existing Transport**: `shadowcat/src/transport/mod.rs` (understand TransportMessage)

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Task Details

### Deliverables
1. Create `src/mcp/batch.rs` with:
   - `BatchHandler` struct that uses `ProtocolVersion` from F.1
   - Method to check if batching should be used (`should_batch`)
   - Method to split batch JSON into individual messages (`split_if_batch`)
   - Method to combine messages into batch format (`combine_if_needed`)
   - Method to group messages by type (`group_by_type`)
   - `GroupedMessages` struct to organize messages by type

2. Create comprehensive tests demonstrating:
   - Batch detection based on protocol version
   - Splitting batch messages into individual messages
   - Combining multiple messages into batches
   - Grouping messages by type (Request/Response/Notification)
   - Handling edge cases (empty arrays, single messages)
   - Version-specific behavior (no batching for 2025-06-18)

3. Update `src/mcp/mod.rs` to export the new batch handler

### Implementation Strategy

#### Phase 1: Module Setup (30 min)
1. Create `src/mcp/batch.rs` file
2. Import necessary dependencies (serde_json::Value, ProtocolVersion, MinimalMessage, MessageType)
3. Define core types: `BatchHandler`, `GroupedMessages`
4. Update `src/mcp/mod.rs` to include and export the batch handler

#### Phase 2: Core Implementation (1.5 hours)
1. Implement `BatchHandler::new(version: ProtocolVersion)`
2. Implement `should_batch()` to check if batching is appropriate
3. Implement `split_if_batch()` to handle batch splitting
4. Implement `combine_if_needed()` to create batches when needed
5. Implement `group_by_type()` to organize messages
6. Create `GroupedMessages` struct with utility methods

#### Phase 3: Testing (45 min)
1. Test batch splitting for 2025-03-26 version
2. Test that 2025-06-18 doesn't batch messages
3. Test combining messages into batches
4. Test message grouping by type
5. Test edge cases (empty inputs, single messages)
6. Test helper methods on GroupedMessages

#### Phase 4: Integration (15 min)
1. Ensure compatibility with MinimalMessage from early_parser
2. Verify the handler can work with TransportMessage conversions
3. Run `cargo fmt` to format the code
4. Run `cargo clippy --all-targets -- -D warnings`
5. Run all tests to ensure nothing broke
6. Update tracker with completion status

## Commands to Use

```bash
# Create the new batch handler file
touch src/mcp/batch.rs

# Run tests as you implement
cargo test mcp::batch

# Check specific test output
cargo test mcp::batch -- --nocapture

# Format your code
cargo fmt

# Check for issues
cargo clippy --all-targets -- -D warnings

# Run all tests to ensure nothing broke
cargo test
```

## Success Criteria Checklist

- [ ] `src/mcp/batch.rs` created with full implementation
- [ ] `BatchHandler` uses `ProtocolVersion` from F.1
- [ ] Correctly splits batch messages for 2025-03-26 version
- [ ] Properly combines messages into batches when appropriate
- [ ] Groups messages by type (Request/Response/Notification)
- [ ] Respects version-specific batching support
- [ ] `GroupedMessages` provides useful utility methods
- [ ] Comprehensive error handling
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code is properly formatted
- [ ] Module exported in `src/mcp/mod.rs`
- [ ] Tracker updated with completion status

## Important Notes

- **Always use TodoWrite tool** to track your progress through the task
- **Start with examining existing code** to understand current architecture
- **Follow established patterns** from previous implementations
- **Test incrementally** as you build each component
- **Run `cargo fmt`** after implementing new functionality
- **Run `cargo clippy --all-targets -- -D warnings`** before any commit
- **Update the refactor tracker** when the task is complete
- **Focus on the current phase objectives**

## Key Design Considerations

1. **Version-Aware Batching**: Only the 2025-03-26 version supports batching. The handler must respect this constraint.

2. **Integration with Parser**: The batch handler works with `MinimalMessage` from the early parser, maintaining consistency.

3. **Bidirectional Processing**: The handler supports both:
   - Splitting incoming batch messages for processing
   - Combining outgoing messages into batches for transmission

4. **Performance**: Keep operations lightweight as this will be called frequently in the message pipeline.

5. **Type Safety**: Use the MessageType enum from early_parser to ensure type safety when grouping messages.

## Model Usage Guidelines

- **IMPORTANT** Be mindful of model capabilities. Assess whether Claude Opus or Claude Sonnet would be best for each step. When there's a benefit to a model change, pause and recommend it. Be mindful of the context window. When the context window has less than 15% availability, suggest creating a new Claude session and output a good prompt, referencing all available plans, tasks, and completion files that are relevant. Save the prompt into NEXT_SESSION_PROMPT.md.

## Development Workflow

1. Create todo list with TodoWrite tool to track progress
2. Examine existing codebase architecture and established patterns
3. Study current implementations related to the task
4. Design the solution approach and identify key components
5. Implement functionality incrementally with frequent testing
6. Add comprehensive error handling following project patterns
7. Create tests demonstrating functionality works correctly
8. Run tests after each significant change to catch issues early
9. Run `cargo fmt` to ensure consistent code formatting
10. Run `cargo clippy -- -D warnings` to catch potential issues
11. Update project documentation and tracker as needed
12. Commit changes with clear, descriptive messages

## Example Implementation Structure

Based on the task details in `plans/integration-tasks/foundation-tasks.md`, the batch handler should follow this structure:

```rust
use serde_json::Value;
use crate::mcp::protocol::ProtocolVersion;
use crate::mcp::early_parser::{MinimalMessage, MessageType};

pub struct BatchHandler {
    version: ProtocolVersion,
}

impl BatchHandler {
    pub fn new(version: ProtocolVersion) -> Self {
        Self { version }
    }
    
    pub fn should_batch(&self, messages: &[MinimalMessage]) -> bool {
        self.version.supports_batching() && messages.len() > 1
    }
    
    pub fn split_if_batch(&self, value: Value) -> Vec<Value> {
        match value {
            Value::Array(arr) if self.version.supports_batching() => arr,
            single => vec![single],
        }
    }
    
    pub fn combine_if_needed(&self, messages: Vec<Value>) -> Value {
        if self.should_batch_values(&messages) {
            Value::Array(messages)
        } else {
            messages.into_iter().next().unwrap_or(Value::Null)
        }
    }
    
    fn should_batch_values(&self, messages: &[Value]) -> bool {
        self.version.supports_batching() && messages.len() > 1
    }
    
    pub fn group_by_type(&self, messages: Vec<MinimalMessage>) -> GroupedMessages {
        // Implementation here
    }
}

pub struct GroupedMessages {
    pub requests: Vec<MinimalMessage>,
    pub responses: Vec<MinimalMessage>,
    pub notifications: Vec<MinimalMessage>,
}
```

## Next Steps After This Task

Once F.3 is complete, the remaining Phase 0 tasks are:
- **F.4**: Create Unified Event ID Generator (2 hours, no dependencies)
- **F.5**: Build Message Context Structure (2 hours, depends on F.1)

After completing Phase 0, we'll move to Phase 1 (SSE Transport with MCP Awareness).

## Related Context

- The batch handler is essential for:
  - SSE transport when handling 2025-03-26 protocol messages
  - Reverse proxy when processing batch requests
  - Interceptors that need to process message groups
  - Recorders that need to maintain batch integrity

- Key integration points:
  - Must work seamlessly with `MinimalMcpParser` from F.2
  - Will be used by SSE transport (S.2) for batch processing
  - Required by reverse proxy (R.1) for batch request handling
  - Foundation for correlation engine (M.4) batch support

---

**Session Goal**: Complete the Batch Handler implementation with comprehensive tests and documentation, ensuring proper handling of MCP batch messages for the 2025-03-26 protocol version while maintaining clean separation of concerns.