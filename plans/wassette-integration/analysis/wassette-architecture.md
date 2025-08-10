# Wassette Technical Architecture

## Core Components

### 1. MCP Server Implementation (`src/main.rs`)
- **Framework**: Built on `rmcp` (Rust MCP SDK)
- **Server Type**: Implements `ServerHandler` trait
- **Capabilities**: Tools only (no resources or prompts currently)
- **Session Management**: Handled by rmcp layer

### 2. Lifecycle Manager (`crates/wassette/src/lib.rs`)
- **Purpose**: Manages WebAssembly component lifecycle
- **Key Responsibilities**:
  - Component loading from file/OCI/HTTP
  - Component registration and tool mapping
  - Policy enforcement
  - WASI state creation per invocation

### 3. Transport Layer
- **Supported Transports**:
  - **stdio** (default): Process stdin/stdout communication
  - **HTTP/SSE**: Server-Sent Events at port 9001
- **Implementation**: Uses `rmcp::transport` module
- **Logging**: stderr for stdio to avoid protocol interference

### 4. WebAssembly Runtime
- **Engine**: Wasmtime with component model support
- **Linker**: Pre-configured with WASI capabilities
- **Instance Model**: Pre-instantiation for performance
- **Async Support**: Full async/await support enabled

### 5. Component Registry
- **Tool Mapping**: Maps tool names to component functions
- **Function Identifiers**: Supports interface.function notation
- **Component Storage**: In-memory HashMap with Arc wrappers
- **Hot Reload**: Components can be replaced at runtime

## MCP Implementation

### Supported Methods
- `initialize`: Protocol negotiation and capabilities
- `tools/list`: Returns available WebAssembly tools
- `tools/call`: Executes WebAssembly component functions
- `prompts/list`: Placeholder (returns empty)
- `resources/list`: Placeholder (returns empty)

### Protocol Details
- **Version**: Uses rmcp's protocol version
- **Session Headers**: `Mcp-Session-Id` handled by rmcp
- **Message Format**: JSON-RPC 2.0
- **Error Handling**: Structured `ErrorData` responses

### Message Flow
1. Transport receives MCP request (stdio/HTTP)
2. rmcp parses and validates JSON-RPC message
3. McpServer routes to appropriate handler
4. LifecycleManager locates component and function
5. WASI state created from policy template
6. WebAssembly component instantiated and invoked
7. Results serialized back through rmcp

## WebAssembly Integration

### Component Model
- **Format**: WebAssembly Component Model (not core modules)
- **Target**: `wasm32-wasip2` compilation target
- **Interface**: WIT (WebAssembly Interface Types) definitions
- **Bindings**: Auto-generated from WIT files

### Component Structure
```wit
package component:example;

world example-world {
    export function-name: func(params) -> result<return-type, string>;
}
```

### Supported Languages
- **Rust**: Native support via cargo-component
- **JavaScript**: Via ComponentizeJS
- **Python**: Via componentize-py
- **Go**: Via WASI SDK

### Function Discovery
- Components analyzed on load for exported functions
- Automatic JSON schema generation from WIT types
- Tool metadata includes descriptions from WIT comments

## Component Lifecycle

### Loading Process
1. **URI Resolution**: file://, oci://, https:// schemes
2. **Download**: To staging directory if remote
3. **Compilation**: Component::new() with Wasmtime
4. **Pre-instantiation**: Linker creates InstancePre
5. **Registration**: Tools added to registry
6. **Persistence**: Copied to plugin directory

### Execution Flow
1. Tool invocation request received
2. Component located in registry
3. WASI state created from policy
4. Store created with WASI state
5. Component instantiated
6. Function invoked with JSON parameters
7. Results converted back to JSON

### Unloading
- Component removed from registry
- Files deleted from plugin directory
- Policy associations cleaned up
- In-flight requests allowed to complete

## Security Architecture

### Sandbox Boundaries
- **Wasmtime Isolation**: Browser-grade sandboxing
- **Memory Isolation**: Each invocation gets fresh memory
- **Resource Limits**: Configurable via WASI
- **No Ambient Authority**: Deny-by-default

### Capability System
- **File System**: Path-based allow lists
- **Network**: Host/CIDR allow lists  
- **Environment**: Specific variable access
- **Process**: No process spawning capability

### Policy Enforcement
- Policies parsed from YAML files
- Per-component policy association
- WASI state template created from policy
- Runtime enforcement by Wasmtime

## Storage and Persistence

### Plugin Directory
- **Default**: `$XDG_DATA_HOME/wassette/components`
- **Structure**:
  ```
  components/
  ├── component-name.wasm
  ├── component-name.policy.yaml
  └── downloads/  # Staging area
  ```

### Component Discovery
- Automatic loading on startup
- .wasm files in plugin directory
- Co-located .policy.yaml files restored

### OCI Registry Support
- Uses `oci-wasm` client
- Standard OCI distribution spec
- Authentication via Docker config

## Performance Characteristics

### Startup
- Component compilation on first load
- Pre-instantiation for fast invocation
- Parallel component loading on startup

### Runtime
- Per-invocation instantiation overhead
- Shared engine and linker across components
- Async execution with tokio runtime

### Memory
- Component memory isolated per invocation
- No sharing between invocations
- Cleanup after each call

## Limitations and Constraints

### Current Limitations
- No streaming support (request/response only)
- No component-to-component calls
- No persistent state between invocations
- Single transport at a time (not simultaneous)

### Transport Constraints  
- stdio: Single client, process lifetime
- HTTP: Multiple clients, stateless
- No WebSocket transport currently

### Component Constraints
- Must target wasm32-wasip2
- Must use Component Model (not modules)
- No native code execution
- Limited to WASI capabilities

## Integration Considerations

### For Proxy Integration
- **Primary Transport**: stdio for simplicity
- **Message Interception**: At JSON-RPC level
- **Session Tracking**: Via rmcp session IDs
- **Component Loading**: Via file:// URIs initially

### Key Integration Points
1. **Transport Layer**: Proxy stdio streams
2. **Message Level**: Intercept JSON-RPC messages
3. **Component Loading**: Proxy OCI/HTTP fetches
4. **Policy Management**: External policy configuration
5. **Monitoring**: Hook into lifecycle events