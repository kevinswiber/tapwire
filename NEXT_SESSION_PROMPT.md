# Next Session: Phase 0 - Task F.4: Create Unified Event ID Generator

## Project Status Update

We are implementing SSE proxy integration with MCP message handling capabilities in Shadowcat. The unified tracker (`plans/proxy-sse-message-tracker.md`) coordinates this work across 7 phases.

**Total Project**: 118-138 hours  
**Current Phase**: Phase 0 - Foundation Components (Week 1)  
**Phase 0 Progress**: 9 hours completed of 11 hours total

### Completed Foundation Tasks (Phase 0)

1. **Transport Context Refactor** (✅ 2025-08-08) - 17.5 hours
   - MessageEnvelope system ready with TransportContext::Sse
   - All tests passing, clippy clean

2. **F.1: Protocol Version Manager** (✅ 2025-08-08) - 2 hours
   - Created `src/mcp/protocol.rs` with enum-based version management
   - Supports both MCP versions with capability detection
   - 22 comprehensive tests

3. **F.2: Minimal MCP Parser** (✅ 2025-08-08) - 4 hours
   - Created `src/mcp/early_parser.rs` with lightweight parser
   - Extracts message type, method names, and IDs
   - 37 comprehensive tests

4. **F.3: Batch Handler** (✅ 2025-08-08) - 3 hours
   - Created `src/mcp/batch.rs` with complete batch handling
   - Version-aware batching (only for 2025-03-26)
   - Message grouping, splitting, combining functionality
   - 18 comprehensive tests

5. **F.5: Message Context** (✅ From Refactor)
   - Already exists as `MessageContext` in `src/transport/envelope.rs`

## Current Task: F.4 - Create Unified Event ID Generator

### Objective
Create a unified event ID generator that produces IDs suitable for both SSE event streams and MCP message correlation. This will enable tracking messages across the entire proxy system.

### Essential Files to Review

1. **Primary Tracker**: `plans/proxy-sse-message-tracker.md` (see F.4 details)
2. **Task Details**: `plans/integration-tasks/foundation-tasks.md` (F.4 section)
3. **Existing Types**: 
   - `shadowcat/src/transport/envelope.rs` (MessageContext, MessageEnvelope)
   - `shadowcat/src/mcp/protocol.rs` (ProtocolVersion)
   - `shadowcat/src/mcp/early_parser.rs` (MinimalMessage types)

### Deliverables

1. **Create `src/mcp/event_id.rs`** with:
   - `UnifiedEventIdGenerator` struct with thread-safe counter
   - Method to generate event IDs with correlation info (`generate`)
   - Method to extract correlation from event IDs (`extract_correlation`)
   - Simple ID generation for non-correlated events (`generate_simple`)
   - `CorrelationInfo` struct to hold extracted data

2. **Key Features**:
   - Include session ID in event IDs for tracking
   - Include JSON-RPC ID for request-response correlation
   - Support notifications (no JSON-RPC ID)
   - Thread-safe counter using AtomicU64
   - Unique node ID to prevent collisions

3. **Comprehensive tests** demonstrating:
   - Event ID generation with various inputs
   - Correlation extraction from IDs
   - Thread safety with concurrent generation
   - Handling of notifications vs requests
   - Edge cases (missing fields, invalid formats)

4. **Update `src/mcp/mod.rs`** to export the event ID generator

### Implementation Structure

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

pub struct UnifiedEventIdGenerator {
    node_id: String,
    counter: AtomicU64,
}

impl UnifiedEventIdGenerator {
    pub fn new() -> Self {
        // Generate unique node ID
    }
    
    pub fn generate(
        &self,
        session_id: &str,
        json_rpc_id: Option<&serde_json::Value>,
    ) -> String {
        // Format: {session_id}-{node_id}-{json_rpc_id}-{counter}
        // For notifications: {session_id}-{node_id}-notif-{counter}
    }
    
    pub fn extract_correlation(&self, event_id: &str) -> Option<CorrelationInfo> {
        // Parse event ID back into components
    }
    
    pub fn generate_simple(&self) -> String {
        // Simple ID without correlation: {node_id}-{counter}
    }
}

pub struct CorrelationInfo {
    pub session_id: String,
    pub node_id: String,
    pub json_rpc_id: Option<String>,
    pub sequence: Option<u64>,
}
```

### Commands to Use

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Create the event ID generator file
touch src/mcp/event_id.rs

# Run tests as you implement
cargo test mcp::event_id

# Check formatting and clippy
cargo fmt
cargo clippy --all-targets -- -D warnings

# Run all tests to ensure nothing broke
cargo test
```

### Success Criteria

- [ ] `src/mcp/event_id.rs` created with full implementation
- [ ] Thread-safe ID generation using AtomicU64
- [ ] IDs include correlation information (session, JSON-RPC ID)
- [ ] Correlation can be extracted from generated IDs
- [ ] Support for both requests (with ID) and notifications (without ID)
- [ ] Comprehensive tests covering all scenarios
- [ ] No clippy warnings
- [ ] Code properly formatted
- [ ] Module exported in `src/mcp/mod.rs`

## Next Steps After F.4

Once F.4 is complete, Phase 0 will be finished! We'll move to **Phase 1: SSE Transport with MCP Awareness** (Week 1-2):

- **S.1**: Add SSE Transport CLI Option (2h)
- **S.2**: Create MCP-Aware SSE Transport Wrapper (4h) - Will use MessageEnvelope!
- **S.3**: Integrate with Forward Proxy (3h)
- **S.4**: Add MCP Parser Hooks to Transport (2h)

## Key Design Considerations

1. **Thread Safety**: Use atomic operations for counter increment
2. **Uniqueness**: Include node ID to prevent collisions across instances
3. **Correlation**: Embed enough info to track request-response pairs
4. **SSE Compatibility**: IDs must be valid SSE event IDs (no newlines)
5. **Performance**: Keep generation lightweight for high throughput

## Time Management

- **Estimated**: 2 hours
- **Suggested breakdown**:
  - 20 min: Module setup and structure
  - 60 min: Core implementation with thread safety
  - 30 min: Comprehensive testing
  - 10 min: Integration and cleanup

## Development Workflow Reminder

1. Create todo list with TodoWrite tool
2. Examine existing code patterns for consistency
3. Design the ID format carefully
4. Implement with tests alongside
5. Run `cargo fmt` after implementation
6. Run `cargo clippy --all-targets -- -D warnings`
7. Update tracker with completion

---

**Session Goal**: Complete the Unified Event ID Generator (F.4) with thread-safe implementation and comprehensive tests. This will complete Phase 0 and prepare us for Phase 1 SSE Transport implementation.