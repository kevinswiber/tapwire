# Task D.3: Update Proxies to Use Directional Transports

## Overview
Refactor ForwardProxy and ReverseProxy to use the new IncomingTransport and OutgoingTransport traits instead of the generic Transport trait.

**Duration**: 6 hours (increased from 3h after analysis)
**Dependencies**: C.1, C.2, A.1 (all complete)
**Status**: Ready to implement

## Current State Analysis

### ForwardProxy Current Structure
```rust
pub async fn start<C, S>(
    &mut self,
    mut client_transport: C,
    mut server_transport: S,
) -> Result<()>
where
    C: Transport + 'static,  // Should be IncomingTransport
    S: Transport + 'static,  // Should be OutgoingTransport
```

The ForwardProxy currently:
1. Accepts connections from clients (should use IncomingTransport)
2. Forwards requests to upstream servers (should use OutgoingTransport)
3. Uses `connect()`, `send()`, and `receive()` methods

### ReverseProxy Current Structure
Similar pattern but reversed:
- Accepts external connections (IncomingTransport)
- Routes to internal services (OutgoingTransport)

## Required Changes

### 1. Update ForwardProxy (2h)

#### Change Signatures
```rust
// Before
pub async fn start<C, S>(
    &mut self,
    mut client_transport: C,
    mut server_transport: S,
) -> Result<()>
where
    C: Transport + 'static,
    S: Transport + 'static,

// After
pub async fn start<C, S>(
    &mut self,
    mut client_transport: C,
    mut server_transport: S,
) -> Result<()>
where
    C: IncomingTransport + 'static,
    S: OutgoingTransport + 'static,
```

#### Update Method Calls
```rust
// Client side (IncomingTransport)
client_transport.connect() → client_transport.accept()
client_transport.receive() → client_transport.receive_request()
client_transport.send() → client_transport.send_response()

// Server side (OutgoingTransport)
server_transport.connect() → server_transport.connect()
server_transport.send() → server_transport.send_request()
server_transport.receive() → server_transport.receive_response()
```

#### Files to Modify
- `src/proxy/forward.rs`
  - Line 112-120: Update start() signature
  - Line 123-128: Update connection logic
  - Line 717-726: Update run_with_shutdown() signature
  - Lines with send/receive: Update all message flow

### 2. Update ReverseProxy (2h)

Similar changes but with appropriate transport directions:
- External connections use IncomingTransport
- Internal routing uses OutgoingTransport

#### Files to Modify
- `src/proxy/reverse.rs`
  - Update all Transport references
  - Update method signatures
  - Update message flow logic

### 3. Update Transport Factory (1h)

Create factory methods to instantiate directional transports:

```rust
// src/transport/factory.rs
pub fn create_incoming_stdio() -> Result<Box<dyn IncomingTransport>> {
    Ok(Box::new(StdioIncoming::new()))
}

pub fn create_outgoing_subprocess(cmd: String) -> Result<Box<dyn OutgoingTransport>> {
    Ok(Box::new(SubprocessOutgoing::new(cmd)))
}

pub fn create_incoming_http(bind: &str) -> Result<Box<dyn IncomingTransport>> {
    HttpServerIncoming::new(bind).map(|t| Box::new(t) as Box<dyn IncomingTransport>)
}

pub fn create_outgoing_http(url: String) -> Result<Box<dyn OutgoingTransport>> {
    HttpClientOutgoing::new(url).map(|t| Box::new(t) as Box<dyn OutgoingTransport>)
}
```

### 4. Update CLI Integration (1h)

Update `src/main.rs` to use the new factory methods and directional transports.

## Migration Strategy

### Phase 1: Add Compatibility Layer
1. Create adapter types that implement Transport using IncomingTransport/OutgoingTransport
2. This allows gradual migration without breaking everything

### Phase 2: Update Proxies
1. Update ForwardProxy first (most used)
2. Update ReverseProxy
3. Update tests

### Phase 3: Remove Old Code
1. Remove Transport trait implementations from directional types
2. Remove compatibility adapters
3. Clean up imports

## Testing Plan

### Unit Tests
- Test proxy with mock IncomingTransport and OutgoingTransport
- Verify message flow direction is correct
- Test error handling

### Integration Tests
```rust
#[tokio::test]
async fn test_forward_proxy_with_directional_transports() {
    let incoming = StdioIncoming::new();
    let outgoing = SubprocessOutgoing::new("echo test".to_string());
    
    let mut proxy = ForwardProxy::new();
    // Test proxy operations
}
```

## Risks and Mitigations

### Risk 1: Breaking Existing Functionality
**Mitigation**: Keep old Transport trait temporarily, use adapters

### Risk 2: Complex Message Flow Changes
**Mitigation**: Careful mapping of old methods to new ones, extensive testing

### Risk 3: Session Management Issues
**Mitigation**: Session ID mutability already added (A.1)

## Success Criteria

- [ ] ForwardProxy uses IncomingTransport for client connections
- [ ] ForwardProxy uses OutgoingTransport for server connections
- [ ] ReverseProxy uses appropriate transport directions
- [ ] All existing tests pass
- [ ] New integration tests for directional transports
- [ ] No performance regression
- [ ] Clean removal of old Transport usage

## Implementation Order

1. **Create adapters** (30 min)
   - Transport → IncomingTransport adapter
   - Transport → OutgoingTransport adapter

2. **Update ForwardProxy** (2h)
   - Change signatures
   - Update message flow
   - Test with adapters

3. **Update ReverseProxy** (2h)
   - Similar changes
   - Test with adapters

4. **Create factory methods** (30 min)
   - Centralized transport creation
   - Type-safe builders

5. **Update CLI** (30 min)
   - Use new factory methods
   - Update command parsing

6. **Remove old code** (30 min)
   - Remove adapters
   - Clean up imports
   - Final testing

## Notes

- This is a breaking change for the internal API
- External CLI interface remains unchanged
- Consider feature flag for gradual rollout
- Document the new transport model thoroughly

## Commands to Run

```bash
# Before starting
cargo test --lib  # Baseline: 865 tests passing

# After each major change
cargo test transport::directional
cargo test proxy::

# Final validation
cargo clippy --all-targets -- -D warnings
cargo test
```