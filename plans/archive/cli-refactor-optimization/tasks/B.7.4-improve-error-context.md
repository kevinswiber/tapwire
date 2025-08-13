# Task B.7.4: Improve Transport Factory Error Context

**Status**: ⬜ Not Started  
**Estimated Duration**: 1 hour  
**Dependencies**: B.4 (Transport Factory), B.5 (Error Handling)  
**Priority**: MEDIUM  

## Context

From the [Comprehensive Review](../../../reviews/cli-refactor-optimization/comprehensive-review.md#92-medium-priority), transport factory error messages need more context about which transport type failed and why.

## Problem

Current error messages in `src/transport/factory.rs` are generic and don't provide enough context for debugging:
- Which transport type was being created
- What specific configuration was invalid
- Why the creation failed

## Solution

Enhance error messages with specific context:

```rust
// Before:
Err(e) => Err(ShadowcatError::Config(
    ConfigError::Invalid(format!("Failed to create transport: {e}"))
))

// After:
Err(e) => Err(ShadowcatError::Config(
    ConfigError::Invalid(format!(
        "Failed to create {} transport from '{}': {e}",
        transport_type, original_spec
    ))
))
```

## Implementation Steps

1. [ ] Review all error paths in `src/transport/factory.rs`
2. [ ] Identify error messages that lack context
3. [ ] Add context for each error type:
   - [ ] Transport type being created (stdio, http, sse)
   - [ ] Original specification or URL
   - [ ] Specific validation that failed
4. [ ] Update error messages in:
   - [ ] `from_spec()` method
   - [ ] `from_url()` method
   - [ ] `parse_command()` method
   - [ ] Builder validation methods
5. [ ] Consider adding error variants for specific transport failures

## Examples of Improved Messages

```rust
// Before:
"Invalid URL"

// After:
"Invalid HTTP transport URL 'htp://example.com': scheme must be 'http' or 'https'"

// Before:
"Failed to create transport"

// After:
"Failed to create stdio transport for command 'echo test': command not found"

// Before:
"Transport creation failed"

// After:
"Failed to create SSE transport: URL 'http://example.com/sse' returned 404"
```

## Testing

- [ ] Test with invalid URLs for each transport type
- [ ] Test with missing commands for stdio
- [ ] Test with unreachable servers for HTTP/SSE
- [ ] Verify error messages are actionable
- [ ] Ensure no sensitive information in errors

## Success Criteria

- [ ] All error messages include transport type
- [ ] Error messages include relevant configuration details
- [ ] Users can understand what went wrong from the error
- [ ] No generic "failed to create" messages remain
- [ ] Tests verify error message quality