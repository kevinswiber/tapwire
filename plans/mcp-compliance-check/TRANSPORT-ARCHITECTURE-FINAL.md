# Transport Architecture - FINAL CONSOLIDATED DECISION

> ⚠️ **DEPRECATED**: This architecture has been superseded by the Connection pattern.
> 
> **See**: [TRANSPORT-ARCHITECTURE-FINAL-V3-CONNECTION-PATTERN.md](analysis/TRANSPORT-ARCHITECTURE-FINAL-V3-CONNECTION-PATTERN.md)
> 
> **Reason**: Sink/Stream pattern doesn't scale for proxy use case (10K+ connections).
> Worker pattern adds unacceptable overhead. Moving to async_trait Connection pattern.
>
> **This document is preserved for historical context only.**

---

**Original Status**: ~~FINAL~~ DEPRECATED  
**Date**: 2025-08-24  
**Original Decision**: Framed/Sink/Stream with WebSocket as separate transport

## Architecture Decision

### Core Approach: Message-Level Sink+Stream
```rust
// All transports implement these standard traits
impl Sink<Value> for Transport { }
impl Stream<Item = io::Result<Value>> for Transport { }
```

### Transport Types

| Transport | Implementation | Protocol | Sessions |
|-----------|---------------|----------|----------|
| **Stdio** | `Framed` with `JsonLineCodec` | Line-delimited JSON | N/A |
| **Subprocess** | `Framed` with `JsonLineCodec` | Line-delimited JSON | N/A |
| **TCP/Unix** | `Framed` with `JsonLineCodec` | Line-delimited JSON | Optional |
| **HTTP** | Worker with channels | POST + JSON/SSE | Headers (optional) |
| **WebSocket** | Separate transport | GET + Upgrade | Required in messages |

### Key Decisions

1. **Framed for line protocols only** - stdio, subprocess, TCP, Unix sockets
2. **HTTP is adaptive** - Single transport handling JSON response or SSE stream
3. **WebSocket is separate** - Different lifecycle, auth, session requirements
4. **Worker pattern for HTTP** - Manages client, SSE streams, reconnection
5. **Background receiver for Client** - Prevents deadlock, enables concurrent ops

## Implementation Status

### ✅ Completed
- JsonLineCodec with Encoder/Decoder
- StdioTransport using Framed
- SubprocessTransport with process management
- Basic HttpTransport structure
- Client/Server using Sink+Stream traits

### 🔴 Critical Bugs (from GPT-5 review)
1. **Client deadlock** - `request()` blocks without `run()`, but `run()` consumes self
2. **HTTP not working** - Just shuffles queues, doesn't send requests

### 📋 TODO
1. Fix Client with background receiver (CRITICAL)
2. Implement HTTP worker pattern (CRITICAL)
3. Create WebSocket transport (separate module)
4. Harden JsonLineCodec (CRLF, overlong lines)
5. Wire version negotiation

## Deprecated Documents

The following documents are superseded by this consolidated decision:
- `transport-architecture-investigation.md` → Initial exploration
- `transport-architecture-final.md` → First decision
- `transport-architecture-final-v2.md` → Second iteration
- `transport-decision-summary.md` → Partial summary
- `transport-investigation-summary.md` → Investigation notes
- `transport-patterns-analysis.md` → Pattern exploration
- `transport-deviation-analysis.md` → Deviation notes
- `http-sse-unified-transport.md` → Subsumed into HTTP adaptive
- `http-transport-unified-architecture.md` → Subsumed into HTTP adaptive
- `rmcp-vs-framed-comparison.md` → Analysis complete
- `framed-sink-stream-architecture.md` → Integrated here

## Active Documents

Keep these for reference:
- **This document** - Final architecture decision
- `websocket-separation-decision.md` - WebSocket rationale
- `gpt-findings-analysis.md` - Critical bug analysis
- `CURRENT-ARCHITECTURE.md` - Overall system architecture
- `DECISION-LOG.md` - Decision history

## Code Locations

- **Transports**: `crates/mcp/src/transport/`
  - `codec.rs` - JsonLineCodec
  - `stdio.rs` - StdioTransport  
  - `subprocess.rs` - SubprocessTransport
  - `http/mod.rs` - HttpTransport (needs worker)
  - `websocket.rs` - TODO: WebSocket transport
- **Client/Server**: `crates/mcp/src/`
  - `client.rs` - Needs background receiver
  - `server.rs` - Using Sink+Stream
- **Session Management**: `src/session/` - Existing infrastructure

## References

- [MCP WebSocket Proposal #1288](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1288)
- [GPT-5 Findings](analysis/gpt-findings-analysis.md)
- [WebSocket Decision](analysis/websocket-separation-decision.md)