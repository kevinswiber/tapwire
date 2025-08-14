# Test Coverage Gap Analysis - Transport Refactor

## Deleted Test Files Analysis

### 1. src/transport/size_limit_tests.rs
**Purpose**: Validated message size limits to prevent memory exhaustion
**Key Test Scenarios**:
- `test_stdio_send_message_too_large`: Verified StdioTransport rejects messages exceeding max_message_size
- `test_stdio_send_message_within_limit`: Verified messages within limit are accepted
- `test_http_send_message_too_large`: Same validation for HTTP transport

**Impact**: HIGH - No message size validation in new transports could lead to memory issues

### 2. src/transport/validation_test.rs  
**Purpose**: Validated transport configuration parameters
**Key Test Scenarios**:
- `test_stdio_builder_empty_command_validation`: Rejected empty command strings
- `test_http_builder_zero_timeout_validation`: Rejected zero/negative timeouts
- `test_sse_builder_zero_reconnect_validation`: Validated reconnect parameters

**Impact**: MEDIUM - Invalid configurations could cause runtime failures

### 3. tests/pause_resume_test.rs
**Purpose**: Tested pause/resume functionality for intercepted messages
**Key Components**:
- PauseController for managing paused messages
- PausingInterceptor that could pause specific methods
- Resume mechanism with timeout support

**Impact**: LOW - Feature appears to be removed entirely (no PauseController found)

### 4. tests/integration_forward_proxy_sse.rs
**Purpose**: Integration tests for SSE forward proxy scenarios
**Key Test Scenarios**:
- MockSseServer implementation
- SSE connection handling
- Event stream processing
- Session management over SSE

**Impact**: MEDIUM - SSE functionality exists in raw layer but lacks integration tests

### 5. tests/sse_interceptor_test.rs
**Purpose**: Tested message interception in SSE streams
**Key Test Scenarios**:
- ModifyingInterceptor that altered message content
- Interceptor chain with SSE transport
- Message modification during streaming

**Impact**: LOW - Interceptor functionality may have changed architecture

### 6. tests/sse_transport_test.rs
**Purpose**: Basic SSE transport functionality tests
**Key Test Scenarios**:
- SSE transport creation and configuration
- MCP message handling over SSE
- Event ID generation for correlation

**Impact**: MEDIUM - Basic SSE functionality needs coverage

### 7. tests/transport_regression_suite.rs
**Purpose**: Comprehensive regression tests for transport behavior
**Key Test Scenarios**:
- StdioTransport subprocess spawning
- Send/receive message flow
- Process lifecycle management
- Configuration application

**Impact**: HIGH - Core transport behaviors need regression coverage

## Current Architecture Assessment

### What Still Exists
1. **SSE Support**: Found in `src/transport/raw/sse.rs` and `src/transport/sse/` module
2. **Streamable HTTP**: Present in `src/transport/raw/streamable_http.rs`
3. **Buffer Pooling**: Exists in `src/transport/buffer_pool.rs`
4. **HTTP Utils**: Preserved in `src/transport/http_utils.rs`

### What's Missing
1. **Message Size Limits**: No validation found in directional transports
2. **Configuration Validation**: No builder validation for invalid parameters
3. **Pause/Resume**: Feature completely removed
4. **Integration Tests**: Limited coverage for directional transports

## Critical Gaps Requiring Tests

### Priority 1 - Safety Critical
1. **Message Size Limits** (was in size_limit_tests.rs)
   - Need to add max_message_size validation to directional transports
   - Prevent memory exhaustion attacks
   - Location: `src/transport/directional/incoming.rs` and `outgoing.rs`

2. **Process Lifecycle** (was in transport_regression_suite.rs)
   - Verify SubprocessOutgoing properly spawns/terminates processes
   - Test cleanup on drop/error
   - Location: `src/transport/raw/subprocess.rs`

### Priority 2 - Functionality
3. **Configuration Validation** (was in validation_test.rs)
   - Validate construction parameters (empty commands, zero timeouts)
   - Return proper errors instead of panicking
   - Location: Directional transport constructors

4. **SSE/Streaming Integration** (was in multiple SSE test files)
   - Test StreamableHttpRawTransport end-to-end
   - Verify event stream processing
   - Location: `src/transport/raw/streamable_http.rs`

### Priority 3 - Nice to Have
5. **Interceptor Integration** (was in sse_interceptor_test.rs)
   - May need redesign for new architecture
   - Lower priority if interceptor pattern changed

6. **Pause/Resume** (was in pause_resume_test.rs)
   - Feature appears removed - confirm if needed

## Recommendations

### Immediate Actions
1. Add message size limit validation to directional transports
2. Create subprocess lifecycle tests for SubprocessOutgoing
3. Add configuration validation tests

### Follow-up Actions
4. Create SSE integration tests using raw transports
5. Document removed features (pause/resume) in migration guide
6. Consider if interceptor pattern needs updating

## Test Location Mapping

| Old Test | New Location | Priority |
|----------|--------------|----------|
| size_limit_tests.rs | `src/transport/directional/tests/size_limits.rs` | HIGH |
| validation_test.rs | `src/transport/directional/tests/validation.rs` | MEDIUM |
| transport_regression_suite.rs | `tests/integration_directional_transports.rs` | HIGH |
| integration_forward_proxy_sse.rs | `tests/integration_streamable_http.rs` | MEDIUM |
| pause_resume_test.rs | N/A - Feature removed | N/A |
| sse_interceptor_test.rs | TBD - Depends on interceptor redesign | LOW |
| sse_transport_test.rs | `src/transport/raw/tests/sse.rs` | MEDIUM |