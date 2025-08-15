# Session Prompt: SSE Infrastructure Analysis (Phase A.1)

## Context
I'm refactoring a 3,482-line reverse proxy that has SSE streaming issues. Phase A.0 (code analysis) is complete and identified that the proxy drops Response objects when detecting SSE and makes duplicate requests. The solution is to use an `UpstreamResponse` wrapper that keeps the response alive with unconsumed body stream.

## Your Task: Phase A.1 - SSE Infrastructure Review (1.5 hours)

Analyze our existing SSE modules in `shadowcat/src/transport/sse/` to understand what can be reused for the reverse proxy refactor. We have substantial SSE infrastructure that might solve our streaming problems.

## Starting Points

### 1. Read Project Context
```bash
cd ~/src/tapwire/shadowcat
cat ../plans/reverse-proxy-refactor/tracker.md  # Project status
cat ../plans/reverse-proxy-refactor/tasks/A.1-sse-infrastructure-review.md  # Your specific task
```

### 2. Review Previous Analysis
```bash
# Key findings from Phase A.0
cat ../plans/reverse-proxy-refactor/analysis/findings-summary.md
cat ../plans/reverse-proxy-refactor/analysis/corrected-sse-solution.md
```

### 3. Analyze Existing SSE Modules
The `src/transport/sse/` directory contains:
- `parser.rs` - SSE event parsing from byte streams
- `client.rs` - SSE client implementation
- `connection.rs` - Connection management
- `event.rs` - Event structures
- `manager.rs` - Connection manager
- `reconnect.rs` - Reconnection logic (61KB!)
- `session.rs` - Session-aware SSE
- `buffer.rs` - Stream buffering
- `mod.rs` - Module exports

**Key Questions:**
1. Can we use `SseParser` to parse upstream SSE events incrementally?
2. Does `SseStream` handle chunked responses properly?
3. Can `SessionAwareSseManager` help with session mapping?
4. Is the reconnection logic applicable to proxy scenarios?
5. What's the relationship between these modules?

### 4. Compare with Reference Implementations

Check the official MCP implementations for patterns:

```bash
# TypeScript SDK (most mature implementation)
cat ~/src/modelcontextprotocol/typescript-sdk/src/client/sse.ts
cat ~/src/modelcontextprotocol/typescript-sdk/src/server/sse.ts

# Rust SDK for comparison
find ~/src/modelcontextprotocol/rust-sdk -name "*.rs" | xargs grep -l "SSE\|EventSource"

# Inspector's SSE handling (production example)
find ~/src/modelcontextprotocol/inspector -name "*.ts" | xargs grep -l "EventSource\|text/event-stream"
```

### 5. Integration Analysis

Based on the corrected solution in `analysis/corrected-sse-solution.md`, we need:
1. Parse SSE events from `UpstreamResponse.response.bytes_stream()`
2. Run parsed events through interceptor chain
3. Forward modified events to client
4. Handle connection drops gracefully

**Which existing modules can help with each requirement?**

### 6. Session Management Considerations (CRITICAL)

**Review related plans for distributed session architecture:**
```bash
# Redis session storage abstraction
cat ../plans/redis-session-storage/redis-storage-tracker.md
cat ../plans/redis-session-storage/analysis/redis-architecture.md

# Reverse proxy session mapping
cat ../plans/reverse-proxy-session-mapping/README.md
cat ../plans/reverse-proxy-session-mapping/analysis/transport-layer-analysis.md
```

**Key Requirements for SSE + Session Management:**
1. **Session Storage Abstraction**: We need a `SessionStore` trait that supports both in-memory and Redis backends
2. **Dual Session IDs**: Proxy maintains its own session IDs separate from upstream
3. **Session Mapping**: Map proxy sessions → upstream sessions (many-to-one possible)
4. **Distributed State**: Sessions must work across multiple proxy instances
5. **Connection Pooling**: Reuse upstream connections across client sessions

**SSE Streaming & Backpressure (Important to Investigate):**
- **Minimize Buffering**: The proxy should be a thin pipe, not a reservoir
- **Backpressure Propagation**: Slow client → slow upstream reads, slow upstream → slow client writes
- **Event Buffering**: Should be a SEPARATE abstraction from session storage (if needed at all)
- **Investigate**: How much buffering is actually needed for Last-Event-Id replay?

**Questions to Answer:**
- How does `SessionAwareSseManager` handle distributed sessions?
- Can we propagate backpressure through the proxy naturally?
- How much event history is needed for SSE reconnection?
- What's the minimum viable buffer size for SSE events?
- How do reference implementations handle backpressure?
- How to maintain session mapping during SSE streaming?
- What happens to SSE streams during upstream failover?

## Deliverables

Create `analysis/sse-infrastructure.md` documenting:

1. **Module Capabilities**
   - What each SSE module does
   - Key structs/traits and their purposes
   - Dependencies between modules

2. **Reusability Assessment**
   - Which components can be used as-is
   - Which need modification for proxy use
   - Which are not applicable

3. **Integration Points**
   - How to use `SseParser` with `bytes_stream()`
   - How to integrate with interceptor chain
   - Session management considerations

4. **Session Management Architecture**
   - How SSE modules interact with SessionManager
   - Support for distributed sessions (Redis backend)
   - Session mapping for proxy scenarios
   - Event buffering for reconnection
   - Connection pooling implications

5. **Gap Analysis**
   - What's missing for proxy SSE support
   - What needs to be built new
   - Potential conflicts or issues
   - Session management gaps

6. **Recommendation**
   - Specific modules to reuse
   - Required modifications
   - Implementation approach
   - Session architecture decisions

## Success Criteria

- [ ] Documented all SSE module capabilities
- [ ] Identified specific components for reuse
- [ ] Clear integration strategy with `UpstreamResponse`
- [ ] Comparison with reference implementations
- [ ] Actionable recommendations for Phase B

## Time Management

- 30 min: Explore existing SSE modules
- 30 min: Compare with reference implementations  
- 20 min: Integration analysis
- 10 min: Document findings

## Important Context

The reverse proxy currently:
- Makes HTTP request to upstream
- Gets response with headers
- Detects SSE via Content-Type: text/event-stream
- **PROBLEM**: Drops Response and makes duplicate request
- **SOLUTION**: Keep Response alive in `UpstreamResponse` wrapper

We need SSE infrastructure that can:
- Parse events from unconsumed `bytes_stream()`
- Process events through interceptors
- Stream modified events to client
- Handle large/infinite streams efficiently

## Notes

- The `mime` crate is available for Content-Type parsing
- We already have `tokio`, `futures`, `bytes` for async/streaming
- The existing SSE modules may have been designed for client use, not proxying
- Consider memory efficiency for long-lived SSE connections
- Review `reconnect.rs` carefully - it's 61KB of code!