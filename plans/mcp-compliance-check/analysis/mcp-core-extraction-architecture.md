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

pub mod client;
pub mod server;
pub mod transports;
pub mod interceptor;

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

// crates/mcp/src/transports/mod.rs

pub mod stdio;
pub mod http;

/// Transport trait that both client and server use
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&mut self, data: Vec<u8>) -> Result<()>;
    async fn receive(&mut self) -> Result<Vec<u8>>;
    async fn close(&mut self) -> Result<()>;
}
```

// crates/mcp/src/client.rs

use crate::{JsonRpcRequest, JsonRpcResponse, ProtocolVersion};
use crate::transports::Transport;
use crate::interceptor::{InterceptorChain, Interceptor};

/// A generic MCP client that can connect to any MCP server
pub struct Client<T: Transport> {
    transport: T,
    interceptors: InterceptorChain,
    session_id: Option<String>,
    protocol_version: Option<ProtocolVersion>,
    server_capabilities: Option<ServerCapabilities>,
}

impl<T: transports::Transport> Client<T> {
    /// Create a new client with given transport
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            interceptors: InterceptorChain::new(),
            session_id: None,
            protocol_version: None,
            server_capabilities: None,
        }
    }
    
    /// Add an interceptor to the processing chain
    pub fn add_interceptor(&mut self, interceptor: Box<dyn Interceptor>) {
        self.interceptors.add(interceptor);
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
        // Process request through interceptors
        let request = self.interceptors.process_request(request).await?;
        
        // Serialize and send through transport
        let data = serde_json::to_vec(&request)?;
        self.transport.send(data).await?;
        
        // Receive and deserialize response
        let response_data = self.transport.receive().await?;
        let response: JsonRpcResponse = serde_json::from_slice(&response_data)?;
        
        // Process response through interceptors
        let response = self.interceptors.process_response(response).await?;
        
        Ok(response)
    }
}
```

// crates/mcp/src/server.rs

use crate::{JsonRpcRequest, JsonRpcResponse, ServerCapabilities};
use crate::transports::Transport;
use crate::interceptor::{InterceptorChain, Interceptor};

/// Trait that MCP server implementations must provide
#[async_trait]
pub trait McpHandler: Send + Sync {
    async fn handle_initialize(&self, params: Value) -> Result<InitializeResult>;
    async fn handle_tool_call(&self, name: &str, args: Value) -> Result<Value>;
    async fn handle_resource_get(&self, uri: &str) -> Result<Resource>;
    // ... other methods
}

/// A generic MCP server that can handle any MCP client
pub struct Server<H: McpHandler> {
    handler: H,
    interceptors: InterceptorChain,
    capabilities: ServerCapabilities,
    sessions: HashMap<String, SessionState>,
}

impl<H: McpHandler> Server<H> {
    pub fn new(handler: H, capabilities: ServerCapabilities) -> Self {
        Self {
            handler,
            interceptors: InterceptorChain::new(),
            capabilities,
            sessions: HashMap::new(),
        }
    }
    
    pub fn add_interceptor(&mut self, interceptor: Box<dyn Interceptor>) {
        self.interceptors.add(interceptor);
    }
    
