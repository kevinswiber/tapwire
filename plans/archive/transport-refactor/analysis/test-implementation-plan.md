# Test Implementation Plan for Directional Transports

## Current State
- **0 tests** for directional transports (IncomingTransport/OutgoingTransport)
- **0 tests** for raw transports (StdioRaw, HttpRaw, etc.)
- 795 unit tests passing for other components
- Critical safety features (size limits, validation) are untested

## Implementation Priority

### Phase 1: Critical Safety Tests (HIGH PRIORITY)

#### 1. Message Size Limits
**File**: `shadowcat/src/transport/directional/tests/size_limits.rs`
```rust
- test_incoming_message_too_large()
- test_incoming_message_within_limit()
- test_outgoing_message_too_large()
- test_outgoing_message_within_limit()
```

#### 2. Process Lifecycle Management  
**File**: `shadowcat/src/transport/raw/tests/subprocess.rs`
```rust
- test_subprocess_spawn_and_terminate()
- test_subprocess_cleanup_on_drop()
- test_subprocess_handle_crash()
- test_subprocess_timeout()
```

### Phase 2: Configuration Validation (MEDIUM PRIORITY)

#### 3. Constructor Validation
**File**: `shadowcat/src/transport/directional/tests/validation.rs`
```rust
- test_subprocess_empty_command_rejected()
- test_http_invalid_url_rejected()
- test_zero_timeout_rejected()
- test_negative_buffer_size_rejected()
```

### Phase 3: Integration Tests (MEDIUM PRIORITY)

#### 4. End-to-End Transport Tests
**File**: `shadowcat/tests/integration_directional_transports.rs`
```rust
- test_stdio_incoming_to_subprocess_outgoing()
- test_http_server_incoming_to_http_client_outgoing()
- test_message_flow_with_session_id()
- test_transport_reconnection()
```

#### 5. SSE/Streaming Tests
**File**: `shadowcat/tests/integration_streamable_http.rs`
```rust
- test_streamable_http_connection()
- test_sse_event_stream_processing()
- test_http_to_sse_mode_switch()
```

## Test Infrastructure Needs

### 1. Mock Implementations
Create mock versions of raw transports for testing:
- `MockRawIncoming`
- `MockRawOutgoing`
- `MockProtocolHandler`

### 2. Test Utilities
- Message factory functions
- Transport builder helpers
- Assertion helpers for envelopes

### 3. Test Fixtures
- Sample MCP messages
- Invalid message payloads
- Large message payloads for size testing

## Implementation Order

1. **Start with Phase 1** - Safety critical tests
2. **Add mock infrastructure** as needed
3. **Phase 2** - Validation tests
4. **Phase 3** - Integration tests
5. **Document** any features confirmed as removed

## Files to Create

```
shadowcat/
├── src/
│   └── transport/
│       ├── directional/
│       │   └── tests/
│       │       ├── mod.rs
│       │       ├── size_limits.rs
│       │       └── validation.rs
│       └── raw/
│           └── tests/
│               ├── mod.rs
│               └── subprocess.rs
└── tests/
    ├── integration_directional_transports.rs
    └── integration_streamable_http.rs
```

## Success Metrics
- [ ] All critical safety issues covered with tests
- [ ] No panics possible from invalid input
- [ ] Process cleanup verified
- [ ] Message size limits enforced
- [ ] 100% coverage of public API surface