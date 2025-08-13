# Next Session: Phase 2 - Raw Transport Implementation

## Project Context

We've completed the foundation design for the transport layer refactor. All trait definitions are in place, and we're ready to implement the raw transport layer.

**Project**: Transport Layer Refactor  
**Tracker**: `plans/transport-refactor/transport-refactor-tracker.md`  
**Status**: Phase 2 - Raw Transport Layer (0% Complete)

## Current Status

### What Has Been Completed

#### Phase 0: Prerequisites and Analysis (✅ Completed 2025-08-13)
- A.1: Documented all transport patterns and architectural issues
- A.2: Created 16 regression tests capturing current behavior  
- A.3: Comprehensive risk assessment with migration strategies

#### Phase 1: Foundation Design (✅ Completed 2025-08-13)
- F.1: Created RawTransport trait in `src/transport/raw/mod.rs`
- F.2: Created McpProtocolHandler in `src/transport/protocol/mod.rs`
- F.3: Created IncomingTransport/OutgoingTransport in `src/transport/directional/mod.rs`
- F.4: Created ProcessManager in `src/process/mod.rs`
- F.5: Migration strategy defined (no compat layer needed - pre-release)

### Key Design Decisions Made

1. **No Compatibility Layer**: Since Shadowcat is pre-release, we can make breaking changes
2. **Single Protocol Handler**: `McpProtocolHandler` handles MCP over JSON-RPC 2.0
3. **Process Management Separated**: ProcessManager completely extracted from transports
4. **Clear Naming**: IncomingTransport (accepts connections) vs OutgoingTransport (initiates connections)

### Foundation Files Created
- `shadowcat/src/transport/raw/mod.rs` - RawTransport trait definition
- `shadowcat/src/transport/protocol/mod.rs` - McpProtocolHandler implementation
- `shadowcat/src/transport/directional/mod.rs` - Incoming/Outgoing traits
- `shadowcat/src/process/mod.rs` - ProcessManager trait and implementation
- `shadowcat/src/error.rs` - Added new error variants

## Your Mission

Implement the raw transport layer - the foundational byte-level I/O without any protocol knowledge.

### Priority Tasks for This Session (16 hours total)

1. **R.1: Implement StdioRawTransport** (3h)
   - Read task file: `plans/transport-refactor/tasks/R.1-stdio-raw-transport.md`
   - Create `src/transport/raw/stdio.rs`
   - Implement both StdioRawIncoming and StdioRawOutgoing
   - Line-based framing, async I/O with tokio

2. **R.2: Implement HttpRawTransport** (3h)
   - Read task file: `plans/transport-refactor/tasks/R.2-http-raw-transport.md`
   - Create `src/transport/raw/http.rs`
   - HttpRawClient using reqwest
   - HttpRawServer using axum

3. **R.3: Implement SseRawTransport** (3h)
   - Read task file: `plans/transport-refactor/tasks/R.3-sse-raw-transport.md`
   - Create `src/transport/raw/sse.rs`
   - Implement StreamingRawTransport trait
   - SSE event parsing and generation

4. **R.4: Implement StreamableHttpRawTransport** (4h)
   - Read task file: `plans/transport-refactor/tasks/R.4-streamable-http-raw.md`
   - Create `src/transport/raw/streamable_http.rs`
   - Unified HTTP POST + SSE response
   - This fixes the core architectural issue!

5. **R.5: Create RawTransport tests** (3h)
   - Comprehensive test suite for all raw transports
   - Test framing, buffering, lifecycle
   - Performance benchmarks

## Essential Context Files to Read

1. **Foundation Design**:
   - `plans/transport-refactor/analysis/foundation-design.md` - Complete design overview
   - `shadowcat/src/transport/raw/mod.rs` - RawTransport trait to implement
   
2. **Current Implementation** (for reference):
   - `shadowcat/src/transport/stdio.rs` - Current stdio implementation
   - `shadowcat/src/transport/http.rs` - Current HTTP implementation
   - `shadowcat/src/transport/sse_transport.rs` - Current SSE implementation
   
3. **Test Suite**:
   - `shadowcat/tests/transport_regression_suite.rs` - Must continue passing

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Commands to Run First

```bash
# Verify regression tests still pass
cargo test --test transport_regression_suite

# Check that foundation compiles
cargo check

# See current transport implementations
ls -la src/transport/
```

## Implementation Guidelines

### For Each Raw Transport

1. **No Protocol Knowledge**: These handle only bytes, no JSON-RPC or MCP
2. **Async I/O**: Use tokio for all async operations
3. **Proper Framing**: Each transport defines its framing (lines for stdio, HTTP messages, SSE events)
4. **Error Handling**: Map errors to TransportError variants
5. **Testing**: Unit tests for each implementation

### StdioRawTransport Specifics
- Line-based framing (newline delimited)
- Separate implementations for incoming (stdin) and outgoing (subprocess)
- Use ProcessManager for subprocess variant
- Buffer management with mpsc channels

### HttpRawTransport Specifics
- Reuse existing reqwest/axum code where possible
- Preserve headers for protocol negotiation
- Connection pooling for client
- Proper server lifecycle management

### SseRawTransport Specifics
- Implement StreamingRawTransport trait (not RawTransport)
- Parse SSE format: data:, event:, id:, retry:
- Handle reconnection logic
- Stream-based interface

### StreamableHttpRawTransport Specifics
- Compose HttpRawTransport and SseRawTransport
- Mode switching: Request (HTTP) → Streaming (SSE)
- Session tracking for server
- This is the KEY INNOVATION of the refactor

## Success Criteria Checklist

- [ ] StdioRawTransport implemented and tested
- [ ] HttpRawTransport implemented and tested
- [ ] SseRawTransport implemented and tested
- [ ] StreamableHttpRawTransport implemented and tested
- [ ] All raw transports have unit tests
- [ ] Regression tests still pass
- [ ] No protocol knowledge in raw transports
- [ ] Performance within 2% of current

## Important Notes

- **Focus on R.1 first** - StdioRawTransport is the simplest and sets patterns
- **Then R.2 and R.3 in parallel** - Independent implementations
- **R.4 last** - Composes R.2 and R.3
- **No backward compatibility needed** - We're pre-release
- **Keep it simple** - Just bytes, no protocol logic

## Next Steps After This Session

Once Phase 2 is complete:
- **Phase 3**: Wire up protocol handlers (7h)
- **Phase 4**: Implement direction-aware transports (14h)
- **Phase 5**: Migration and cleanup (11h)

## Key Design Reminder

The whole point of this refactor is to fix the confusion between:
- `StdioTransport` (spawns process) → becomes `SubprocessOutgoing`
- `StdioClientTransport` (reads stdin) → becomes `StdioIncoming`

And to enable:
- Unified `StreamableHttpTransport` for MCP's HTTP+SSE protocol

---

**Session Goal**: Implement all raw transports with comprehensive tests. This is the foundation everything else builds on.

**Last Updated**: 2025-08-13  
**Next Review**: After Phase 2 completion