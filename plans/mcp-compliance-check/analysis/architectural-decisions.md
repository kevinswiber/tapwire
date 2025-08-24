# MCP Library Architectural Decisions

## Executive Summary

After analyzing shadowcat's current implementation and evaluating trade-offs, we've made the following key architectural decisions for the MCP library extraction:

1. **Hybrid Client/Server Architecture** - Protocol logic centralized, transport flexibility maintained
2. **Transport Organization** - `mcp::transports::http::streaming::{sse, websockets}`
3. **Type-Conscious Naming** - `stdio::Transport` not `StdioTransport`
4. **Per-Request Streaming** - Handlers decide streaming per request via `HandlerResult`
5. **Interceptor Integration** - At protocol layer with `interceptor::Chain`
6. **HTTP Library** - Hyper (not reqwest) for SSE support
7. **Single MCP Crate** - One crate with well-organized modules

## 1. Hybrid Client/Server Architecture

### Decision: Protocol Core + Transport Abstraction

```rust
// Core protocol handlers (centralized MCP logic)
mcp::{Client<T>, Server<H>}

// Transport implementations (handle communication)
mcp::transports::{stdio::Transport, http::Transport}
```

### Rationale

**Why not central Client/Server with dynamic dispatch?**
- Would lose type safety for transport-specific features
- Runtime overhead from trait objects
- Can't leverage transport-specific optimizations

**Why not transport-specific clients/servers?**
- Would duplicate protocol logic across transports
- Harder to maintain consistency
- More code to test and debug

**Why hybrid works:**
- Protocol logic stays DRY in `Client<T>` and `Server<H>`
- Transport flexibility through generics
- Type safety when needed (e.g., `Client<SseTransport>` can have SSE-specific methods)
- Clean separation of concerns

### Usage Examples

```rust
// Simple usage with convenience builders
let client = stdio::connect("mcp-server", &["--arg"])?;

// Advanced usage with direct construction
let transport = http::Transport::new("http://server:3000")?;
let mut client = Client::new(transport);
client.add_interceptor(Box::new(LoggingInterceptor));

// Users can alias for clarity if desired
use mcp::transports::stdio::Transport as StdioTransport;
use mcp::transports::http::Transport as HttpTransport;
```

## 2. Transport Organization

### Decision: Hierarchical Module Structure

```
mcp::transports::{
    stdio,                    // stdio::Transport
    http::{                   // http::Transport
        streaming::{
            sse,              // SSE-specific streaming logic
            websockets,       // WebSocket streaming (future)
        }
    }
}
```

### Rationale

- **Separation of concerns** - SSE and WebSocket have distinct implementations
- **Code reuse** - Shared streaming logic can be factored out
- **MCP-aligned** - "Streamable HTTP" uses different streaming strategies
- **Future-proof** - Easy to add WebSocket support alongside SSE
- **Clean organization** - Complex streaming logic gets its own modules

**How it works:**
- `http::Transport` is the main transport implementation
- It internally uses `streaming::sse::Handler` when server returns SSE
- Future: Could use `streaming::websockets::Handler` based on negotiation
- The streaming modules are implementation details, not separate transports

## 3. Interceptor Design

### Decision: Protocol-Level Interceptors

```rust
pub struct Client<T: Transport> {
    transport: T,
    interceptors: interceptor::Chain,  // At protocol level
    // ...
}

#[async_trait]
pub trait Interceptor: Send + Sync {
    async fn process_request(&self, request: JsonRpcRequest) -> Result<JsonRpcRequest>;
    async fn process_response(&self, response: JsonRpcResponse) -> Result<JsonRpcResponse>;
}
```

### Rationale

**Why at protocol layer, not transport?**
- Interceptors operate on MCP messages, not raw bytes
- Consistent processing regardless of transport
- Can modify protocol-level concerns (methods, params, results)
- Simpler mental model

**Benefits:**
- Uniform message processing
- Easy to add logging, metrics, rate limiting
- Can modify/block messages based on content
- Works the same for all transports

## 4. Per-Request Streaming Decision

### Decision: Handler-Controlled Streaming

```rust
pub enum HandlerResult {
    /// Single response - server returns application/json
    Single(Value),
    
    /// Stream of responses - server returns text/event-stream
    Stream(Box<dyn Stream<Item = Value> + Send>),
}

#[async_trait]
pub trait McpHandler: Send + Sync {
    async fn handle_tool_call(&self, name: &str, args: Value) -> Result<HandlerResult>;
}
```

### Rationale

