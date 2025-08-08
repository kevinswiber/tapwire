# Next Session: Phase 0 - Task F.3: Implement Batch Handler

## ✅ Transport Context Refactor Complete!

The Transport Context Refactor has been successfully completed (17.5 hours, 71% faster than estimate), providing:
- `MessageEnvelope`: Complete message with context wrapper
- `MessageContext`: Session ID, direction, transport metadata (F.5 already exists!)
- `ProtocolMessage`: Replaces TransportMessage
- `TransportContext::Sse`: Has all SSE metadata fields ready
- See `SSE_INTEGRATION_UPDATES.md` and `PROXY_SSE_TRACKER_UPDATES.md` for details

## Current Project: SSE Proxy Integration with MCP

We are implementing SSE proxy integration with MCP message handling capabilities in Shadowcat. The unified tracker (`plans/proxy-sse-message-tracker.md`) coordinates this work across 7 phases.

**Total Project**: 118-138 hours (reduced from 120-140 due to refactor benefits)  
**Current Phase**: Phase 0 - Foundation Components (Week 1)  
**Phase 0 Total**: 11 hours (reduced from 13 - F.5 already exists)

### Current Task: F.3 - Implement Batch Handler

**Duration**: 3 hours  
**Dependencies**: F.1 (✅ Completed), F.2 (✅ Completed)  
**Status**: Ready to start

### What Has Been Completed

1. **Transport Context Refactor** (✅ 2025-08-08)
   - MessageEnvelope system ready
   - All tests passing, clippy clean
   - Saved ~9 hours of SSE integration work

2. **F.1: Protocol Version Manager** (✅ 2025-08-08)
   - Created `src/mcp/protocol.rs` with enum-based version management
   - Supports both MCP versions (2025-03-26 with batching, 2025-06-18 without)
   - Version negotiation and capability detection

3. **F.2: Minimal MCP Parser** (✅ 2025-08-08)
   - Created `src/mcp/early_parser.rs` with lightweight parser
   - Extracts message type, method names, and IDs
   - Handles batch message parsing for 2025-03-26
   - 16 comprehensive tests passing

4. **F.5: Message Context** (✅ From Refactor)
   - Already exists as `MessageContext` in `src/transport/envelope.rs`
   - No additional work needed

## Task F.3: Batch Handler Implementation

### Objective
Implement a Batch Handler that provides shared logic for handling MCP batch messages, specifically for the 2025-03-26 protocol version which supports batching.

### Essential Files to Review

1. **Primary Tracker**: `plans/proxy-sse-message-tracker.md` (updated with refactor notes)
2. **Task Details**: `plans/integration-tasks/foundation-tasks.md` (Task F.3 section)
3. **Protocol Version Manager**: `shadowcat/src/mcp/protocol.rs` (F.1 complete)
4. **Minimal Parser**: `shadowcat/src/mcp/early_parser.rs` (F.2 complete)
5. **Transport Types**: `shadowcat/src/transport/mod.rs` (note: ProtocolMessage)
6. **Envelope System**: `shadowcat/src/transport/envelope.rs` (MessageEnvelope, MessageContext)

### Deliverables

1. **Create `src/mcp/batch.rs`** with:
   - `BatchHandler` struct using `ProtocolVersion` from F.1
   - Method to check if batching should be used (`should_batch`)
   - Method to split batch JSON into individual messages (`split_if_batch`)
   - Method to combine messages into batch format (`combine_if_needed`)
   - Method to group messages by type (`group_by_type`)
   - `GroupedMessages` struct to organize messages

2. **Comprehensive tests** demonstrating:
   - Batch detection based on protocol version
   - Splitting/combining batch messages
   - Grouping by type (Request/Response/Notification)
   - Edge cases (empty arrays, single messages)
   - Version-specific behavior (no batching for 2025-06-18)

3. **Update `src/mcp/mod.rs`** to export the batch handler

### Implementation Notes

⚠️ **Important Type Changes from Refactor:**
- Use `ProtocolMessage` instead of `TransportMessage`
- Consider how batches interact with `MessageEnvelope` when used in transports
- The batch handler works at the protocol level, before wrapping in envelopes

### Example Structure

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
    
    // ... other methods
}
```

### Commands to Use

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Create the batch handler file
touch src/mcp/batch.rs

# Run tests as you implement
cargo test mcp::batch

# Check formatting and clippy
cargo fmt
cargo clippy --all-targets -- -D warnings

# Run all tests to ensure nothing broke
cargo test
```

### Success Criteria

- [ ] `src/mcp/batch.rs` created with full implementation
- [ ] Uses `ProtocolVersion` from F.1 correctly
- [ ] Correctly handles batching for 2025-03-26 version only
- [ ] Groups messages by type appropriately
- [ ] Comprehensive error handling
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code properly formatted
- [ ] Module exported in `src/mcp/mod.rs`

## Next Steps After F.3

Once F.3 is complete, the remaining Phase 0 task is:
- **F.4**: Create Unified Event ID Generator (2 hours, no dependencies)

Then we move to **Phase 1: SSE Transport with MCP Awareness** (Week 1-2):
- S.1: Add SSE Transport CLI Option (2h)
- S.2: Create MCP-Aware SSE Transport Wrapper (4h) - Will use MessageEnvelope!
- S.3: Integrate with Forward Proxy (3h)
- S.4: Add MCP Parser Hooks to Transport (2h)

## Key Design Considerations

1. **Version-Aware Batching**: Only 2025-03-26 supports batching
2. **Integration with Parser**: Works with `MinimalMessage` from F.2
3. **Bidirectional Processing**: Support both splitting and combining
4. **Performance**: Keep lightweight for frequent calls
5. **Type Safety**: Use MessageType enum from early_parser

## Development Workflow Reminder

1. Create todo list with TodoWrite tool
2. Examine existing code patterns
3. Design solution approach
4. Implement incrementally with tests
5. Run `cargo fmt` after implementation
6. Run `cargo clippy --all-targets -- -D warnings`
7. Update tracker with completion

## Time Management

- **Estimated**: 3 hours
- **Suggested breakdown**:
  - 30 min: Module setup and structure
  - 90 min: Core implementation
  - 45 min: Comprehensive testing
  - 15 min: Integration and cleanup

---

**Session Goal**: Complete the Batch Handler (F.3) with comprehensive tests, preparing for the Event ID Generator (F.4) and then moving into Phase 1 SSE Transport implementation that leverages our new MessageEnvelope system.