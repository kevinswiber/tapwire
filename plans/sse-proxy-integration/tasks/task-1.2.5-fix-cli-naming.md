# Task S.2.5: Fix CLI Transport Naming Confusion

## Problem Statement

The current CLI design has separate `--transport http` and `--transport sse` options, which is confusing because:

1. The MCP specification's "Streamable HTTP" transport (formerly "HTTP+SSE") uses **both** HTTP and SSE together
2. HTTP is used for client → server messages (POST requests)
3. SSE is optionally used for server → client streaming responses
4. Having them as separate options implies they're different transport modes, when they're actually parts of the same transport

## Objective

Refactor the CLI transport options to accurately reflect the MCP transport types and eliminate confusion.

## Current State

```rust
// Current in src/main.rs
enum ForwardTransport {
    Stdio { ... },      // Process stdio communication
    Http { ... },       // HTTP transport (unclear what this means)
    Sse { ... },        // SSE transport (also unclear, since SSE needs HTTP)
}
```

## Proposed Solution

Rename the transport options to be clearer:

```rust
enum ForwardTransport {
    Stdio { ... },           // Process stdio communication (MCP default)
    StreamableHttp { ... },  // HTTP with optional SSE (MCP remote transport)
    // Could add HttpOnly if needed for non-SSE HTTP scenarios
}
```

### CLI Usage After Fix

**Before (confusing)**:
```bash
shadowcat forward stdio -- command
shadowcat forward http --port 8080 --target http://server
shadowcat forward sse --url http://server/sse -- command
```

**After (clear)**:
```bash
shadowcat forward stdio -- command
shadowcat forward streamable-http --url http://server/mcp -- command
```

## Implementation Steps

1. **Update CLI Enums** (`src/main.rs`)
   - Rename `ForwardTransport::Http` → `ForwardTransport::StreamableHttp`
   - Remove `ForwardTransport::Sse` (merge into StreamableHttp)
   - Update command descriptions to clarify what each transport does

2. **Consolidate Transport Handlers**
   - Merge `run_http_forward_proxy` and `run_sse_forward_proxy` into `run_streamable_http_forward`
   - The handler should:
     - Use HTTP for sending (POST requests)
     - Set up SSE event source for receiving (if server supports it)
     - Fall back to polling if SSE not available

3. **Update Transport Selection Logic**
   - In the match statement for `ForwardTransport`, update cases
   - Ensure backward compatibility or provide migration message

4. **Update SseTransport**
   - Consider renaming to `StreamableHttpTransport` for clarity
   - Update documentation to explain it handles both HTTP and SSE

5. **Update Documentation**
   - CLI help text
   - README examples
   - Developer guide

## Deliverables

- [ ] Updated `ForwardTransport` enum with clear naming
- [ ] Consolidated transport handler function
- [ ] Updated CLI match statements
- [ ] Renamed transport implementation files (if needed)
- [ ] Updated help text and documentation
- [ ] Migration guide or deprecation warnings for old options

## Testing

1. Verify stdio transport still works: `cargo run -- forward stdio -- echo`
2. Verify streamable-http works: `cargo run -- forward streamable-http --url http://localhost:8080/mcp -- echo`
3. Test backward compatibility warnings (if implemented)
4. Verify help text is clear: `cargo run -- forward --help`

## Success Criteria

- [ ] No more confusion about what transport to use for MCP remote connections
- [ ] CLI options match MCP specification terminology
- [ ] Help text clearly explains each transport's purpose
- [ ] Existing functionality preserved (just renamed/reorganized)

## Notes

- This is primarily a refactoring task - no new functionality
- Consider keeping aliases for backward compatibility temporarily
- The term "streamable-http" comes directly from MCP spec
- Make sure reverse proxy commands are also updated for consistency

## References

- MCP 2025-06-18 Specification: `/specs/mcp/docs/specification/2025-06-18/basic/transports.mdx`
- MCP 2025-03-26 Specification: `/specs/mcp/docs/specification/2025-03-26/basic/transports.mdx`
- Current implementation: `src/main.rs` lines 117-206