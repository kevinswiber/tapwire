# Next Session: Phase 1 - SSE Transport Implementation

## Project Status Update

We are implementing SSE proxy integration with MCP message handling in Shadowcat. Phase 0 (Foundation Components) is **100% complete** with all optimizations applied.

**Current Status**: Ready to begin Phase 1 - SSE Transport with MCP Awareness  
**Phase 1 Duration**: 11 hours total (2-3 sessions recommended)

## Recent Accomplishments (Phase 0 Complete ✅)

### Event ID Generator Enhancements
Just completed performance optimizations based on code review:
- ✅ Relaxed memory ordering for better performance
- ✅ Pre-allocated strings to reduce allocation overhead  
- ✅ Conditional newline replacement
- ✅ Performance documentation added
- ✅ All tests passing, no clippy warnings

**Performance**: Handles 5,000+ IDs/second, ~0.1-0.2ms per ID (well within 5% overhead target)

### Foundation Components Ready
1. **Protocol Version Manager** (`src/mcp/protocol.rs`) - 22 tests
2. **Minimal MCP Parser** (`src/mcp/early_parser.rs`) - 37 tests  
3. **Batch Handler** (`src/mcp/batch.rs`) - 18 tests
4. **Event ID Generator** (`src/mcp/event_id.rs`) - 17 tests, optimized
5. **Message Context** (`src/transport/envelope.rs`) - From refactor

## Phase 1 Implementation Plan

### Why Event IDs Matter for SSE Transport

The Event ID Generator we just optimized is critical for SSE because:
1. **SSE Protocol**: Requires unique IDs for reconnection/deduplication
2. **MCP Correlation**: Embeds request-response matching in the ID
3. **Session Tracking**: Maintains context across transport types

Example usage in SSE transport:
```rust
// Generate SSE event with correlation
let event_id = generator.generate(&session_id, msg.get_json_rpc_id());
// Result: "session-abc-f3d2a1bc-req-123-42"

// SSE format
id: session-abc-f3d2a1bc-req-123-42
event: message  
data: {"jsonrpc":"2.0","id":"req-123","method":"tools/list"}
```

### Task Breakdown

| ID | Task | Duration | Priority | Dependencies |
|----|------|----------|----------|--------------|
| **S.1** | Add SSE Transport CLI Option | 2h | 🔴 High | None |
| **S.2** | Create MCP-Aware SSE Transport Wrapper | 4h | 🔴 High | S.1, Foundation |
| **S.3** | Integrate with Forward Proxy | 3h | 🟡 Medium | S.2 |
| **S.4** | Add MCP Parser Hooks to Transport | 2h | 🟡 Medium | S.2 |

### Recommended Session Focus

**This Session (3-4 hours)**: Complete S.1 + Start S.2
- Add CLI option for SSE transport
- Begin SSE transport wrapper implementation
- Focus on Transport trait implementation

## Task S.1: Add SSE Transport CLI Option (2h)

### Implementation Steps

1. **Update CLI in `src/cli/mod.rs`**:
```rust
#[derive(Debug, Clone, ValueEnum)]
pub enum TransportType {
    Stdio,
    Sse,  // New
}

#[derive(Debug, Args)]
pub struct ForwardCommand {
    /// Transport type to use
    #[arg(long, value_enum, default_value = "stdio")]
    pub transport: TransportType,
    
    /// SSE endpoint URL (for SSE transport)
    #[arg(long, required_if_eq("transport", "sse"))]
    pub sse_url: Option<String>,
    
    // ... existing fields
}
```

2. **Update configuration in `src/config/mod.rs`**:
```rust
#[derive(Debug, Clone)]
pub struct ForwardConfig {
    pub transport: TransportType,
    pub sse_config: Option<SseConfig>,
    // ... existing fields
}

#[derive(Debug, Clone)]
pub struct SseConfig {
    pub endpoint_url: String,
    pub retry_interval_ms: u64,
    pub max_retries: u32,
}
```

3. **Tests to add**:
- CLI parsing with `--transport sse --sse-url http://localhost:8080`
- Configuration validation
- Help text verification

## Task S.2: Create SSE Transport Wrapper (4h)

### Key Implementation Points