    /// Handle incoming connection
    pub async fn handle_connection<T: Transport>(&mut self, mut transport: T) -> Result<()> {
        loop {
            // Receive request
            let data = transport.receive().await?;
            let request: JsonRpcRequest = serde_json::from_slice(&data)?;
            
            // Process through interceptors
            let request = self.interceptors.process_request(request).await?;
            
            // Route to appropriate handler
            let response = match request.method.as_str() {
                "initialize" => self.handle_initialize(request).await?,
                "tools/list" => self.handle_tools_list(request).await?,
                "tools/call" => self.handle_tools_call(request).await?,
                _ => self.handle_unknown(request).await?,
            };
            
            // Process response through interceptors
            let response = self.interceptors.process_response(response).await?;
            
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

### Per-Request Streaming Architecture

```rust
// crates/mcp/src/server.rs

/// Handler decides per-request whether to stream
#[async_trait]
pub trait McpHandler: Send + Sync {
    async fn handle_initialize(&self, params: Value) -> Result<InitializeResult>;
    async fn handle_tool_call(&self, name: &str, args: Value) -> Result<HandlerResult>;
    // ... other methods
}

/// Handler can return single response OR stream
pub enum HandlerResult {
    /// Single response - server returns application/json
    Single(Value),
    
    /// Stream of responses - server returns text/event-stream  
    Stream(Box<dyn Stream<Item = Value> + Send>),
}
```

### Transport-Specific Implementations

```rust
// crates/mcp/src/transports/stdio.rs

use crate::client::Client;

pub struct Transport { /* stdio-specific implementation */ }

impl Transport {
    pub fn new() -> Self { /* ... */ }
    pub fn spawn(cmd: &str, args: &[&str]) -> Result<Self> { /* ... */ }
}

/// Convenience function to create a client with stdio transport
pub fn connect(cmd: &str, args: &[&str]) -> Result<Client<Transport>> {
    let transport = Transport::spawn(cmd, args)?;
    Ok(Client::new(transport))
}

// crates/mcp/src/transports/http.rs

use hyper::{Client as HyperClient, Body, Request, Response};

pub struct Transport { 
    client: HyperClient<HttpsConnector<HttpConnector>>,
    url: String,
}

impl Transport {
    pub fn new(url: &str) -> Result<Self> { /* ... */ }
    
    /// Client automatically handles both JSON and SSE responses
    async fn send(&mut self, data: Vec<u8>) -> Result<()> {
        let request = Request::post(&self.url)
            .header("Accept", "application/json, text/event-stream")  // MUST support both
            .body(Body::from(data))?;
        
        let response = self.client.request(request).await?;
        
        // Server decides via Content-Type
        match response.headers().get("content-type") {
            Some(ct) if ct == "text/event-stream" => self.handle_sse_stream(response).await,
            Some(ct) if ct == "application/json" => self.handle_single_response(response).await,
            _ => Err(Error::InvalidContentType)
        }
    }
}

/// Convenience function for HTTP client
pub fn connect(url: &str) -> Result<Client<Transport>> {
    let transport = Transport::new(url)?;
    Ok(Client::new(transport))
}
```

## How Shadowcat Uses These

```rust
// shadowcat/src/proxy.rs

use mcp::{Client, Server, JsonRpcRequest, JsonRpcResponse};
use mcp::transports::{stdio, http};

pub struct ShadowcatProxy {
    // Shadowcat IS an MCP server (to downstream clients)
    server: Server<ProxyHandler>,
    
    // Shadowcat IS an MCP client (to upstream servers)  
    client_pool: Vec<Box<dyn Any>>,  // Type-erased clients
    
    // Proxy-specific state
    session_mappings: HashMap<String, String>,
}

impl ShadowcatProxy {
    pub async fn handle_client_connection<T: Transport>(&mut self, transport: T) {
        // Use MCP server to handle downstream
        self.server.handle_connection(transport).await
    }
    
    pub async fn connect_upstream_stdio(&mut self, cmd: &str) -> Result<()> {
        // Use stdio transport
        let mut client = stdio::connect(cmd, &[])?;
        client.add_interceptor(Box::new(LoggingInterceptor));
        client.initialize(self.client_info(), self.client_capabilities()).await?;
        self.client_pool.push(Box::new(client));
        Ok(())
    }
    
    pub async fn connect_upstream_http(&mut self, url: &str) -> Result<()> {
        // Use HTTP transport (handles SSE automatically)
        let mut client = http::connect(url)?;
        client.add_interceptor(Box::new(MetricsInterceptor));
        client.initialize(self.client_info(), self.client_capabilities()).await?;
        self.client_pool.push(Box::new(client));
        Ok(())
    }
}

// Shadowcat's handler implementation
struct ProxyHandler {
    // Proxy-specific state
}

impl McpHandler for ProxyHandler {
    async fn handle_tool_call(&self, name: &str, args: Value) -> Result<Value> {
        // Forward to upstream using appropriate client
        // Type erasure handled here
    }
}
```

## How Compliance Testing Uses These

```rust
// crates/compliance/src/lib.rs

use mcp::{Client, Server};
use mcp::transports::{stdio, http};

pub struct ComplianceChecker {
    // Can create clients with different transports for testing
    // Can create servers with test handlers
}

impl ComplianceChecker {
    /// Test our client against reference server
    pub async fn test_our_client_vs_reference(&mut self) -> ComplianceMatrix {
        let mut matrix = ComplianceMatrix::new();
        
        // Test our Client with stdio transport against official server
        let mut our_client = stdio::connect("node", &["official-server.js"])?;
        matrix.our_client_vs_reference = self.run_client_tests(&mut our_client).await;
        
        // Also test with HTTP transport (handles SSE when server chooses)
        let mut our_http_client = http::connect("http://official-server:3000")?;
        matrix.our_http_client_vs_reference = self.run_client_tests(&mut our_http_client).await;
        
        matrix
    }
    
    /// Test reference client against our server
    pub async fn test_reference_vs_our_server(&mut self) -> ComplianceMatrix {
        let mut matrix = ComplianceMatrix::new();
        
        // Start our Server with test handler
        let our_server = Server::new(TestHandler::new(), test_capabilities());
        
        // Server can handle any transport
        // Test with stdio, HTTP/SSE, etc.
        
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
Transport implementations with streaming modules:
```rust
crates/mcp/src/
├── lib.rs
├── client.rs
├── server.rs
├── interceptor.rs
└── transports/
    ├── mod.rs         # Transport trait
    ├── stdio.rs       # stdio::Transport
    └── http/
        ├── mod.rs     # http::Transport
        └── streaming/
            ├── mod.rs
            ├── sse.rs       # SSE streaming logic
            └── websockets.rs # Future WebSocket logic
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

### 3. Interceptor Design

```rust
// crates/mcp/src/interceptor.rs

#[async_trait]
pub trait Interceptor: Send + Sync {
    /// Process outgoing request
    async fn process_request(&self, request: JsonRpcRequest) -> Result<JsonRpcRequest> {
        Ok(request)  // Default: pass through
    }
    
    /// Process incoming response
    async fn process_response(&self, response: JsonRpcResponse) -> Result<JsonRpcResponse> {
        Ok(response)  // Default: pass through
    }
}

pub struct InterceptorChain {
    interceptors: Vec<Box<dyn Interceptor>>,
}

impl InterceptorChain {
    pub fn new() -> Self {
        Self { interceptors: Vec::new() }
    }
    
    pub fn add(&mut self, interceptor: Box<dyn Interceptor>) {
        self.interceptors.push(interceptor);
    }
    
    pub async fn process_request(&self, mut request: JsonRpcRequest) -> Result<JsonRpcRequest> {
        for interceptor in &self.interceptors {
            request = interceptor.process_request(request).await?;
        }
        Ok(request)
    }
    
    pub async fn process_response(&self, mut response: JsonRpcResponse) -> Result<JsonRpcResponse> {
        // Process in reverse order for responses
        for interceptor in self.interceptors.iter().rev() {
            response = interceptor.process_response(response).await?;
        }
        Ok(response)
    }
}
```

### 4. Version Management
Runtime version selection with protocol negotiation:
```rust
impl<T: transports::Transport> Client<T> {
    pub fn with_version(mut self, version: ProtocolVersion) -> Self {
        self.preferred_version = Some(version);
        self
    }
}
```

### 5. Why Not Reqwest?

We're using **hyper** directly instead of reqwest because:
- **SSE Issues**: Reqwest doesn't handle Server-Sent Events well
- **More Control**: Direct access to HTTP/2 features
- **Less Overhead**: No unnecessary client features
- **Shadowcat Proven**: Already working well in shadowcat

```rust
// Example of http::Transport internally handling SSE

impl http::Transport {
    async fn handle_sse_stream(&mut self, response: Response<Body>) -> Result<()> {
        // Parse SSE events from the response body
        let stream = response.into_body();
        // Process events...
    }
    
    async fn handle_single_response(&mut self, response: Response<Body>) -> Result<()> {
        // Parse single JSON response
        let bytes = hyper::body::to_bytes(response.into_body()).await?;
        let json: Value = serde_json::from_slice(&bytes)?;
        // Process response...
    }
}
```

## Conclusion

This **hybrid architecture** provides the best of both worlds:

### Architecture Summary
1. **Core Protocol Layer**: `mcp::{Client, Server}` handle MCP logic
2. **Transport Abstraction**: Clean modules `mcp::transports::{stdio, http}`
3. **Interceptor Chain**: Protocol-level processing with `interceptor::Chain`
4. **Per-Request Streaming**: Handlers decide streaming per request via `HandlerResult`
5. **Type-Conscious Naming**: `stdio::Transport` not `StdioTransport`
6. **Hyper over Reqwest**: Better SSE support, proven in shadowcat

### Key Benefits
- **DRY Principle**: Protocol logic in one place
- **Transport Flexibility**: Easy to add new transports
- **Clean Separation**: Protocol vs transport concerns
- **Interceptor Integration**: Uniform message processing
- **Type Safety**: Transport-specific features when needed
- **Proven Foundation**: Leverages shadowcat's working patterns

### Design Decisions
- **Hybrid over Pure**: Balance between central logic and transport flexibility
- **Interceptors at Protocol Layer**: Not in transports, for consistency
- **Hyper for HTTP/SSE**: Proven in shadowcat, better than reqwest
- **Convenience Builders**: Each transport provides easy constructors

The architecture enables both simple usage (`stdio::connect()`) and advanced control (custom transports, interceptors, transport-specific features).

---

*Created: 2025-08-24*
*Updated: 2025-08-24*
*Key Decisions:*
- *Hybrid architecture with protocol/transport separation*
- *Simplified transport: `mcp::transports::{stdio, http}`*
- *Per-request streaming via `HandlerResult` enum*
- *Type-conscious naming: `module::Type` not `ModuleType`*
- *HTTP Library: Hyper (not reqwest) for SSE support*