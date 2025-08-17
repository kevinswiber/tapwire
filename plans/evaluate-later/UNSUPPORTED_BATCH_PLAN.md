# Unsupported Batch Request Handling Plan

## Overview

This document outlines the plan for properly handling unsupported batch requests in Shadowcat, ensuring we return appropriate JSON-RPC 2.0 compliant error responses rather than transport-level errors.

## Current State Analysis

### What We Have Now

1. **Batch Detection**: We detect batch requests (JSON arrays) in:
   - `StdioTransport::parse_message()` - Returns `TransportError::ProtocolError`
   - `ReverseProxy::handle_mcp_request()` - Returns `ReverseProxyError::InvalidHeaders`

2. **Error Messages**: 
   - Current: "Batch messages are not supported. Please send individual JSON-RPC messages."
   - Problem: These are transport/HTTP errors, not JSON-RPC errors

### What We Need

Per JSON-RPC 2.0 specification, we should return a proper JSON-RPC error response:
```json
{
  "jsonrpc": "2.0",
  "id": null,
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": "Batch requests are not supported. Please send individual JSON-RPC messages."
  }
}
```

## Investigation Tasks

### 1. Current Error Handling Analysis

**Investigate**:
- [ ] How are transport errors currently converted to responses?
- [ ] Do we properly return JSON-RPC errors for other error conditions?
- [ ] What error codes are we using throughout the codebase?
- [ ] Are we following the reserved error code ranges?

**Files to examine**:
- `src/error.rs` - Error type definitions
- `src/transport/stdio.rs` - How errors become responses
- `src/proxy/reverse.rs` - HTTP error response generation
- `src/transport/http_mcp.rs` - JSON-RPC error handling

### 2. JSON-RPC Error Compliance Audit

**Check if we properly handle**:
- [ ] Parse errors (-32700): Invalid JSON
- [ ] Invalid Request (-32600): Not a valid Request object (includes batches)
- [ ] Method not found (-32601): Unknown method
- [ ] Invalid params (-32602): Bad parameters
- [ ] Internal error (-32603): Server errors
- [ ] Custom errors (-32000 to -32099): Our implementation-specific errors

### 3. Batch Request Specifics

**Questions to answer**:
- Can we extract IDs from batch requests to return errors for each?
- Should we return a single error or one per request in the batch?
- How do other MCP implementations handle this?

## Implementation Plan

### Phase 1: Investigation (2 hours)

1. **Analyze Current Error Handling**
   ```bash
   # Find all error code usage
   rg "32700|32600|32601|32602|32603|32000" --type rust
   
   # Find error response generation
   rg "error.*code.*message" --type rust
   
   # Check JSON-RPC error creation
   rg "jsonrpc.*error|error.*jsonrpc" --type rust -A 5
   ```

2. **Document Findings**
   - Create error handling flow diagram
   - List all error codes currently in use
   - Identify gaps in JSON-RPC compliance

### Phase 2: Design (1 hour)

1. **Define Error Response Strategy**
   - Batch requests → Single error response with code -32600
   - Include helpful data field explaining batch non-support
   - Ensure consistency across transports

2. **Create Error Constants**
   ```rust
   // src/error/json_rpc.rs
   pub mod json_rpc_errors {
       pub const PARSE_ERROR: i32 = -32700;
       pub const INVALID_REQUEST: i32 = -32600;
       pub const METHOD_NOT_FOUND: i32 = -32601;
       pub const INVALID_PARAMS: i32 = -32602;
       pub const INTERNAL_ERROR: i32 = -32603;
       
       // Custom errors (-32000 to -32099)
       pub const BATCH_NOT_SUPPORTED: i32 = -32000;
   }
   ```

### Phase 3: Implementation (3 hours)

1. **Create JSON-RPC Error Builder**
   ```rust
   impl JsonRpcError {
       pub fn invalid_request(message: &str, data: Option<Value>) -> Value {
           json!({
               "jsonrpc": "2.0",
               "id": null,
               "error": {
                   "code": -32600,
                   "message": message,
                   "data": data
               }
           })
       }
       
       pub fn batch_not_supported() -> Value {
           Self::invalid_request(
               "Batch requests not supported",
               Some(json!({
                   "reason": "Shadowcat does not implement MCP batch support",
                   "suggestion": "Send individual JSON-RPC messages",
                   "spec_note": "Batching is optional per MCP 2025-03-26 and removed in later versions"
               }))
           )
       }
   }
   ```

2. **Update StdioTransport**
   - Instead of returning TransportError, write JSON-RPC error to stdout
   - Ensure process continues running after batch rejection

3. **Update ReverseProxy**
   - Return proper JSON-RPC error response with correct HTTP status (200 OK)
   - Set appropriate content-type headers

4. **Update HTTP-MCP Transport**
   - Ensure consistent error handling

### Phase 4: Testing (2 hours)

1. **Unit Tests**
   - Test batch detection returns proper JSON-RPC error
   - Test error has correct structure and codes
   - Test other JSON-RPC errors are properly formatted

2. **Integration Tests**
   - Send batch request via stdio → receive JSON-RPC error
   - Send batch request via HTTP → receive JSON-RPC error
   - Verify client can parse error response