```rust
// src/transport/sse_transport.rs
use crate::mcp::event_id::UnifiedEventIdGenerator;
use crate::mcp::early_parser::MinimalMcpParser;
use crate::transport::{Transport, MessageEnvelope, TransportContext};

pub struct SseTransport {
    url: Url,
    client: reqwest::Client,
    event_id_generator: UnifiedEventIdGenerator,
    parser: MinimalMcpParser,
    session_id: SessionId,
    event_source: Option<EventSource>,  // For receiving
}

impl Transport for SseTransport {
    async fn send(&mut self, envelope: MessageEnvelope) -> Result<()> {
        // 1. Extract JSON-RPC ID for correlation
        let json_rpc_id = self.parser.extract_id(&envelope.message)?;
        
        // 2. Generate event ID with correlation
        let event_id = self.event_id_generator.generate(
            &self.session_id,
            json_rpc_id.as_ref()
        );
        
        // 3. Format as SSE event
        let sse_data = format!(
            "id: {}\nevent: message\ndata: {}\n\n",
            event_id,
            serde_json::to_string(&envelope.message)?
        );
        
        // 4. POST to SSE endpoint
        self.client.post(&self.url)
            .header("Content-Type", "text/event-stream")
            .body(sse_data)
            .send()
            .await?;
            
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<MessageEnvelope> {
        // 1. Get next SSE event
        let event = self.event_source.next().await?;
        
        // 2. Extract correlation from event ID
        let correlation = self.event_id_generator
            .extract_correlation(&event.id)?;
        
        // 3. Parse message with MinimalMcpParser
        let parsed = self.parser.parse(&event.data)?;
        
        // 4. Build MessageEnvelope with SSE context
        Ok(MessageEnvelope {
            message: parsed.into_protocol_message(),
            context: MessageContext {
                session_id: correlation.session_id,
                direction: MessageDirection::ServerToClient,
                transport_metadata: TransportContext::Sse {
                    event_id: Some(event.id),
                    event_type: Some(event.event_type),
                    retry_ms: None,
                    headers: HashMap::new(),
                },
                timestamp: SystemTime::now(),
            },
        })
    }
}
```

### Integration with Foundation Components

1. **Event ID Generator**: Generate and extract correlation
2. **Minimal Parser**: Early message inspection
3. **Protocol Version**: Track in transport metadata
4. **Message Envelope**: Wrap all messages properly
5. **Batch Handler**: Detect and handle batches (if version supports)

## Important Architecture Notes

### SSE Transport Flow
```
Client Request → Forward Proxy → SSE Transport → HTTP POST → Server
                                      ↓
                              Generate Event ID
                              (with correlation)
                                      ↓
Server Response ← SSE Transport ← EventSource ← SSE Stream
                        ↓
                Extract Correlation
                  Build Envelope
```

### Correlation in Phase 1 vs Phase 3

**Phase 1 (Now)**: Basic correlation embedding
- Event IDs contain correlation info
- No active tracking of pending requests
- No timeout handling yet

**Phase 3 (Future)**: Full correlation engine
- Track pending requests with timeouts
- Match responses to requests
- Handle orphaned responses
- Collect metrics

We're laying the groundwork now, full correlation comes later.

## Commands for Development

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Test CLI changes
cargo run -- forward --help
cargo run -- forward --transport sse --sse-url http://localhost:8080 -- echo

# Run tests during development
cargo test transport::sse
cargo test cli::

# Check code quality
cargo fmt
cargo clippy --all-targets -- -D warnings

# Run integration test (once implemented)
cargo test --test sse_integration
```

## Success Criteria for This Session

- [ ] CLI accepts `--transport sse` with URL option
- [ ] Basic SseTransport struct implemented
- [ ] Transport trait methods stubbed/partially implemented
- [ ] Event ID generator integrated
- [ ] Initial tests written
- [ ] Code compiles without warnings

## Context for Next Session

After this session, we'll need to:
1. Complete S.2 if not finished
2. S.3: Wire SSE transport into forward proxy
3. S.4: Add deeper parser integration
4. Testing and refinement

## Key Files to Focus On

1. `src/cli/mod.rs` - Add transport option
2. `src/config/mod.rs` - SSE configuration
3. `src/transport/mod.rs` - Export new transport
4. `src/transport/sse_transport.rs` - New file to create
5. `src/proxy/forward.rs` - Integration point (S.3)

## Remember

- Keep changes small and testable
- Use the foundation components we built
- Don't implement full correlation yet (that's Phase 3)
- Focus on getting messages flowing through SSE
- The Event ID generator is already optimized and ready

---

**Goal**: Get SSE transport working with basic MCP awareness, leveraging all Phase 0 foundation components. Focus on S.1 (CLI) and S.2 (Transport wrapper) in this session.