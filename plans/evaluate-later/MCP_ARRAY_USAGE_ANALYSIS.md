# MCP Array Usage Analysis

## Executive Summary

**JSON arrays at the top-level (message batching) are ONLY used in MCP version 2025-03-26.**

After reviewing all MCP specification versions:
- **2024-11-05**: No batching support
- **2025-03-26**: SUPPORTS batching (JSON arrays at top-level)
- **2025-06-18**: Explicitly REMOVED batching support
- **draft** (unreleased): No batching support

## Key Findings

### 1. Top-Level Arrays (Batching)

**Only in 2025-03-26:**
- Messages can be sent as JSON arrays for batching
- Both requests/notifications AND responses can be batched
- Per specification: "MCP implementations **MAY** support sending JSON-RPC batches, but **MUST** support receiving JSON-RPC batches"

**Explicitly removed in 2025-06-18:**
From the changelog: "Remove support for JSON-RPC [batching](https://www.jsonrpc.org/specification#batch)"

### 2. Arrays Within Messages

Arrays ARE used within message parameters/results across ALL versions for:
- `tools`: Array of tool definitions
- `resources`: Array of resource definitions  
- `prompts`: Array of prompt templates
- `roots`: Array of root directories
- `values`: Array in completion results (max 100 items)
- `audience`: Array of strings in resource definitions

These are NOT top-level arrays but arrays within the JSON-RPC message structure.

## Implementation Recommendation

Given that:
1. **Only MCP 2025-03-26 uses top-level array batching**
2. **Later versions explicitly removed this feature**
3. **The BatchHandler correctly implements version-aware batching**

### Recommended Approach: Minimal Impact Integration

**Option 1: Version-Gated Transport Enhancement** (Recommended)
```rust
// In StdioTransport and other transports
impl StdioTransport {
    fn parse_message(&self, line: &str, direction: MessageDirection) 
        -> TransportResult<MessageEnvelope> {
        
        let json_value: Value = serde_json::from_str(line)?;
        
        // Only check for batches if using 2025-03-26
        if self.protocol_version == ProtocolVersion::V2025_03_26 
           && json_value.is_array() {
            // Handle batch - return error for now with clear message
            return Err(TransportError::ProtocolError(
                "Batch messages not yet supported. Use individual messages.".to_string()
            ));
        }
        
        // Continue with existing single message parsing...
    }
}
```

**Option 2: Defer Until Needed**
Since:
- Most MCP implementations don't use batching (it's optional: "MAY support")
- Newer protocol versions removed batching entirely
- We can handle single messages fine

We could:
1. Document that batching is not yet supported for 2025-03-26
2. Return clear error messages if batch messages are received
3. Implement full batch support only if/when needed

### Why This Is Safe

1. **Batching is optional**: The spec says implementations "MAY" support sending batches
2. **Single messages work**: All MCP operations can be done without batching
3. **Future versions don't batch**: 2025-06-18 and draft removed batching
4. **Clear failure mode**: Batch messages would fail immediately with clear errors

## Technical Debt Documentation

### Current State
- ✅ BatchHandler fully implemented and tested
- ✅ Correctly handles version-aware batching logic
- ❌ Not integrated into transport layer
- ❌ Transport cannot parse batch messages (arrays)

### Impact
- **Low**: Batching is optional and rarely used
- **Predictable**: Batch messages fail with clear errors
- **Version-specific**: Only affects 2025-03-26 protocol

### Future Integration Path
When/if batch support is needed:
1. Update `parse_message()` to return `Vec<MessageEnvelope>` 
2. Use BatchHandler to split incoming batches
3. Update `send()` to support batching multiple messages
4. Add batch-aware session management

## Conclusion

Since batch support is:
- Optional in the spec
- Only for one older protocol version (2025-03-26)
- Removed in newer versions (2025-06-18+)
- Not blocking any current functionality

**Recommendation**: Take our time with integration. Add version-gated error handling now, implement full batch support only if/when needed by actual MCP servers/clients.

The BatchHandler is ready and well-designed for when we need it, but there's no urgency to integrate it given the protocol evolution away from batching.