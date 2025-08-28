# Next Session Prompt - Streamable HTTP Implementation

## 🎯 Current Focus: Streamable HTTP Transport

**IMPORTANT**: Read the comprehensive knowledge base first:
```bash
cat /Users/kevin/src/tapwire/plans/mcp-unified-architecture/SSE-AND-STREAMING-KNOWLEDGE.md
```

## Context
We're implementing MCP's **Streamable HTTP** transport - a single transport that supports both:
- **HTTP-only mode**: Returns `application/json` for single responses
- **SSE mode**: Returns `text/event-stream` for streaming responses

## What We've Done
✅ Understood the Streamable HTTP specification  
✅ Created `StreamableHttpConfig` for both stateful/stateless modes  
✅ Started `StreamableIncomingConnection` (server-side)  
✅ Documented all SSE knowledge and existing code  
✅ Identified reusable components from shadowcat  

## What's Next

### Immediate TODO: Fix SSE Body Streaming
Location: `crates/mcp/src/transport/http/streamable_incoming.rs`

Current issue at line ~219:
```rust
// TODO: Implement SSE streaming body
.body(Full::new(Bytes::from("TODO: Implement SSE streaming body")))
```

Need to:
1. Use `http_body_util::StreamBody` or similar for streaming
2. Reference shadowcat's SSE implementation for patterns
3. Stream SSE events through the HTTP response body

### Then: Complete Server Implementation
- [ ] GET request handling for server-initiated streams
- [ ] Session management integration
- [ ] Last-Event-Id support for resumability

### Next: Create Client Implementation
- [ ] Create `streamable_outgoing.rs` 
- [ ] Implement `Outgoing` trait
- [ ] Handle both JSON and SSE response types
- [ ] Reuse SSE parser from shadowcat

## Key Code Locations

```bash
# What we're working on
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance/crates/mcp/src/transport/http/
ls streamable_*.rs

# Existing SSE to reuse
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance/src/transport/sse/
ls *.rs  # Full SSE implementation we can leverage

# Event tracking abstraction
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance/crates/mcp/src/events/
cat tracker.rs  # Generic event tracking trait
```

## Key Insights to Remember

1. **Streamable HTTP = One Transport, Two Modes**
   - Not a separate transport!
   - Server chooses based on Accept header and config

2. **We Have Sophisticated SSE Already**
   - Full implementation in shadowcat
   - Reconnection, parsing, buffering all done
   - Just need to integrate with MCP patterns

3. **Event Tracking is Abstracted**
   - Generic `EventTracker` trait
   - SSE-specific implementation exists
   - Ready for WebSockets in future

## Architecture Reminder

```
Client Request:
  Accept: application/json, text/event-stream
  
Server Decision:
  if stateless_mode OR !accepts_sse:
    → Return application/json
  else:
    → Return text/event-stream
```

## Testing Approach

Start with simple cases:
1. HTTP-only mode (stateless) - single JSON responses
2. SSE mode (stateful) - streaming responses
3. Dynamic switching based on Accept header

## Questions to Keep in Mind

- **"Would this abstraction work for WebSockets too?"**
- **"Can we reuse existing shadowcat SSE code?"**
- **"Is this properly abstracted from transport specifics?"**

## Commands to Start

```bash
# Navigate to working directory
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance/crates/mcp

# Review current implementation
cat src/transport/http/streamable_incoming.rs | grep -A5 -B5 TODO

# Check shadowcat SSE for reference
cat ../../src/transport/sse/buffer.rs  # How they handle streaming

# Run tests to ensure nothing broke
cargo test --lib transport::http

# When ready to test streaming
cargo run --example streamable_http_demo  # (need to create this)
```

## Remember
- This is a **big lift** - quality over speed!
- We have existing SSE code - **reuse it**!
- Think about WebSocket compatibility
- Document as you go

---

**Start Point**: Fix the SSE body streaming TODO in `streamable_incoming.rs`  
**Knowledge Base**: `SSE-AND-STREAMING-KNOWLEDGE.md` has everything you need