**Why per-request, not per-server?**
- Different operations have different needs
- `tool/list` → Single response makes sense
- `long_running_analysis` → Streaming progress is valuable
- Matches MCP spec intent (server chooses Content-Type per request)

**Client perspective:**
- Clients MUST send `Accept: application/json, text/event-stream`
- Server decides response format via Content-Type header
- Client handles either transparently

**Benefits:**
- Maximum flexibility for handlers
- No unnecessary overhead for simple operations
- Natural API for handler implementers
- Spec compliant

## 5. Type-Conscious Naming

### Decision: Module Provides Context

```rust
// Not this:
use mcp::transports::stdio::StdioTransport;
use mcp::interceptor::InterceptorChain;

// But this:
use mcp::transports::stdio::Transport;
use mcp::interceptor::Chain;

// Users can alias if they want:
use mcp::transports::stdio::Transport as StdioTransport;
```

### Rationale

- Follows Rust stdlib patterns (`std::io::Error` not `IoError`)
- Module path provides context
- Reduces redundancy
- Cleaner API surface

## 6. HTTP Library Choice

### Decision: Hyper over Reqwest

```rust
// Using hyper directly
use hyper::{Client as HyperClient, Body};
use hyper_tls::HttpsConnector;

pub struct Transport {  // http::Transport
    client: HyperClient<HttpsConnector<HttpConnector>>,
    // Handles both regular JSON and SSE responses
}
```

### Rationale

**Problems with reqwest:**
- Poor Server-Sent Events (SSE) support
- Abstracts away too much for our needs
- Additional dependency overhead
- Issues discovered in shadowcat development

**Why hyper works:**
- Proven in shadowcat's current implementation
- Direct control over HTTP/2 features
- Better SSE handling
- Less overhead
- More flexibility for streaming

## 7. Single MCP Crate Structure

### Decision: One Crate with Organized Modules

```
crates/mcp/
├── src/
│   ├── lib.rs           # Public API
│   ├── client.rs        # Client implementation
│   ├── server.rs        # Server implementation
│   ├── interceptor.rs  # Interceptor system
│   └── transports/      # Transport implementations
│       ├── mod.rs       # Transport trait
│       ├── stdio.rs
│       └── http/
│           └── streaming/
│               └── sse.rs
```

### Rationale

**Why not separate crates (mcp-core, mcp-client, mcp-server)?**
- Unnecessary complexity for tightly coupled components
- More overhead managing versions and dependencies
- Harder to refactor across boundaries

**Benefits of single crate:**
- Simpler dependency management
- Easier to maintain and refactor
- Can still have clear module boundaries
- Reduces workspace complexity

## Implementation Priority

1. **Phase 1: Core Extraction**
   - Transport trait definition
   - Basic Client and Server structs
   - Protocol types (JsonRpcRequest, etc.)

2. **Phase 2: Transport Implementation**
   - StdioTransport (from shadowcat)
   - SseTransport with hyper (from shadowcat)
   - Transport convenience builders

3. **Phase 3: Interceptor System**
   - Interceptor trait and chain
   - Basic interceptors (logging, metrics)
   - Integration with Client/Server

4. **Phase 4: Integration**
   - Update shadowcat to use MCP crate
   - Build compliance framework using MCP crate
   - Test matrix implementation

## Trade-offs Acknowledged

### What we're optimizing for:
- **Code reuse** - DRY principle for protocol logic
- **Type safety** - Leverage Rust's type system
- **Performance** - Zero-cost abstractions where possible
- **Maintainability** - Clear separation of concerns

### What we're accepting:
- **Some generics complexity** - Client<T> and Server<H> have type parameters
- **Not pure OOP** - Mix of traits and concrete types
- **Shadowcat patterns** - Following existing conventions even if not ideal

## Validation Approach

The architecture will be validated through:

1. **Extraction from shadowcat** - Proves the design works with real code
2. **Compliance matrix testing** - Tests interoperability
3. **Performance benchmarks** - Ensures no regression
4. **Integration tests** - Validates all transports work correctly

## Future Considerations

1. **WebSocket Support** - Architecture supports adding `http::streaming::websockets`
2. **HTTP/3** - Can add as new transport when needed
3. **Custom Transports** - Users can implement Transport trait
4. **More Interceptors** - Easy to add new processing stages

---

*Document created: 2025-08-24*
*Decisions based on: Shadowcat analysis, MCP spec requirements, Rust best practices*
*Key insight: Hybrid architecture balances simplicity with flexibility*