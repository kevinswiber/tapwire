# MCP Core Library Extraction Architecture

## Brilliant Insight

You're absolutely right! We need Rust MCP client/server implementations for testing anyway. Instead of duplicating this code or depending on Shadowcat internals, we should **extract the MCP protocol implementation into shared crates**.

## The New Architecture

```
shadowcat/                 # Workspace root
├── src/                  # Shadowcat lib/CLI
├── Cargo.toml           # Workspace + shadowcat package
├── crates/
│   ├── mcp/            # Shared MCP implementation (core + client + server)
│   └── compliance/     # Tests using mcp crate
└── xtask/              # Build automation
```

## What Gets Extracted

### Single MCP Crate (Protocol, Client, and Server)

```rust
// crates/mcp/src/lib.rs

/// Core protocol types used by both client and server
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<JsonRpcId>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<JsonRpcId>,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

/// Protocol version support
#[derive(Clone, Copy, PartialEq)]
pub enum ProtocolVersion {
    V2025_03_26,
    V2025_06_18,
}

/// Capabilities structure
#[derive(Serialize, Deserialize, Default)]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub prompts: Option<PromptsCapability>,
    pub logging: Option<LoggingCapability>,
}

/// Transport trait that both client and server use
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&mut self, data: Vec<u8>) -> Result<()>;
    async fn receive(&mut self) -> Result<Vec<u8>>;
    async fn close(&mut self) -> Result<()>;
}
```

// crates/mcp/src/client.rs

use crate::{JsonRpcRequest, JsonRpcResponse, Transport, ProtocolVersion};

/// A generic MCP client that can connect to any MCP server
pub struct McpClient {
    transport: Box<dyn Transport>,
    session_id: Option<String>,
    protocol_version: Option<ProtocolVersion>,
    server_capabilities: Option<ServerCapabilities>,
}

impl McpClient {
    /// Create a new client with given transport
    pub fn new(transport: Box<dyn Transport>) -> Self {
        Self {
            transport,
            session_id: None,
            protocol_version: None,
            server_capabilities: None,
        }
    }
    
    /// Initialize connection with server
    pub async fn initialize(
        &mut self,
        client_info: ClientInfo,
        capabilities: ClientCapabilities,
    ) -> Result<InitializeResult> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(JsonRpcId::String("init".to_string())),
            method: "initialize".to_string(),
            params: Some(json!({
                "clientInfo": client_info,
                "capabilities": capabilities,
                "protocolVersion": "2025-06-18",
            })),
        };
        
        let response = self.send_request(request).await?;
        
        // Store capabilities and version
        if let Some(result) = response.result {
            self.protocol_version = Some(result["protocolVersion"].parse()?);
            self.server_capabilities = Some(serde_json::from_value(result["capabilities"])?);
        }
        
        Ok(response.into())
    }
    
    /// Call a tool on the server
    pub async fn call_tool(&mut self, name: &str, args: Value) -> Result<Value> {
        self.send_request(JsonRpcRequest {
            method: "tools/call".to_string(),
            params: Some(json!({ "name": name, "arguments": args })),
            ..Default::default()
        }).await
    }
    
    /// Send any request
    pub async fn send_request(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        // Serialize and send through transport
        let data = serde_json::to_vec(&request)?;
        self.transport.send(data).await?;
        
        // Receive and deserialize response
        let response_data = self.transport.receive().await?;
        let response: JsonRpcResponse = serde_json::from_slice(&response_data)?;
        
        Ok(response)
    }
}
```

// crates/mcp/src/server.rs

use crate::{JsonRpcRequest, JsonRpcResponse, Transport, ServerCapabilities};

/// Trait that MCP server implementations must provide
#[async_trait]
pub trait McpHandler: Send + Sync {
    async fn handle_initialize(&self, params: Value) -> Result<InitializeResult>;
    async fn handle_tool_call(&self, name: &str, args: Value) -> Result<Value>;
    async fn handle_resource_get(&self, uri: &str) -> Result<Resource>;
    // ... other methods
}

/// A generic MCP server that can handle any MCP client
pub struct McpServer {
    handler: Box<dyn McpHandler>,
    capabilities: ServerCapabilities,
    sessions: HashMap<String, SessionState>,
}

impl McpServer {
    pub fn new(handler: Box<dyn McpHandler>, capabilities: ServerCapabilities) -> Self {
        Self {
            handler,
            capabilities,
            sessions: HashMap::new(),
        }
    }
    
    /// Handle incoming connection
    pub async fn handle_connection(&mut self, transport: Box<dyn Transport>) -> Result<()> {
        loop {
            // Receive request
            let data = transport.receive().await?;
            let request: JsonRpcRequest = serde_json::from_slice(&data)?;
            
            // Route to appropriate handler
            let response = match request.method.as_str() {
                "initialize" => self.handle_initialize(request).await?,
                "tools/list" => self.handle_tools_list(request).await?,
                "tools/call" => self.handle_tools_call(request).await?,
                _ => self.handle_unknown(request).await?,
            };
            
            // Send response
            let response_data = serde_json::to_vec(&response)?;
            transport.send(response_data).await?;
        }
    }
    
    async fn handle_initialize(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let result = self.handler.handle_initialize(request.params.unwrap_or_default()).await?;
        
        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::to_value(result)?),
            error: None,
        })
    }
}
```

