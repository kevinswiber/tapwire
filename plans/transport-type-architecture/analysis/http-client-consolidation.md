# HTTP Client Consolidation Analysis

**Created**: 2025-08-17  
**Purpose**: Analyze and consolidate the multiple HTTP client implementations in shadowcat

## Current State - Too Many HTTP Clients!

We have **FIVE** different HTTP client implementations, using two different libraries (reqwest and hyper):

### 1. `transport::http_client` - Global Pooled Client
- **Library**: reqwest
- **Purpose**: Global singleton with connection pooling
- **Used by**: **NOTHING!** (only its own tests)
- **Features**: Connection pooling, HTTP/2, configurable timeouts
- **Status**: 🗑️ **DELETE** - Completely unused

### 2. `transport::raw::http::HttpRawClient` - Raw HTTP Transport
- **Library**: reqwest (creates its own client)
- **Purpose**: Raw byte-level HTTP transport without protocol knowledge
- **Used by**: `HttpClientOutgoing` directional transport
- **Features**: Custom headers, retries, request/response channels
- **Status**: ✅ **KEEP** - Core transport abstraction

### 3. `transport::raw::streamable_http::StreamableHttpRawClient` - HTTP+SSE Transport
- **Library**: None directly (uses HttpRawClient + SseRawClient)
- **Purpose**: Combines HTTP POST requests with SSE streaming responses
- **Used by**: `StreamableHttpOutgoing` directional transport
- **Features**: Mode switching between request/streaming, session management
- **Status**: ✅ **KEEP** - MCP streamable-http protocol support

### 4. `transport::directional::HttpClientOutgoing` - Directional HTTP
- **Library**: Uses HttpRawClient
- **Purpose**: OutgoingTransport trait implementation for HTTP
- **Used by**: Forward proxy for HTTP upstreams
- **Features**: Protocol handling, message envelope wrapping
- **Status**: ✅ **KEEP** - Required for forward proxy

### 5. `proxy::reverse::hyper_client::HyperHttpClient` - Direct Hyper Client
- **Library**: hyper (not reqwest)
- **Purpose**: Fine control over SSE streaming for reverse proxy
- **Used by**: Reverse proxy for HTTP upstreams (process_via_http_hyper)
- **Features**: Direct body streaming control, SSE detection
- **Status**: ⚠️ **REFACTOR** - Should use directional transport

## The Problems

### 1. Library Inconsistency
- Most transports use **reqwest** (higher-level, easier)
- Reverse proxy uses **hyper** directly (lower-level, more control)
- This split creates maintenance burden

### 2. Abstraction Level Mismatch
- Forward proxy uses clean trait abstractions (HttpClientOutgoing)
- Reverse proxy bypasses abstractions (HyperHttpClient)
- Can't share features like connection pooling

### 3. Unused Code
- `transport::http_client` is completely unused
- Global connection pool that nobody uses

### 4. SSE Handling Duplication
- Both streamable_http and hyper_client handle SSE
- Different approaches to same problem

## Root Cause Analysis

The proliferation happened because:
1. **Evolution**: Different parts added at different times
2. **SSE Complexity**: Reqwest doesn't handle SSE streaming well, led to hyper
3. **Missing Abstraction**: No HttpOutgoing for reverse proxy to use
4. **Premature Optimization**: Global pool created but never integrated

## Consolidation Plan

### Phase 1: Quick Cleanup (30 minutes)
1. **DELETE** `transport::http_client.rs` - Completely unused
2. **DELETE** `transport::http_utils.rs` (if also unused)
3. Update any imports/exports

### Phase 2: Unify on Hyper (2-3 hours)
Convert all HTTP clients to use hyper directly for consistency:

1. **Refactor HttpRawClient** to use hyper instead of reqwest
   - Better streaming control
   - Consistent with reverse proxy needs
   - Enables proper SSE handling

2. **Create HyperOutgoing** implementing OutgoingTransport
   - Wraps HyperHttpClient functionality
   - Enables reverse proxy to use trait abstraction
   - Supports both JSON and SSE responses

### Phase 3: Simplify Architecture (1-2 hours)
Final structure should be:

```
transport/
├── raw/
│   ├── http.rs          # Hyper-based raw HTTP
│   ├── sse.rs           # Hyper-based SSE streaming
│   └── streamable_http.rs # Combines HTTP+SSE
├── directional/
│   └── outgoing/
│       ├── http.rs      # HttpOutgoing (wraps raw)
│       ├── streamable.rs # StreamableHttpOutgoing
│       └── hyper.rs     # HyperOutgoing (for reverse proxy)
```

## Benefits of Consolidation

1. **Single HTTP Library**: All on hyper for consistency
2. **Better SSE Support**: Hyper handles streaming properly
3. **Code Reuse**: Reverse proxy can use directional transports
4. **Connection Pooling**: Can be shared across all HTTP transports
5. **Less Maintenance**: Fewer implementations to maintain

## Migration Strategy

### Step 1: Delete Unused Code
```bash
rm src/transport/http_client.rs
rm src/transport/http_utils.rs  # if unused
```

### Step 2: Create HyperOutgoing
This aligns with our Phase D.0 task but with clearer requirements:
- Implement OutgoingTransport trait
- Wrap existing HyperHttpClient
- Support SSE detection and streaming

### Step 3: Migrate HttpRawClient
- Replace reqwest with hyper
- Maintain same interface
- Add connection pooling

### Step 4: Update Reverse Proxy
- Use HyperOutgoing instead of direct HyperHttpClient
- Enable trait-based testing and pooling

## Risks and Mitigations

**Risk**: Breaking existing HTTP functionality
- **Mitigation**: Comprehensive testing at each step

**Risk**: Hyper is lower-level than reqwest
- **Mitigation**: Create helper functions for common patterns

**Risk**: SSE handling differences
- **Mitigation**: Test with MCP Inspector and real SSE servers

## Recommendation

### Immediate Actions (Today)
1. ✅ Delete unused `http_client.rs`
2. ✅ Proceed with Phase D.0 but create `HyperOutgoing` instead of `HttpOutgoing`
3. ✅ Use existing `HyperHttpClient` as the implementation

### Future Work (Later)
1. Migrate all HTTP to hyper (not urgent)
2. Unify SSE handling approaches
3. Add connection pooling to hyper implementations

## Conclusion

We have too many HTTP clients because of organic growth and the SSE streaming challenge. The reverse proxy needed hyper for proper streaming control, while other parts used simpler reqwest.

The path forward is clear:
1. Delete what's unused
2. Create HyperOutgoing for trait abstraction
3. Eventually standardize on hyper everywhere

This gives us immediate value (reverse proxy can use traits) without a massive refactor.