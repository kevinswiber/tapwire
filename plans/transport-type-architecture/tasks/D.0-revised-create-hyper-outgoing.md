# Task D.0 (Revised): Create HyperOutgoing Transport and Clean Up

**Duration**: 2.5 hours  
**Dependencies**: Phase C complete  
**Priority**: HIGH

## Context Change

After analyzing HTTP client implementations, we discovered:
- **5 different HTTP client implementations** exist
- `transport::http_client.rs` is **completely unused** (delete it!)
- Reverse proxy uses hyper directly for SSE streaming control
- Forward proxy uses reqwest-based HttpClientOutgoing
- We should create HyperOutgoing (not HttpOutgoing) to unify on hyper

## Objective

1. Delete unused HTTP client code
2. Create HyperOutgoing implementation of OutgoingTransport trait
3. Enable reverse proxy to use trait abstractions
4. Maintain SSE streaming capabilities

## Key Decisions

1. **Use Hyper, not Reqwest**: Better streaming control for SSE
2. **Wrap existing HyperHttpClient**: Don't reinvent, just add trait
3. **Delete unused code**: Remove technical debt immediately

## Process

### Step 1: Delete Unused Code (10 min)

```bash
# Delete the completely unused global HTTP client
rm src/transport/http_client.rs

# Update module exports
# Edit src/transport/mod.rs to remove:
# - pub mod http_client;
# - Any exports from http_client

# Verify nothing breaks
cargo check
```

### Step 2: Create HyperOutgoing Structure (30 min)

Create new file: `src/transport/directional/outgoing/hyper.rs`

```rust
use super::OutgoingTransport;
use crate::proxy::reverse::hyper_client::HyperHttpClient;
use crate::transport::{SessionId, MessageEnvelope, ResponseMode};

/// Hyper-based HTTP outgoing transport for reverse proxy
/// 
/// This provides trait abstraction over HyperHttpClient,
/// enabling connection pooling, testing, and SSE support.
pub struct HyperOutgoing {
    client: HyperHttpClient,
    session_id: SessionId,
    target_url: String,
    pending_response: Option<hyper::Response<hyper::body::Incoming>>,
}

impl HyperOutgoing {
    pub fn new(target_url: String) -> TransportResult<Self> {
        Ok(Self {
            client: HyperHttpClient::new(),
            session_id: SessionId::new(),
            target_url,
            pending_response: None,
        })
    }
}
```

### Step 3: Implement OutgoingTransport Trait (45 min)

Key implementation points:

```rust
impl OutgoingTransport for HyperOutgoing {
    async fn connect(&mut self) -> TransportResult<()> {
        // Hyper client doesn't need explicit connection
        Ok(())
    }

    async fn send_request(&mut self, envelope: MessageEnvelope) -> TransportResult<()> {
        // Use HyperHttpClient::send_mcp_request
        let response = self.client.send_mcp_request(
            &self.target_url,
            &envelope.message,
            // Need to pass session somehow - may need to refactor
        ).await?;
        
        // Store response for receive_response
        self.pending_response = Some(response);
        Ok(())
    }

    async fn receive_response(&mut self) -> TransportResult<MessageEnvelope> {
        let response = self.pending_response.take()
            .ok_or(TransportError::NotConnected)?;
        
        // Detect response mode
        let response_mode = if response.is_sse() {
            ResponseMode::SseStream
        } else {
            ResponseMode::Json
        };
        
        // Handle based on mode
        match response_mode {
            ResponseMode::Json => {
                // Buffer and parse JSON response
                let body = response.into_body().collect().await?;
                // Parse and return envelope
            }
            ResponseMode::SseStream => {
                // Stream SSE events
                // This is complex - may need to return a different type
            }
        }
    }
}
```

### Step 4: Handle SSE Streaming Challenge (30 min)

The main challenge: OutgoingTransport expects discrete messages, but SSE is a stream.

Options:
1. **Buffer first event**: Return first SSE event, queue rest
2. **Change trait**: Add streaming support to OutgoingTransport
3. **Hybrid approach**: Return special "streaming started" message

Recommended: Option 1 with internal event queue

### Step 5: Update Reverse Proxy (20 min)

Modify `src/proxy/reverse/legacy.rs`:

```rust
// Instead of direct HyperHttpClient usage:
use crate::transport::directional::outgoing::HyperOutgoing;

// In process_via_http_hyper:
let mut transport = HyperOutgoing::new(url)?;
transport.connect().await?;
transport.send_request(envelope).await?;
let response = transport.receive_response().await?;
```

### Step 6: Enable Connection Pooling (15 min)

Make HyperOutgoing work with PoolableOutgoingTransport:

```rust
// HyperOutgoing should be compatible with pooling wrapper
let pooled = PoolableOutgoingTransport::new(Box::new(transport));
```

## Commands to Run

```bash
# Step 1: Delete unused file
rm src/transport/http_client.rs

# Step 2-4: Create new file
touch src/transport/directional/outgoing/hyper.rs

# Step 5: Run tests
cargo test transport::directional::outgoing::hyper
cargo test reverse_proxy

# Step 6: Full test suite
cargo test --lib
```

## Deliverables

1. **Deleted Files**:
   - `src/transport/http_client.rs` - Removed unused code

2. **Created Files**:
   - `src/transport/directional/outgoing/hyper.rs` - HyperOutgoing implementation

3. **Modified Files**:
   - `src/transport/mod.rs` - Remove http_client module
   - `src/transport/directional/outgoing/mod.rs` - Export HyperOutgoing
   - `src/proxy/reverse/legacy.rs` - Use HyperOutgoing

4. **Tests**:
   - Unit tests for HyperOutgoing
   - Integration tests with reverse proxy

## Success Criteria

- [ ] Unused http_client.rs deleted
- [ ] HyperOutgoing implements OutgoingTransport completely
- [ ] SSE streaming works through trait interface
- [ ] Reverse proxy can use HyperOutgoing
- [ ] Connection pooling compatible
- [ ] All existing tests pass
- [ ] No performance regression

## Challenges and Solutions

### Challenge 1: SSE Streaming Through Trait
**Problem**: OutgoingTransport expects discrete messages, SSE is continuous
**Solution**: Buffer events internally, return them one by one

### Challenge 2: Session Management  
**Problem**: HyperHttpClient needs Session object, trait only has SessionId
**Solution**: May need to refactor HyperHttpClient to work with just SessionId

### Challenge 3: Response Body Ownership
**Problem**: Hyper response body can only be consumed once
**Solution**: Store pending response, consume in receive_response

## Alternative Approach

If HyperOutgoing proves too complex, consider:
1. Keep HyperHttpClient as-is for now
2. Create simpler HttpOutgoing using reqwest
3. Migrate to hyper later in dedicated refactor

## Notes

- This achieves the same goal as original D.0 but with better foundation
- Deleting unused code is always good
- Hyper gives us the streaming control we need
- This is still a targeted improvement, not a full refactor