3. **Manual Testing**
   ```bash
   # Test stdio transport
   echo '[{"jsonrpc":"2.0","id":1,"method":"test"}]' | shadowcat forward stdio -- server
   
   # Test HTTP transport
   curl -X POST http://localhost:8080/mcp \
     -H "Content-Type: application/json" \
     -d '[{"jsonrpc":"2.0","id":1,"method":"test"}]'
   ```

## Success Criteria

1. **JSON-RPC Compliance**
   - [ ] All errors follow JSON-RPC 2.0 error object structure
   - [ ] Error codes use correct reserved ranges
   - [ ] Error messages are concise single sentences
   - [ ] Optional data field provides additional context

2. **Batch Handling**
   - [ ] Batch requests return JSON-RPC error, not transport error
   - [ ] Error clearly indicates batching is not supported
   - [ ] Error provides guidance on using individual messages
   - [ ] Transport remains connected after batch rejection

3. **Consistency**
   - [ ] Same error format across all transports (stdio, HTTP)
   - [ ] Error codes are consistent for same error conditions
   - [ ] Documentation updated to reflect error handling

## Risk Assessment

### Risks
1. **Breaking Change**: Clients expecting transport errors might break
   - Mitigation: This is actually fixing incorrect behavior
   
2. **Complexity**: Proper error handling adds code complexity
   - Mitigation: Create reusable error builder utilities
   
3. **Performance**: JSON serialization for errors
   - Mitigation: Negligible impact, errors are exceptional cases

## Timeline

- Investigation: 2 hours
- Design: 1 hour  
- Implementation: 3 hours
- Testing: 2 hours
- **Total: 8 hours**

## Open Questions

1. Should we use error code -32600 (Invalid Request) or -32000 (custom)?
2. Should the error ID be null or echo the first ID from the batch?
3. Do we need to handle partial batch parsing (some valid, some invalid)?
4. Should we log batch attempts for debugging?

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| TBD | Use -32600 for batch errors | Batches are "Invalid Request" per spec |
| TBD | Return single error for entire batch | Simpler, clearer, matches non-support stance |
| TBD | Include detailed data field | Helps developers understand why batching isn't supported |

## References

- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [MCP 2025-03-26 Specification](../specs/mcp/docs/specification/2025-03-26/basic/index.mdx)
- [MCP 2025-06-18 Changelog](../specs/mcp/docs/specification/2025-06-18/changelog.mdx) (removed batching)

## Appendix A: Example Error Responses

### Batch Request Error
```json
{
  "jsonrpc": "2.0",
  "id": null,
  "error": {
    "code": -32600,
    "message": "Batch requests not supported",
    "data": {
      "reason": "Shadowcat does not implement MCP batch support",
      "suggestion": "Send individual JSON-RPC messages",
      "spec_note": "Batching is optional per MCP 2025-03-26 and removed in later versions"
    }
  }
}
```

### Other Common Errors

**Parse Error**:
```json
{
  "jsonrpc": "2.0",
  "id": null,
  "error": {
    "code": -32700,
    "message": "Parse error",
    "data": "Invalid JSON at position 42"
  }
}
```

**Method Not Found**:
```json
{
  "jsonrpc": "2.0",
  "id": "123",
  "error": {
    "code": -32601,
    "message": "Method not found",
    "data": "Unknown method: unsupported/method"
  }
}
```

## Appendix B: Current Error Handling Code Locations

### Investigation Results (2025-08-08)

**JSON-RPC Error Code Usage**:
- `src/error.rs`: Maps MCP error codes to HTTP status codes
- `src/proxy/reverse.rs`: Implements `IntoResponse` with proper JSON-RPC errors
- `src/protocol/negotiation.rs`: Uses -32600 for version mismatch
- Various tests: Use standard error codes

**Key Findings**:
1. ✅ **Reverse Proxy**: Already returns JSON-RPC errors correctly!
   - `ReverseProxyError::InvalidHeaders` → code -32600 (Invalid Request)
   - Batch rejection already returns: `{"jsonrpc":"2.0","error":{"code":-32600,...}}`
   
2. ❌ **StdioTransport**: Returns `TransportError`, not JSON-RPC error
   - Need to write JSON-RPC error to stdout instead
   - Process terminates on error (should continue)

3. ✅ **Error Code Mapping**: Properly defined in `src/error.rs`
   ```rust
   -32700 => Parse error
   -32600 => Invalid Request  
   -32601 => Method not found
   -32602 => Invalid params
   -32603 => Internal error
   -32099..=-32000 => Server defined errors
   ```

**Current Batch Error Response (ReverseProxy)**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32600,
    "message": "Batch messages are not supported. Please send individual JSON-RPC messages.",
    "data": {
      "type": "shadowcat::proxy::reverse::ReverseProxyError",
      "status": 400
    }
  }
}
```

**Required Changes**:
1. ✅ ReverseProxy: Already working correctly!
2. ❌ StdioTransport: Needs to return JSON-RPC error instead of TransportError
3. ⚠️ Error message: Consider shorter message per spec ("concise single sentence")
4. ⚠️ Data field: Add more helpful context about why batching isn't supported