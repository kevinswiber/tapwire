# MCP Compliance Framework - Decision Log

## Transport Architecture (2025-08-24)

### Decision: Framed/Sink/Stream Architecture

**What**: All transports implement `Sink<JsonRpcMessage> + Stream<Item = Result<JsonRpcMessage>>`

**Why**: 
- Message-level abstraction matches MCP's message-oriented protocol
- Standard async traits enable ecosystem compatibility
- Simpler than custom Transport trait

**Details**:
- Line-delimited protocols use `tokio_util::codec::Framed`
- HTTP implements custom Sink/Stream for three modes
- Validated by RMCP's SinkStreamTransport pattern

### Decision: HTTP as Single Adaptive Transport

**What**: HTTP is ONE transport with THREE response modes (JSON, SSE, WebSocket)

**Why**:
- All three start with HTTP request
- Server chooses response mode based on operation
- Client handles all modes transparently

**Details**:
- 200 OK + application/json → Single response
- 200 OK + text/event-stream → SSE streaming
- 101 Switching Protocols → WebSocket upgrade

### Decision: Framed Only for Line-Delimited JSON

**What**: Use Framed with JsonLineCodec only for stdio, subprocess, future TCP/Unix

**Why**:
- Framed requires AsyncRead + AsyncWrite
- HTTP works at request/response level, not byte stream
- Each transport handles its own wire format

---

## Library Architecture (2025-08-23)

### Decision: Build Our Own MCP Implementation

**What**: Create independent MCP library, not depend on rmcp

**Why**:
- Need proxy-specific optimizations
- Want full control over core infrastructure
- Performance critical for our use case

### Decision: Single MCP Crate

**What**: One `crates/mcp/` crate with organized modules

**Why**:
- Simpler dependency management
- Easier refactoring across boundaries
- Clear module organization provides structure

### Decision: Copy-First Extraction

**What**: Copy code from shadowcat to create clean MCP crate

**Why**:
- No risk to existing shadowcat
- Freedom to design ideal API
- Can integrate back later (Phase H)

---

## Testing Strategy (2025-08-23)

### Decision: Three-Way Test Separation

**What**: Separate client tests (60) + server tests (60) + proxy tests (50)

**Why**:
- Precise diagnostics when tests fail
- Clear responsibility boundaries
- Easier to maintain and debug

### Decision: Streaming Test Results

**What**: Tests output JSON Lines for real-time progress

**Why**:
- Better CI/CD integration
- Real-time feedback during long test runs
- Machine-readable results

---

## Version Support (2025-08-23)

### Decision: Support Three MCP Versions

**What**: 2025-03-26, 2025-06-18, and draft (living spec)

**Why**:
- Cover current stable and latest
- Stay ahead with draft spec testing
- Version-agnostic architecture allows easy updates

---

*This log contains only decisions, not analysis. For detailed reasoning, see analysis/ directory.*