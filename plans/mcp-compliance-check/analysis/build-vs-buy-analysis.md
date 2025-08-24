# MCP Implementation Strategy for Shadowcat

## Core Philosophy

**Build a fast, compliant, low-level MCP implementation** focused on:
1. **Performance** - Minimal overhead, zero-copy where possible
2. **Compliance** - 100% spec adherence, including draft versions
3. **Simplicity** - Direct, no-magic implementation
4. **Independence** - No dependency on external MCP libraries

## Why Independent Implementation

### Issues with Official Rust SDK (rmcp)
- Heavy use of macros (`#[tool_router]`, `#[tool]`) - adds complexity
- Many dependencies (oauth2, schemars, reqwest, etc.)
- Focused on developer ergonomics over performance
- May not support all our proxy-specific needs
- External dependency for core functionality

### Our Approach
```rust
// NO macro magic
// NO: #[tool_router]
// NO: #[tool(description = "...")]

// YES: Direct, explicit implementation
impl McpHandler for Counter {
    async fn handle_tool_call(&self, name: &str, args: Value) -> Result<Value> {
        match name {
            "increment" => {
                let mut counter = self.counter.lock().await;
                *counter += 1;
                Ok(json!({ "count": *counter }))
            }
            _ => Err(Error::MethodNotFound)
        }
    }
}
```

## Version Support Strategy

### Supported Versions
1. **2025-03-26** - Current stable
2. **2025-06-18** - Latest release  
3. **draft** - Living spec for early testing

### Draft Version Handling
```rust
pub enum ProtocolVersion {
    V2025_03_26,
    V2025_06_18,
    Draft { snapshot_date: String }, // Track which draft snapshot
}

impl ProtocolVersion {
    pub fn capabilities(&self) -> VersionCapabilities {
        match self {
            Self::V2025_03_26 => /* ... */,
            Self::V2025_06_18 => /* ... */,
            Self::Draft { .. } => {
                // May have experimental features
                // May have breaking changes
                // Use with caution
            }
        }
    }
}
```

### Draft Testing Benefits
- **Early feedback** on spec changes
- **Proactive adaptation** to breaking changes
- **Influence spec development** by finding issues early
- **Stay ahead** of other implementations

## Compliance Matrix with rmcp

Even though we won't depend on rmcp, we should test against it:

```
                    | Our Server | rmcp Server | Reference JS Server
--------------------|------------|-------------|--------------------
Our Client          |    ✅      |     ✅      |        ✅
rmcp Client         |    ✅      |     ✅      |        ✅
Reference JS Client |    ✅      |     ✅      |        ✅
```

This ensures our implementation is compatible with the ecosystem.

## Implementation Priorities

### Phase 1: Core Protocol (Fast & Compliant)
```rust
// Simple, direct types
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<JsonRpcId>,
    pub method: String,
    pub params: Option<Value>,
}

// No fancy derive macros, just what we need
impl JsonRpcRequest {
    pub fn new(method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(JsonRpcId::new()),
            method: method.into(),
            params: Some(params),
        }
    }
}
```

### Phase 2: Performance Optimizations
```rust
// Zero-copy where possible
pub struct McpMessage<'a> {
    raw: &'a [u8],
    parsed: OnceCell<JsonRpcRequest>,
}

// Buffer pooling
pub struct MessagePool {
    buffers: Vec<BytesMut>,
}

// Efficient serialization
impl JsonRpcRequest {
    pub fn write_to(&self, buf: &mut BytesMut) -> Result<()> {
        // Direct serialization to reusable buffer
    }
}
```

### Phase 3: Proxy-Specific Features
```rust
// Session management optimized for proxy use
pub struct ProxySession {
    client_id: Uuid,
    upstream_id: Uuid,
    // Minimal state, fast lookups
}

// Connection pooling
pub struct UpstreamPool {
    connections: Vec<McpClient>,
    strategy: LoadBalancing,
}
```

## What We DON'T Need (Initially)

1. **Macro magic** - No procedural macros for tools
2. **Auto-generated schemas** - Manual schemas are fine
3. **Fancy type wrappers** - Direct Value manipulation
4. **Heavy abstractions** - Keep it simple
5. **All transport types** - Focus on HTTP/SSE first

## What We DO Need

1. **Strict compliance** - Every MUST in the spec
2. **Fast message processing** - Minimal overhead
3. **Efficient session management** - Optimized for proxy
4. **Clean error handling** - Proper error codes
5. **Version flexibility** - Support stable + draft

## Comparison with rmcp

| Aspect | rmcp | Our Implementation |
|--------|------|-------------------|
| **Focus** | Developer ergonomics | Performance & compliance |
| **Dependencies** | Many (oauth2, schemars, etc.) | Minimal |
| **Macros** | Heavy use (#[tool], etc.) | None initially |
| **Flexibility** | Opinionated structure | Direct control |
| **Proxy support** | Not designed for | Built-in consideration |
| **Draft support** | Unknown | First-class support |

## Testing Strategy

### Against Our Implementation
```rust
#[test]
async fn test_our_client_our_server() {
    let server = our::McpServer::new();
    let client = our::McpClient::new();
    // Should work perfectly
}
```

### Against rmcp
```rust
#[test]
async fn test_our_client_rmcp_server() {
    let server = start_rmcp_server();
    let client = our::McpClient::new();
    // Should be compatible
}

#[test]
async fn test_rmcp_client_our_server() {
    let server = our::McpServer::new();
    let client = rmcp::Client::new();
    // Should be compatible
}
```

### Against Reference Implementation
```rust
#[test]
async fn test_our_client_reference_server() {
    let server = start_js_reference_server();
    let client = our::McpClient::new();
    // Ultimate compatibility test
}
```

## Benefits of This Approach

1. **Full control** - No surprises from external deps
2. **Optimized for proxy** - Built with Shadowcat needs in mind
3. **Performance focused** - No overhead from ergonomic features
4. **Spec compliant** - Direct implementation of spec
5. **Future proof** - Easy to adapt to spec changes

## Later Enhancements (If Needed)

Once core is solid, we could add:
- Derive macros for tools (like rmcp)
- Schema generation
- Higher-level abstractions
- Additional transports

But these are **optional** - core functionality comes first.

## Success Criteria

Our MCP implementation should be:
1. **Faster** than rmcp for proxy use cases
2. **100% compliant** with all spec versions including draft
3. **Compatible** with rmcp and reference implementations
4. **Maintainable** without external dependencies
5. **Flexible** enough for all proxy scenarios

## Conclusion

By building our own low-level, performance-focused MCP implementation, we:
- Maintain complete control
- Optimize for our specific needs
- Ensure maximum compliance
- Support draft versions early
- Create the foundation for the best MCP proxy

The official SDK is great for building MCP tools quickly, but for Shadowcat's core infrastructure, we need something more fundamental and under our control.

---

*Created: 2025-08-24*
*Strategy: Fast, compliant, independent MCP implementation*
*Priority: Performance and spec compliance over developer ergonomics*