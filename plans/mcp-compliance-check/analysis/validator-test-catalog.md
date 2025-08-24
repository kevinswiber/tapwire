# mcp-validator Test Catalog

## Overview

This document catalogs all 54 test cases extracted from mcp-validator, organized by category and annotated with purpose, protocol version applicability, and implementation notes for our Rust compliance framework.

## Test Categories Summary

| Category | Test Count | Description |
|----------|------------|-------------|
| Base Protocol | 2 | Core initialization and capability negotiation |
| Tools | 11 | Tool discovery, invocation, and validation |
| Async Operations | 7 | Async tool support (2025-03-26+) |
| Specification Coverage | 19 | Protocol compliance validation |
| Version-Specific | 10 | Features specific to certain protocol versions |
| HTTP Transport | 2 | HTTP-specific session and transport tests |
| Resources | 3 | Resource management (2025-06-18) |
| Total | 54 | Complete test suite |

## Detailed Test Catalog

### Category: Base Protocol

#### Test: test_initialization
**Purpose**: Validates basic initialization flow
**Protocol Versions**: All (2024-11-05, 2025-03-26, 2025-06-18)
**Key Validations**:
- Protocol version was negotiated
- Server capabilities were received
- Server info was received (optional)

**Implementation Notes**:
- Foundation test - must pass before other tests
- Test both forward and reverse proxy modes
- Add timeout handling (not in original)

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "init",
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-03-26",
    "capabilities": {},
    "clientInfo": {"name": "test_client", "version": "1.0.0"}
  }
}
```

#### Test: test_server_capabilities
**Purpose**: Verify server advertises required capabilities
**Protocol Versions**: All
**Key Validations**:
- Capabilities match protocol version
- Required capabilities are present
- Capabilities format is correct (boolean vs object)

**Implementation Notes**:
- Version-specific validation logic needed
- 2024-11-05: Boolean capability values
- 2025-03-26+: Object capability values

---

### Category: Tools

#### Test: test_tools_list
**Purpose**: Validate tools/list returns proper format
**Protocol Versions**: All
**Key Validations**:
- Response is array
- Each tool has name and description
- Schema format matches protocol version (inputSchema vs parameters)

**Implementation Notes**:
- Cache tool list for subsequent tests
- Validate schema structure based on version

#### Test: test_tool_functionality
**Purpose**: Test calling an available tool
**Protocol Versions**: All
**Key Validations**:
- Tool accepts valid parameters
- Response format is correct
- Error handling for auth/rate limits

**Implementation Notes**:
- Use parameter generation logic from validator
- Handle authentication errors gracefully
- Consider rate limiting in test design

#### Test: test_tool_with_invalid_params
**Purpose**: Verify parameter validation
**Protocol Versions**: All
**Key Validations**:
- Missing required parameters rejected
- Type validation enforced
- Appropriate error messages returned

**Implementation Notes**:
- Test both missing and wrong-type parameters
- Some servers may be lenient with type conversion

#### Test: test_jsonrpc_batch_support
**Purpose**: Verify JSON-RPC batch handling
**Protocol Versions**: 2024-11-05, 2025-03-26 (removed in 2025-06-18)
**Key Validations**:
- Server accepts batch requests
- Correct number of responses
- Each response has result or error

**Implementation Notes**:
- Skip for 2025-06-18 (batching removed)
- Handle transport limitations
- Use timeout for batch operations

#### Test: test_invalid_tool_name
**Purpose**: Test calling non-existent tool
**Protocol Versions**: All
**Key Validations**:
- Returns appropriate error
- Error code indicates tool not found
- Server remains responsive

#### Test: test_invalid_tool_arguments
**Purpose**: Test malformed arguments
**Protocol Versions**: All
**Key Validations**:
- Rejects invalid JSON structure
- Clear error messages
- No server crash

#### Test: test_tool_schema_validation
**Purpose**: Validate tool input schemas
**Protocol Versions**: All
**Key Validations**:
- Schema follows JSON Schema spec
- Required fields marked correctly
- Type definitions valid

#### Test: test_structured_tool_output
**Purpose**: Test structured output with schemas
**Protocol Versions**: 2025-06-18 only
**Key Validations**:
- Output matches outputSchema
- structuredContent field present
- isError field included

#### Test: test_each_tool (Dynamic)
**Purpose**: Iteratively test each available tool
**Protocol Versions**: All
**Key Validations**:
- Each tool callable
- Parameters accepted
- Response format consistent

#### Test: test_dynamic_tool_discovery
**Purpose**: Test runtime tool discovery
**Protocol Versions**: All
**Key Validations**:
- Tools discoverable after init
- Tool list can change
- New tools immediately callable

---

### Category: Async Operations (2025-03-26+)

#### Test: test_async_tool_support
**Purpose**: Verify async capability advertisement
**Protocol Versions**: 2025-03-26, 2025-06-18
**Key Validations**:
- asyncSupported in capabilities
- Protocol adapter supports async calls
- Version check correct

#### Test: test_async_echo_tool
**Purpose**: Test basic async tool execution
**Protocol Versions**: 2025-03-26, 2025-06-18
**Key Validations**:
- Returns operation ID
- Status tracking works
- Result retrieval successful

#### Test: test_async_long_running_tool
**Purpose**: Test long-running async operations
**Protocol Versions**: 2025-03-26, 2025-06-18
**Key Validations**:
- Operation continues in background
- Status updates available
- Timeout handling

#### Test: test_async_tool_cancellation
**Purpose**: Test canceling async operations
**Protocol Versions**: 2025-03-26, 2025-06-18
**Key Validations**:
- Cancellation accepted
- Status reflects cancellation
- Resources cleaned up

#### Test: test_async_tools_capability
**Purpose**: Verify async tools listing
**Protocol Versions**: 2025-03-26, 2025-06-18
**Key Validations**:
- Async tools marked appropriately
- Capability negotiation correct
- Fallback to sync if needed

#### Test: test_dynamic_async_support
**Purpose**: Dynamic async tool detection
**Protocol Versions**: 2025-03-26, 2025-06-18
**Key Validations**:
- Runtime async detection
- Proper version handling
- Graceful degradation

#### Test: test_dynamic_async_cancellation
**Purpose**: Dynamic cancellation testing
**Protocol Versions**: 2025-03-26, 2025-06-18
**Key Validations**:
- All async ops cancelable
- Consistent cancellation behavior
- Error handling

---

### Category: Specification Coverage

#### Test: test_request_format
**Purpose**: Validate JSON-RPC 2.0 request format
**Protocol Versions**: All
**Key Validations**:
- jsonrpc: "2.0" field present
- ID is string or integer (not null)
- method field is string

#### Test: test_response_format
**Purpose**: Validate JSON-RPC 2.0 response format
**Protocol Versions**: All
**Key Validations**:
- ID matches request
- Has result XOR error
- Error has code and message

#### Test: test_unique_request_ids
**Purpose**: Test request ID handling
**Protocol Versions**: All
**Key Validations**:
- Each ID gets unique response
- Duplicate IDs handled correctly
- ID tracking works

#### Test: test_notification_format
**Purpose**: Test notifications (no ID)
**Protocol Versions**: All
**Key Validations**:
- Notifications don't get responses
- Server processes notifications
- No ID in notification

#### Test: test_error_handling
**Purpose**: Test error response format
**Protocol Versions**: All
**Key Validations**:
- Error codes follow spec
- Error messages descriptive
- Data field optional but valid

#### Test: test_initialization_order
**Purpose**: Test initialization sequencing
**Protocol Versions**: All
**Key Validations**:
- Initialize before other requests
- Initialized notification accepted
- State tracking correct

#### Test: test_initialization_negotiation
**Purpose**: Test version negotiation
**Protocol Versions**: All
**Key Validations**:
- Version downgrade supported
- Capabilities adjusted
- Negotiation result valid

#### Test: test_protocol_version_negotiation
**Purpose**: Test cross-version compatibility
**Protocol Versions**: All
**Key Validations**:
- Handles version mismatches
- Falls back appropriately
- Clear version in response

#### Test: test_shutdown_sequence
**Purpose**: Test graceful shutdown
**Protocol Versions**: All
**Key Validations**:
- Shutdown request accepted
- Cleanup performed
- No further requests accepted

#### Test: test_server_info_requirements
**Purpose**: Validate server/info response
**Protocol Versions**: All
**Key Validations**:
- Name and version present
- Additional fields allowed
- Format consistent

#### Test: test_capability_declaration
**Purpose**: Test capability format
**Protocol Versions**: All
**Key Validations**:
- Format matches version
- All capabilities documented
- No unknown capabilities

#### Test: test_parallel_requests
**Purpose**: Test concurrent request handling
**Protocol Versions**: All
**Key Validations**:
- Multiple requests processed
- Responses matched correctly
- No race conditions

#### Test: test_versioning_requirements
**Purpose**: Test version headers/fields
**Protocol Versions**: All
**Key Validations**:
- MCP-Protocol-Version header
- Version in messages
- Consistent versioning

#### Test: test_authorization_requirements
**Purpose**: Test auth handling
**Protocol Versions**: All
**Key Validations**:
- Auth headers processed
- Unauthorized errors correct
- Token handling secure

#### Test: test_logging_capability
**Purpose**: Test logging features
**Protocol Versions**: 2025-03-26+
**Key Validations**:
- Logging methods available
- Log levels supported
- Structured logging works

#### Test: test_prompts_capability
**Purpose**: Test prompts feature
**Protocol Versions**: 2025-03-26+
**Key Validations**:
- Prompts listing works
- Prompt execution successful
- Arguments handled

#### Test: test_resources_capability
**Purpose**: Test resources feature
**Protocol Versions**: 2025-06-18
**Key Validations**:
- Resource listing works
- Resource fetching successful
- Subscriptions supported

#### Test: test_tools_capability
**Purpose**: Test tools capability declaration
**Protocol Versions**: All
**Key Validations**:
- Capability matches reality
- Tools accessible if declared
- Proper capability format

#### Test: test_workspace_configuration
**Purpose**: Test workspace config
**Protocol Versions**: 2025-06-18
**Key Validations**:
- Config methods available
- Settings persisted
- Validation works

---

### Category: Version-Specific Features

#### Test: test_2025_06_18 (Suite)
**Purpose**: Test 2025-06-18 specific features
**Protocol Versions**: 2025-06-18 only
**Key Validations**:
- Structured output
- Elicitation support
- No batch support
- Enhanced validation

#### Test: test_elicitation_support
**Purpose**: Test requesting user information
**Protocol Versions**: 2025-06-18
**Key Validations**:
- Elicitation requests valid
- User responses handled
- Context maintained

#### Test: test_batch_request_rejection
**Purpose**: Verify batching removed
**Protocol Versions**: 2025-06-18
**Key Validations**:
- Batch requests rejected
- Clear error message
- Single requests work

#### Test: test_enhanced_tool_validation
**Purpose**: Test stricter validation
**Protocol Versions**: 2025-06-18
**Key Validations**:
- Strict type checking
- Schema enforcement
- Better error messages

#### Test: test_enhanced_ping_validation
**Purpose**: Test ping enhancements
**Protocol Versions**: 2025-06-18
**Key Validations**:
- Ping format strict
- Timing information
- Health checks

#### Test: test_prompt_arguments_validation
**Purpose**: Test prompt arg validation
**Protocol Versions**: 2025-06-18
**Key Validations**:
- Argument types enforced
- Required args checked
- Defaults applied

#### Test: test_resource_metadata_support
**Purpose**: Test resource metadata
**Protocol Versions**: 2025-06-18
**Key Validations**:
- Metadata fields present
- MIME types correct
- Timestamps valid

#### Test: test_resource_uri_validation
**Purpose**: Test URI validation
**Protocol Versions**: 2025-06-18
**Key Validations**:
- URI format enforced
- Schemes supported
- Templates work

#### Test: test_cancellation_validation
**Purpose**: Test cancellation requirements
**Protocol Versions**: 2025-06-18
**Key Validations**:
- All ops cancelable
- Clean cancellation
- State consistency

#### Test: test_async_tool_calls_validation
**Purpose**: Enhanced async validation
**Protocol Versions**: 2025-06-18
**Key Validations**:
- Stricter async checks
- Progress reporting
- Error recovery

---

### Category: HTTP Transport

#### Test: test_http_transport_requirements
**Purpose**: Test HTTP-specific requirements
**Protocol Versions**: All
**Key Validations**:
- Session ID handling
- Header requirements
- SSE support for streamable

#### Test: test_http_session_management
**Purpose**: Test HTTP session lifecycle
**Protocol Versions**: All
**Key Validations**:
- Session creation
- Session persistence
- Session cleanup

---

### Category: stdio Transport

#### Test: test_stdio_transport_requirements
**Purpose**: Test stdio-specific requirements
**Protocol Versions**: All
**Key Validations**:
- Message framing
- Binary safety
- Stream handling

---

## Proxy-Specific Test Gaps

The following test scenarios are missing from mcp-validator but essential for proxy testing:

### Session Management
1. **test_dual_session_tracking** - Verify client and upstream session mapping
2. **test_session_persistence** - Test session recovery after disconnect
3. **test_session_timeout** - Verify proper timeout handling
4. **test_session_cleanup** - Ensure resources freed on termination

### Multi-Upstream
5. **test_upstream_failover** - Test switching to backup upstream
6. **test_load_balancing** - Verify request distribution
7. **test_upstream_health_checks** - Test health monitoring
8. **test_sticky_sessions** - Ensure session affinity

### Connection Pooling
9. **test_connection_reuse** - Verify connection pooling works
10. **test_pool_limits** - Test maximum connection limits
11. **test_idle_timeout** - Verify idle connection cleanup
12. **test_pool_exhaustion** - Handle pool exhaustion gracefully

### Security & Auth
13. **test_oauth_token_forwarding** - Verify token handling
14. **test_token_refresh** - Test automatic token refresh
15. **test_auth_header_sanitization** - Ensure no token leakage
16. **test_rate_limiting** - Verify rate limit enforcement

### SSE Specific
17. **test_sse_reconnection** - Test SSE auto-reconnect
18. **test_sse_heartbeat** - Verify keepalive handling
19. **test_sse_buffering** - Test message buffering
20. **test_sse_compression** - Verify compression support

### Performance
21. **test_latency_overhead** - Measure proxy latency
22. **test_throughput** - Verify message throughput
23. **test_concurrent_sessions** - Test many simultaneous sessions
24. **test_memory_usage** - Monitor memory per session

### Error Recovery
25. **test_partial_message_handling** - Handle incomplete messages
26. **test_malformed_json_recovery** - Recover from bad JSON
27. **test_upstream_error_forwarding** - Properly forward errors
28. **test_circuit_breaker** - Test circuit breaker activation

## Implementation Priority

### Phase 1: Core Tests (Must Have)
- All Base Protocol tests
- Basic Tools tests (list, call, validation)
- Core Specification Coverage tests
- Basic proxy session management

### Phase 2: Protocol Features
- Async Operations (for 2025-03-26+)
- Version-specific features
- Advanced tools testing
- Transport-specific tests

### Phase 3: Proxy-Specific
- Multi-upstream handling
- Connection pooling
- Security and auth
- SSE-specific features

### Phase 4: Performance & Resilience
- Performance benchmarks
- Error recovery
- Circuit breakers
- Load testing

## Test Execution Strategy

### Environment Setup
```rust
// Test configuration per protocol version
struct TestConfig {
    protocol_version: String,
    transport: Transport,
    upstream_url: String,
    test_filter: Option<String>,
}
```

### Test Runner Architecture
```rust
// Modular test runner
trait ComplianceTest {
    async fn run(&self, adapter: &ProtocolAdapter) -> TestResult;
    fn applicable_versions(&self) -> Vec<String>;
    fn category(&self) -> TestCategory;
}
```

### Reporting Format
```json
{
  "version": "2025-03-26",
  "transport": "http",
  "results": {
    "passed": 45,
    "failed": 5,
    "skipped": 4,
    "total": 54
  },
  "categories": {
    "base_protocol": {"passed": 2, "total": 2},
    "tools": {"passed": 9, "total": 11}
  }
}
```

## Notes on mcp-validator Implementation

### Valuable Patterns to Adopt
1. **Test Categorization** - Clear separation by functionality
2. **Version Filtering** - Skip tests for unsupported versions
3. **Parameter Generation** - Smart test value generation
4. **Graceful Degradation** - Handle missing features elegantly

### Issues to Avoid
1. **Transport Coupling** - Our tests should be transport-agnostic
2. **Initialization Bugs** - Ensure proper setup before tests
3. **SSE Handling** - Native SSE support from the start
4. **Protocol Mismatches** - Strict adherence to spec field names

## Summary

This catalog documents 54 tests from mcp-validator plus 28 proxy-specific tests we need to add, totaling 82 comprehensive compliance tests. The tests are organized by category, annotated with version requirements, and prioritized for implementation.

The Rust implementation will improve upon mcp-validator by:
- Adding proxy-specific test coverage
- Implementing proper SSE handling
- Following MCP spec field names exactly
- Supporting all transport types natively
- Providing better error messages and debugging

---

*Extracted: 2025-08-23*
*Source: mcp-validator v0.1.0*
*Target: Shadowcat MCP Compliance Framework*