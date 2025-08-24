# Test-to-Requirement Coverage Matrix

## Critical Finding

**mcp-validator provides insufficient coverage of MCP specification requirements.**

## Coverage Summary

| Source | Count | Description |
|--------|-------|-------------|
| **MCP Spec Requirements** | 233 | Official requirements from 2025-03-26 spec |
| **mcp-validator Tests** | 54 | Test functions found in validator |
| **Coverage Gap** | ~179 | Requirements without explicit test coverage |
| **Coverage Percentage** | ~23% | Rough estimate of spec coverage |

## Detailed Gap Analysis

### 1. Lifecycle Requirements (14 total)

| Requirement | Spec Reference | mcp-validator Coverage | Gap |
|-------------|---------------|------------------------|-----|
| Initialization MUST be first | lifecycle.mdx:40 | ✅ test_initialization | - |
| Client MUST send initialize | lifecycle.mdx:47 | ✅ test_initialization | - |
| Initialize MUST NOT be batch | lifecycle.mdx:74 | ❌ Not tested | 🔴 |
| Server MUST respond with capabilities | lifecycle.mdx:80 | ✅ test_server_capabilities | - |
| Client MUST send initialized | lifecycle.mdx:110 | ⚠️ Partial | Needs explicit test |
| Version negotiation logic | lifecycle.mdx:130-137 | ⚠️ test_protocol_version_negotiation | Missing edge cases |
| Timeout handling | lifecycle.mdx:201-210 | ❌ Not tested | 🔴 |
| Shutdown sequence | lifecycle.mdx:183-191 | ⚠️ test_shutdown_sequence | Missing stdio specifics |

**Coverage: 4/14 (29%)**

### 2. Transport Requirements (28 total)

| Requirement | Spec Reference | mcp-validator Coverage | Gap |
|-------------|---------------|------------------------|-----|
| UTF-8 encoding MUST | transports.mdx:7 | ❌ Not tested | 🔴 |
| stdio no embedded newlines | transports.mdx:30 | ❌ Not tested | 🔴 |
| Server MUST NOT write non-MCP to stdout | transports.mdx:33 | ❌ Not tested | 🔴 |
| Origin header validation MUST | transports.mdx:78 | ❌ Not tested | 🔴 SECURITY |
| Accept header requirements | transports.mdx:90-91 | ❌ Not tested | 🔴 |
| SSE stream requirements | transports.mdx:109-127 | ❌ Not tested | 🔴 |
| 202 Accepted for notifications | transports.mdx:100-101 | ❌ Not tested | 🔴 |
| Session ID handling | transports.mdx | ⚠️ test_http_session_management | Basic only |

**Coverage: 1/28 (4%)** 🚨

### 3. Message Format Requirements (15 total)

| Requirement | Spec Reference | mcp-validator Coverage | Gap |
|-------------|---------------|------------------------|-----|
| JSON-RPC 2.0 format | protocol | ✅ test_request_format | - |
| Request ID uniqueness | protocol | ✅ test_unique_request_ids | - |
| Response ID matching | protocol | ✅ test_response_format | - |
| Error format | protocol | ⚠️ test_error_handling | Missing codes |
| Batch support (2025-03-26) | protocol | ✅ test_jsonrpc_batch_support | - |
| No batch (2025-06-18) | protocol | ⚠️ test_batch_request_rejection | Version-specific |
| Notification format | protocol | ✅ test_notification_format | - |

**Coverage: 6/15 (40%)**

### 4. Tools Requirements (25 total)

| Requirement | Spec Reference | mcp-validator Coverage | Gap |
|-------------|---------------|------------------------|-----|
| tools/list format | tools.mdx | ✅ test_tools_list | - |
| Tool schema validation | tools.mdx | ⚠️ test_tool_schema_validation | Basic only |
| Parameter validation | tools.mdx | ✅ test_tool_with_invalid_params | - |
| Async tool support | tools.mdx | ✅ test_async_tool_support | - |
| Tool cancellation | tools.mdx | ⚠️ test_async_tool_cancellation | Limited |
| Structured output (2025-06-18) | tools.mdx | ⚠️ test_structured_tool_output | Version-specific |
| Error handling | tools.mdx | ⚠️ test_tool_functionality | Basic only |

**Coverage: 7/25 (28%)**

### 5. Security Requirements (8 total)

| Requirement | Spec Reference | mcp-validator Coverage | Gap |
|-------------|---------------|------------------------|-----|
| Origin validation MUST | transports.mdx:78 | ❌ Not tested | 🔴 CRITICAL |
| Bind to localhost SHOULD | transports.mdx:79 | ❌ Not tested | 🔴 |
| Authentication SHOULD | transports.mdx:80 | ❌ test_authorization_requirements | Minimal |
| DNS rebinding protection | transports.mdx:78 | ❌ Not tested | 🔴 CRITICAL |
| Token handling | auth | ❌ Not tested | 🔴 |

