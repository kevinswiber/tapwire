# Transport Architecture Investigation Plan

## Current Pause Point

As of 2025-08-24, we've completed:
- Phase B: Core MCP library extraction (all tasks)
- Phase C.0: HTTP transport with SSE support
- Phase C.1: Interceptor system extraction

We're pausing to investigate a fundamental architectural question about Transport design.

## The Core Questions

### 1. Transport Trait Design

**Current Implementation:**
```rust
#[async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn send(&mut self, message: Value) -> Result<()>;
    async fn receive(&mut self) -> Result<Value>;
    async fn close(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
}
```

**Questions:**
- Should this be split into `IncomingTransport` and `OutgoingTransport`?
- Are the semantics different enough to warrant separation?
- How does shadowcat handle this distinction?

### 2. Subprocess Management

**Current Situation:**
- We have `StdioTransport` - reads/writes process's own stdin/stdout
- We have `SubprocessTransport` - spawns and manages a subprocess
- We have `HttpTransport` - HTTP client with SSE support

**The Dilemma:**

For a client connecting to a stdio server, there are two approaches:

**Approach A: Client Manages Subprocess**
```rust
// Client spawns the server
let transport = SubprocessTransport::new("npx")
    .arg("@modelcontextprotocol/server-everything");
let client = Client::new(transport, handler);
```

**Approach B: External Process Management**
```rust
// User/orchestrator spawns both, passes streams
let (stdin, stdout) = external_process_manager.get_streams();
let transport = StdioTransport::from_streams(stdin, stdout);
let client = Client::new(transport, handler);
```

### 3. Official Rust SDK Pattern

We need to investigate `~/src/modelcontextprotocol/rust-sdk` to understand:
- How does `rmcp` handle transports?
- Do they separate client/server transports?
- How do they handle subprocess management?
- What's their approach to AsyncRead/AsyncWrite?

## Investigation Tasks

### Task 1: Review Official Rust SDK
- [ ] Check rmcp transport traits
- [ ] Look for subprocess handling
- [ ] Understand their client/server separation
- [ ] Note AsyncRead/AsyncWrite patterns

### Task 2: Review Shadowcat's Approach
- [ ] How does `IncomingTransport` vs `OutgoingTransport` work?
- [ ] What's the semantic difference?
- [ ] Is this complexity necessary for MCP library?

### Task 3: Consider Use Cases

**MCP Server Use Cases:**
1. Stdio server - reads/writes own stdin/stdout
2. HTTP server - accepts HTTP requests, returns JSON or SSE

**MCP Client Use Cases:**
1. Connect to HTTP server - straightforward
2. Connect to stdio server - needs subprocess OR external management
3. Testing - might want to use in-memory channels

### Task 4: Design Implications

**If we split Transport:**
```rust
// Server uses IncomingTransport
pub struct Server<I: IncomingTransport, H: ServerHandler>

// Client uses OutgoingTransport  
pub struct Client<O: OutgoingTransport, H: ClientHandler>
```

**If we keep unified Transport:**
```rust
// Both use same trait (current design)
pub struct Server<T: Transport, H: ServerHandler>
pub struct Client<T: Transport, H: ClientHandler>
```

## Architectural Principles to Consider

1. **Simplicity**: Don't add complexity unless necessary
2. **Flexibility**: Support common use cases elegantly
3. **Separation of Concerns**: Transport vs Process Management
4. **Type Safety**: Leverage Rust's type system appropriately
5. **Proxy Independence**: MCP library shouldn't be proxy-aware

## Potential Recommendations

### Option 1: Keep Unified, Remove Subprocess
- Single `Transport` trait
- Remove `SubprocessTransport`
- Let applications handle process management
- Provide examples of subprocess usage

### Option 2: Split Transport, Keep Simple
- `ClientTransport` and `ServerTransport` traits
- Different semantics for each direction
- No subprocess in core library

### Option 3: AsyncRead/AsyncWrite Based
- Transport accepts `AsyncRead + AsyncWrite`
- Maximum flexibility
- Users can pass any stream type
- Subprocess becomes a stream provider, not transport

## Next Steps

1. Investigate official Rust SDK (2 hours)
2. Document findings
3. Make architectural decision
4. Refactor if needed
5. Continue with Phase C.2 (batch support)

## Decision Criteria

The chosen design should:
- [ ] Be simple to understand and use
- [ ] Support all MCP communication patterns
- [ ] Not impose unnecessary process management
- [ ] Allow for testing with mock transports
- [ ] Be consistent with Rust ecosystem patterns
- [ ] Not duplicate shadowcat's proxy complexity