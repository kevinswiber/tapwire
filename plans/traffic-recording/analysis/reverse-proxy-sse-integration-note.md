# Reverse Proxy SSE Integration Note

## Important Integration Point

The reverse proxy refactor plan (`plans/reverse-proxy-refactor/`) has identified that we need to integrate `transport::outgoing::Http` with the reverse proxy. This is tracked in the transport type architecture plan.

## Current State

### Transport::outgoing::Http (Good SSE Buffering)
- Location: `src/transport/outgoing/http.rs`
- Has proper SSE event buffering with `VecDeque<Vec<u8>>`
- Handles SSE streaming through `OutgoingTransport` trait
- Recently simplified (removed unnecessary `BufferedSseEvent` struct)
- Clean separation between initial SSE response and buffered events

### Reverse Proxy (Better SSE Infrastructure)
- Location: `src/proxy/reverse/`
- Has 3 working SSE modules after consolidation:
  - `hyper_client.rs` - Low-level HTTP client
  - `hyper_raw_streaming.rs` - Raw SSE forwarding
  - `hyper_sse_intercepted.rs` - SSE with interceptors
- More mature SSE handling with proper streaming
- Already works with MCP Inspector

## Integration Plan (from transport-type-architecture)

From `plans/transport-type-architecture/tasks/D.0-unified-http-transport.md`:

### Remaining Work (Line 260-262):
> 2. **Reverse Proxy Integration** (30 min):
>    - Migrate from `proxy/reverse/hyper_client.rs` to unified transport
>    - Requires careful testing of existing functionality

### Status:
- This integration is **NOT YET DONE**
- Part of the 20% remaining work in the transport unification
- Needs to happen AFTER the traffic recording refactor

## What Needs to Happen

1. **After Traffic Recording Refactor**: Complete the removal of `TransportContext::Sse`
2. **During Reverse Proxy Integration**: 
   - Use `transport::outgoing::Http` as the upstream transport
   - Leverage its SSE buffering instead of duplicating
   - Keep the reverse proxy's superior SSE streaming for client connections
3. **Best of Both Worlds**:
   - Use reverse proxy's SSE infrastructure for client-facing connections
   - Use `transport::outgoing::Http` for upstream connections
   - Avoid duplication of SSE buffering logic

## Key Insight

The reverse proxy has better SSE handling for **client connections** (receiving SSE), while `transport::outgoing::Http` has good buffering for **upstream connections** (sending requests that receive SSE responses). We should use each where it's strongest.

## Action Items

1. ✅ Complete traffic recording refactor (remove `TransportContext::Sse`)
2. ⬜ Complete transport type architecture Phase D.0 (unified HTTP transport)
3. ⬜ Integrate `transport::outgoing::Http` with reverse proxy
4. ⬜ Remove duplicate SSE buffering logic
5. ⬜ Test with MCP Inspector to ensure no regression

## References

- SSE Module Consolidation: `plans/reverse-proxy-refactor/analysis/sse-module-consolidation.md`
- SSE Comparison: `plans/reverse-proxy-refactor/analysis/sse-comparison.md`
- Transport Unification: `plans/transport-type-architecture/tasks/D.0-unified-http-transport.md`
- Traffic Recording: `plans/traffic-recording/traffic-recording-tracker.md`

---

**Created**: 2025-08-17
**Purpose**: Track the integration point between transport::outgoing::Http and reverse proxy
**Status**: Waiting for traffic recording refactor completion