**Coverage: 0/8 (0%)** 🚨

### 6. Resources (12 total)

| Requirement | Spec Reference | mcp-validator Coverage | Gap |
|-------------|---------------|------------------------|-----|
| resources/list format | resources.mdx | ⚠️ Minimal in test_resources_capability | 🔴 |
| Resource URI validation | resources.mdx | ⚠️ test_resource_uri_validation | Basic |
| Subscription management | resources.mdx | ❌ Not tested | 🔴 |
| Resource metadata (2025-06-18) | resources.mdx | ⚠️ test_resource_metadata_support | Version-specific |

**Coverage: 2/12 (17%)**

### 7. Error Handling (10 total)

| Requirement | Spec Reference | mcp-validator Coverage | Gap |
|-------------|---------------|------------------------|-----|
| Standard error codes | protocol | ⚠️ test_error_handling | Incomplete |
| Parse error -32700 | protocol | ❌ Not tested | 🔴 |
| Invalid Request -32600 | protocol | ❌ Not tested | 🔴 |
| Method not found -32601 | protocol | ⚠️ test_invalid_method | Basic |
| Invalid params -32602 | protocol | ⚠️ In various tests | Not explicit |

**Coverage: 2/10 (20%)**

## Missing Test Categories

### Completely Missing from mcp-validator

1. **Transport Compliance** (0% coverage)
   - stdio message framing
   - HTTP header requirements
   - SSE stream management
   - Connection handling

2. **Security Testing** (0% coverage)
   - Origin validation
   - DNS rebinding protection
   - Authentication flows
   - Token handling

3. **Session Management** (minimal coverage)
   - Session ID format
   - Session persistence
   - Cleanup and timeouts

4. **Negative Testing** (minimal)
   - Malformed JSON
   - Invalid message sequences
   - Resource exhaustion
   - Timeout scenarios

5. **Proxy-Specific** (0% coverage)
   - Message forwarding
   - Dual session tracking
   - Error propagation
   - Connection pooling

## Required Test Count Estimation

### Based on Spec Requirements

| Category | Requirements | Tests Needed | mcp-validator Has |
|----------|-------------|--------------|-------------------|
| Lifecycle | 14 | 20-25 | 4 |
| Transport | 28 | 35-40 | 1 |
| Message Format | 15 | 20-25 | 6 |
| Tools | 25 | 30-35 | 7 |
| Resources | 12 | 15-20 | 2 |
| Security | 8 | 15-20 | 0 |
| Error Handling | 10 | 15-20 | 2 |
| Proxy-Specific | 28 | 35-40 | 0 |
| **Total** | **140** | **185-220** | **22** |

**Actual Coverage: ~12% of needed tests** 🚨

## Recommendations

### 1. Don't Rely on mcp-validator Tests Alone
- Use them as **reference implementations** only
- They provide good **test scenarios** but poor **compliance coverage**
- Missing 80%+ of specification requirements

### 2. Build Comprehensive Test Suite
Priority order:
1. **Security requirements** (currently 0% coverage)
2. **Transport compliance** (currently 4% coverage)  
3. **Error handling** (currently 20% coverage)
4. **Session management** (minimal coverage)
5. **Proxy-specific** (currently 0% coverage)

### 3. Test Design Approach
For each requirement:
- Positive test (correct behavior)
- Negative test (error handling)
- Edge cases (boundary conditions)
- Version-specific variations

### 4. Example: Proper Test Coverage

For requirement "Initialize MUST NOT be batch":

```rust
// Positive test
#[test]
fn test_initialize_not_in_batch_accepted() {
    // Send initialize alone - should succeed
}

// Negative test  
#[test]
fn test_initialize_in_batch_rejected() {
    // Send initialize in batch - should fail
}

// Edge case
#[test]
fn test_initialize_with_other_requests_rejected() {
    // Send [initialize, tools/list] - should fail
}
```

## Conclusion

**mcp-validator is inadequate for compliance testing.** We need to:

1. Create ~200 tests based on actual spec requirements
2. Ensure 100% coverage of MUST requirements
3. Add comprehensive negative testing
4. Include proxy-specific scenarios
5. Build version-aware test framework

The validator tests can serve as **implementation examples** but should not be our primary compliance validation.

---

*Analysis Date: 2025-08-23*
*Finding: mcp-validator covers only ~12% of required compliance tests*
*Recommendation: Build comprehensive test suite from specifications*