## How Shadowcat Uses These

```rust
// shadowcat/src/proxy.rs

use mcp::{McpClient, McpServer, Transport, JsonRpcRequest, JsonRpcResponse};

pub struct ShadowcatProxy {
    // Shadowcat IS an MCP server (to downstream clients)
    server: McpServer,
    
    // Shadowcat IS an MCP client (to upstream servers)
    client_pool: Vec<McpClient>,
    
    // Proxy-specific state
    session_mappings: HashMap<String, String>,
}

impl ShadowcatProxy {
    pub async fn handle_client_connection(&mut self, transport: Box<dyn Transport>) {
        // Use MCP server to handle downstream
        self.server.handle_connection(transport).await
    }
    
    pub async fn connect_upstream(&mut self, url: &str) -> Result<()> {
        // Use MCP client to connect upstream
        let transport = HttpTransport::new(url);
        let mut client = McpClient::new(Box::new(transport));
        client.initialize(self.client_info(), self.client_capabilities()).await?;
        self.client_pool.push(client);
        Ok(())
    }
}

// Shadowcat implements the handler for its server side
impl McpHandler for ShadowcatProxy {
    async fn handle_tool_call(&self, name: &str, args: Value) -> Result<Value> {
        // Forward to upstream using client
        let client = self.select_upstream_client();
        client.call_tool(name, args).await
    }
}
```

## How Compliance Testing Uses These

```rust
// crates/compliance/src/lib.rs

use mcp::{McpClient, McpServer, Transport};

pub struct ComplianceChecker {
    // We can test any MCP server (including Shadowcat)
    test_client: McpClient,
    
    // We can test any MCP client (including Shadowcat)
    test_server: McpServer,
}

impl ComplianceChecker {
    /// Test our client against reference server
    pub async fn test_our_client_vs_reference(&mut self) -> ComplianceMatrix {
        let mut matrix = ComplianceMatrix::new();
        
        // Test our McpClient against official server
        let transport = connect_to_official_server();
        let mut our_client = McpClient::new(transport);
        matrix.our_client_vs_reference = self.run_client_tests(&mut our_client).await;
        
        matrix
    }
    
    /// Test reference client against our server
    pub async fn test_reference_vs_our_server(&mut self) -> ComplianceMatrix {
        let mut matrix = ComplianceMatrix::new();
        
        // Start our McpServer
        let our_server = McpServer::new(TestHandler::new(), test_capabilities());
        
        // Connect official client to it
        let official_client = start_official_client(&our_server.url());
        matrix.reference_vs_our_server = self.run_server_tests(official_client).await;
        
        matrix
    }
    
    /// Test Shadowcat as both client and server
    pub async fn test_shadowcat(&mut self) -> ComplianceMatrix {
        // Shadowcat uses the same mcp-client and mcp-server
        // So we're testing the same implementation
        // But in proxy configuration
    }
}
```

