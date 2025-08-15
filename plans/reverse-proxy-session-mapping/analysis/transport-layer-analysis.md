# Transport Layer and Proxy Type Analysis

## Key Discoveries

### Transport Layer Impact

1. **MessageEnvelope and MessageContext**
   - `MessageContext` contains a `session_id: SessionId` field
   - This is used by both IncomingTransport and OutgoingTransport
   - Transport implementations expect a single session ID per message

2. **Current Transport Assumptions**
   - Transports work with a single session ID
   - `set_session_id()` method on transports assumes one session
   - Connection pooling is based on session IDs

### Forward vs Reverse Proxy Differences

#### Forward Proxy (One-to-One)
```rust
pub struct ForwardProxy {
    session_id: SessionId,  // Single session for entire proxy lifetime
    // ...
}
```
- **Single Session**: One client connection = one upstream connection = one session
- **Session Ownership**: Client owns the session, proxy just passes it through
- **Transport Usage**: Both transports use the same session ID
- **Simpler Model**: No session mapping needed

#### Reverse Proxy (Many-to-Many)
```rust
// Currently handles multiple sessions dynamically
async fn handle_mcp_request(...) {
    // Extract session from each request
    let session_id = extract_from_headers(...);
    // ...
}
```
- **Multiple Sessions**: Many clients, potentially many upstreams
- **Session Ownership**: Proxy manages sessions, maps to upstream sessions
- **Transport Usage**: Different session IDs for client-facing vs upstream
- **Complex Model**: Needs session mapping

## Transport Layer Modifications Needed

### 1. **MessageContext Enhancement**
```rust
pub struct MessageContext {
    pub session_id: SessionId,           // Proxy's session ID
    pub upstream_session_id: Option<String>, // NEW: Upstream's session ID
    // ... existing fields
}
```

### 2. **Transport Trait Changes**
The transports themselves don't need changes - they continue using the proxy's session ID. The mapping happens at the proxy layer when:
- Sending to upstream: Replace session_id in headers/protocol
- Receiving from upstream: Map back to proxy session

### 3. **Connection Pooling Impact**
- Pool connections by upstream_session_id when available
- Fall back to proxy session_id for initial requests
- May need to update PoolKey to include both IDs

## Proxy-Specific Behaviors

### Forward Proxy
- **No changes needed** - continues to use single session
- Pass-through model remains simple
- Session ID from client is used throughout

### Reverse Proxy
- **Primary changes here** - needs full mapping implementation
- Generate proxy session IDs for clients
- Map to upstream session IDs after initialization
- Handle SSE reconnection with proxy session IDs
- Buffer events per proxy session

## Implementation Strategy

### Phase 1: Reverse Proxy Only
Focus all changes on reverse proxy initially:
1. Add mapping table to reverse proxy
2. Keep forward proxy unchanged
3. Transports use proxy session IDs

### Phase 2: Transport Context (Optional)
If needed for optimization:
1. Add upstream_session_id to MessageContext
2. Use for connection pooling optimization
3. Still transparent to transport implementations

## Risk Assessment

### Lower Risk Approach
- Keep transport layer mostly unchanged
- Do mapping at proxy layer (application level)
- Transports continue working with single session ID

### Higher Risk Approach  
- Modify transports to understand dual sessions
- More complex but potentially more efficient
- Risk breaking existing transport implementations

## Recommendation

**Start with Lower Risk Approach**:
1. Implement mapping in reverse proxy only
2. Keep transports using proxy session IDs
3. Do header/protocol translation at proxy boundaries
4. Only modify MessageContext if performance requires it

This minimizes changes to the transport layer while achieving all our goals for session mapping, SSE reconnection, and failover support.