## The Compliance Matrix

This architecture enables a comprehensive compliance matrix:

```
                    | Our Server | Reference Server | Shadowcat Server
--------------------|------------|------------------|------------------
Our Client          |    ✅      |       ✅         |       ✅
Reference Client    |    ✅      |       ✅         |       ✅  
Shadowcat Client    |    ✅      |       ✅         |       ✅

Each ✅ represents a full test suite run
```

## Benefits of This Approach

### 1. Code Reuse
- No duplication of MCP protocol implementation
- Single source of truth for protocol logic
- Shared between proxy and compliance checker

### 2. Still Independent
- Not depending on Shadowcat internals
- Depending on shared MCP protocol libraries
- Clean separation of concerns

### 3. Enables More Testing
- Test our client vs reference server
- Test reference client vs our server
- Build compliance matrix
- Identify implementation differences

### 4. Better Architecture
- MCP protocol separated from proxy logic
- Reusable MCP client/server for other projects
- Cleaner, more modular codebase

### 5. Easier Maintenance
- Protocol updates in one place
- Shared bug fixes
- Consistent behavior

## Implementation Strategy

### Phase 1: Create MCP Crate
1. Create `crates/mcp/` with unified structure
2. Move protocol types from shadowcat to `mcp/src/lib.rs`
3. Extract client logic to `mcp/src/client.rs`
4. Extract server logic to `mcp/src/server.rs`
5. Define Transport trait and McpHandler trait

### Phase 2: Refactor Shadowcat
1. Update to use single `mcp` crate
2. Remove duplicate protocol code
3. Focus on proxy-specific logic

### Phase 3: Build Compliance Framework
1. Create `crates/compliance/` 
2. Use `mcp` crate for testing
3. Build compliance matrix tests
4. Test all combinations

## What You Might Be Missing

Actually, you've identified the key insight! But here are some additional considerations:

### 1. Transport Implementations
Transport implementations can be modules within the mcp crate:
```rust
crates/mcp/src/
├── lib.rs
├── client.rs
├── server.rs
└── transports/
    ├── mod.rs
    ├── stdio.rs
    ├── http.rs
    └── sse.rs
```

### 2. Test Utilities
Test utilities can be part of the mcp crate for reuse:
```rust
crates/mcp/src/
└── test_utils/
    ├── mod.rs
    ├── mock_server.rs
    ├── mock_client.rs
    └── test_transport.rs
```

### 3. Version Management
How do we handle protocol versions in shared crates?
```rust
// Feature flags?
[features]
v2025-03-26 = []
v2025-06-18 = []

// Or runtime selection?
impl McpClient {
    pub fn with_version(version: ProtocolVersion) -> Self { ... }
}
```

### 4. Async Trait Complexity
The async-trait crate adds some overhead. Consider:
```rust
// Option 1: async-trait (easier but slight overhead)
#[async_trait]
pub trait Transport { ... }

// Option 2: manual futures (more complex but zero overhead)
pub trait Transport {
    fn send(&mut self, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<()>>>>;
}
```

## Conclusion

This is a **major architectural improvement** that:
1. Enables code reuse without compromising independence
2. Allows comprehensive compliance matrix testing
3. Creates a reusable MCP implementation in a single crate
4. Maintains clean separation of concerns
5. Makes the codebase more maintainable with less overhead

The single `mcp` crate approach keeps things simple while providing all the benefits of extraction. We avoid the overhead of managing multiple interdependent crates while still achieving modularity through well-organized modules within the crate.

The key insight is that "independence" doesn't mean "no shared code" - it means "no dependency on implementation details". A shared MCP protocol library is perfectly fine and actually better than duplicating code.

---

*Created: 2025-08-24*
*Updated: 2025-08-24*
*Key Insight: Extract MCP protocol into single shared crate*
*Result: Simpler architecture, easier maintenance, full testing capabilities*