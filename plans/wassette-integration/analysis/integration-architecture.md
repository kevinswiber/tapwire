# Wassette-Shadowcat Integration Architecture

## Integration Patterns

### Pattern 1: Upstream Proxy (Process Spawn)

```
┌────────┐     stdio     ┌───────────┐     stdio     ┌──────────┐
│ Client ├──────────────►│ Shadowcat ├──────────────►│ Wassette │
└────────┘               │   Proxy   │               │ Process  │
                         └─────┬─────┘               └──────────┘
                               │
                         ┌─────▼─────┐
                         │ Recording │
                         │ Intercept │
                         └───────────┘
```

#### Implementation
```rust
// Shadowcat spawns Wassette as child process
let wassette_cmd = Command::new("wassette")
    .args(["serve", "--stdio"])
    .arg("--plugin-dir").arg(plugin_dir);

let wassette_transport = StdioTransport::new(wassette_cmd);
let proxy = ForwardProxy::new(client_transport, wassette_transport);
```

#### Pros
- ✅ Simple, clean integration
- ✅ Full lifecycle control
- ✅ Complete message visibility
- ✅ Easy recording/replay
- ✅ Process isolation per client

#### Cons
- ❌ Process overhead per client
- ❌ Component reload requires restart
- ❌ No component sharing between clients
- ❌ Higher memory usage

#### Use Cases
- Development environments
- Testing and debugging
- Security-sensitive deployments
- Single-tenant scenarios

### Pattern 2: HTTP Reverse Proxy

```
┌────────┐     HTTP      ┌───────────┐     HTTP      ┌──────────┐
│ Client ├──────────────►│ Shadowcat ├──────────────►│ Wassette │
└────────┘               │   Proxy   │               │  Server  │
                         └─────┬─────┘               └──────────┘
                               │                       Port 9001
                         ┌─────▼─────┐
                         │  Session  │
                         │  Manager  │
                         └───────────┘
```

#### Implementation
```rust
// Wassette runs as HTTP server
// wassette serve --http

// Shadowcat proxies HTTP requests
let wassette_upstream = HttpTransport::connect("http://localhost:9001");
let proxy = ReverseProxy::new(wassette_upstream);
```

#### Pros
- ✅ Multiple concurrent clients
- ✅ Component sharing
- ✅ Hot reload without restart
- ✅ Lower memory per client
- ✅ Standard HTTP tooling

#### Cons
- ❌ Additional network hop
- ❌ Session management complexity
- ❌ Requires HTTP transport support
- ❌ Potential port conflicts

#### Use Cases
- Production deployments
- Multi-tenant environments
- High-concurrency scenarios
- Cloud deployments

### Pattern 3: Embedded Library (Future)

```
┌────────┐              ┌─────────────────────┐
│ Client ├─────────────►│      Shadowcat      │
└────────┘              │ ┌─────────────────┐ │
                        │ │ Wassette Library│ │
                        │ │   (embedded)    │ │
                        │ └─────────────────┘ │
                        └─────────────────────┘
```

#### Implementation
```rust
// Theoretical - Wassette as library
use wassette::{LifecycleManager, McpServer};

let lifecycle = LifecycleManager::new(plugin_dir).await?;
let server = McpServer::new(lifecycle);
// Direct function calls, no IPC
```

#### Pros
- ✅ Lowest latency
- ✅ No IPC overhead
- ✅ Shared memory access
- ✅ Deepest integration

#### Cons
- ❌ Requires Wassette refactoring
- ❌ Rust-only integration
- ❌ Complex error isolation
- ❌ No process boundary

#### Use Cases
- Performance-critical applications
- Embedded systems
- Custom implementations

### Pattern 4: Sidecar Architecture

```
┌────────┐──────────────►┌───────────┐
│ Client │               │ Shadowcat │
└────────┘               └─────┬─────┘
                               │ Metrics/Logs
                         ┌─────▼─────┐
┌────────┐──────────────►┌──────────┐
│ Client │     Direct    │ Wassette │
└────────┘               └──────────┘
```

#### Implementation
```rust
// Shadowcat monitors Wassette without proxying
let monitor = SidecarMonitor::new();
monitor.attach_to_wassette(wassette_pid);
monitor.collect_metrics();
```

#### Pros
- ✅ Zero latency overhead
- ✅ Non-invasive monitoring
- ✅ Independent scaling
- ✅ Gradual adoption

#### Cons
- ❌ No message interception
- ❌ Limited control
- ❌ Requires correlation
- ❌ Complex deployment

#### Use Cases
- Observability-only needs
- Existing Wassette deployments
- Compliance monitoring

## Recommended Approach

### Primary: Pattern 1 (Upstream Proxy) for Development

**Rationale**:
1. Simplest to implement and debug
2. Full control over lifecycle
3. Complete message visibility
4. Natural recording point
5. Security through isolation

### Secondary: Pattern 2 (HTTP Proxy) for Production

**Rationale**:
1. Better resource utilization
2. Multi-client support
3. Standard deployment model
4. Cloud-native approach
5. Hot reload capability

## Component Lifecycle Management

### Loading Through Proxy

```rust
impl ProxyComponentLoader {
    async fn load_component(&self, uri: &str) -> Result<()> {
        match parse_uri(uri) {
            // Intercept OCI pulls
            Uri::Oci(reference) => {
                self.record_pull(reference);
                self.verify_signature(reference).await?;
                self.forward_to_wassette(uri).await
            }
            // Local files pass through
            Uri::File(path) => {
                self.forward_to_wassette(uri).await
            }
        }
    }
}
```

### OCI Registry Integration

1. **Proxy as Registry Mirror**
   - Cache pulled components
   - Verify signatures
   - Apply policies

2. **Authentication Flow**
   - Shadowcat handles registry auth
   - Wassette receives authenticated pulls
   - Token isolation maintained

### Version Management

```yaml
# Component version policy
components:
  - name: "fetch-rs"
    version: "1.0.0"
    auto_update: false
    source: "oci://registry/fetch:1.0.0"
```

## Recording and Replay

### Recording Architecture

```rust
struct WassetteRecording {
    session_id: SessionId,
    component_loads: Vec<ComponentLoad>,
    tool_invocations: Vec<ToolCall>,
    timing: Vec<Timestamp>,
}

impl Recorder {
    async fn record_tool_call(&mut self, call: ToolCall) {
        self.tape.write(Entry {
            timestamp: Instant::now(),
            message: call.to_message(),
            component_state: self.capture_state(),
        }).await;
    }
}
```

### Storage Format

```sql
CREATE TABLE recordings (
    id INTEGER PRIMARY KEY,
    session_id TEXT,
    timestamp INTEGER,
    message_type TEXT,
    message BLOB,
    component_id TEXT,
    policy BLOB
);
```

### Replay Mechanism

```rust
impl ReplayEngine {
    async fn replay(&self, tape: Tape) -> Result<()> {
        // Restore component state
        for load in tape.component_loads {
            self.wassette.load_component(&load.uri).await?;
            self.wassette.attach_policy(&load.policy).await?;
        }
        
        // Replay tool calls with timing
        for entry in tape.entries {
            sleep_until(entry.timestamp).await;
            let result = self.wassette.call_tool(entry.call).await?;
            self.verify_determinism(entry.expected, result)?;
        }
    }
}
```

### State Management

**Challenges**:
- WebAssembly components are stateless
- External state (files, network) may differ
- Non-deterministic operations (time, random)

**Solutions**:
1. Record external state snapshots
2. Mock non-deterministic operations
3. Provide replay-mode WASI capabilities

## Interception Capabilities

### Message Interception Points

```rust
enum InterceptionPoint {
    PreComponentLoad,    // Before loading component
    PostComponentLoad,   // After loading
    PreToolCall,        // Before invoking tool
    PostToolCall,       // After tool returns
    PolicyCheck,        // During policy evaluation
}
```

### Modification Capabilities

```rust
impl Interceptor for WassetteInterceptor {
    async fn intercept(&self, point: InterceptionPoint, data: &mut Data) -> Action {
        match point {
            PreToolCall => {
                // Modify parameters
                if let Some(params) = data.params_mut() {
                    self.sanitize_params(params);
                }
            }
            PostToolCall => {
                // Filter results
                if let Some(result) = data.result_mut() {
                    self.redact_sensitive(result);
                }
            }
            PolicyCheck => {
                // Additional policy enforcement
                if !self.custom_policy_check(data) {
                    return Action::Block("Policy violation");
                }
            }
        }
        Action::Continue
    }
}
```

### Debugging Tools

```rust
impl DebugInterceptor {
    async fn intercept(&self, message: &Message) -> Action {
        // Breakpoint functionality
        if self.breakpoint_matches(message) {
            println!("🔴 Breakpoint hit: {}", message);
            self.wait_for_continue().await;
        }
        
        // Message inspection
        self.log_structured(message);
        
        // State snapshot
        self.capture_snapshot();
        
        Action::Continue
    }
}
```

### Security Considerations

**Capability-Aware Interception**:
```rust
// Interceptor respects Wassette's capability model
impl SecurityInterceptor {
    async fn check_allowed(&self, component: &str, operation: &str) -> bool {
        let policy = self.get_policy(component).await?;
        policy.allows(operation) && self.proxy_policy.allows(operation)
    }
}
```

## Implementation Roadmap

### Phase 1: Basic Integration (Week 1)
✅ **Minimal Viable Proxy**
- [ ] Stdio transport integration
- [ ] Basic message forwarding
- [ ] Process lifecycle management
- [ ] Error handling

**Deliverable**: `shadowcat forward stdio --upstream wassette`

### Phase 2: Recording (Week 2)
📼 **Recording Capabilities**
- [ ] Message capture
- [ ] Component state tracking
- [ ] Tape storage format
- [ ] Basic replay

**Deliverable**: Recording and replay of Wassette sessions

### Phase 3: Interception (Week 3)
🔍 **Advanced Features**
- [ ] Interceptor chain
- [ ] Policy integration
- [ ] Debug tooling
- [ ] Performance optimization

**Deliverable**: Full-featured proxy with interception

### Phase 4: Production (Week 4)
🚀 **Production Hardening**
- [ ] HTTP transport support
- [ ] Multi-client handling
- [ ] Monitoring and metrics
- [ ] Documentation

**Deliverable**: Production-ready integration

## Technical Decisions

### Integration Pattern
**Decision**: Start with Pattern 1 (Upstream Proxy)
**Rationale**: Simplest implementation, most control, natural evolution path

### Transport Protocol
**Decision**: stdio first, HTTP later
**Rationale**: Wassette's primary transport, simpler initial implementation

### Recording Format
**Decision**: SQLite with JSON messages
**Rationale**: Structured queries, portable format, good tooling

### Interception Model
**Decision**: Chain of Responsibility pattern
**Rationale**: Flexible, composable, testable

## Performance Considerations

### Latency Budget
```
Client → Shadowcat:      1ms  (network/IPC)
Shadowcat processing:    1ms  (proxy logic)
Shadowcat → Wassette:    1ms  (stdio IPC)
Wassette processing:     5ms  (component invoke)
Return path:            3ms  (same in reverse)
─────────────────────────────────────────
Total overhead:         6ms  (< 10% target)
```

### Optimization Strategies

1. **Connection Pooling**: Reuse Wassette processes
2. **Message Batching**: Group related calls
3. **Async Processing**: Parallel component loads
4. **Caching**: Component metadata and schemas
5. **Zero-Copy**: Direct buffer passing where possible

### Scalability Limits

| Metric | Single Process | HTTP Pool | Target |
|--------|---------------|-----------|--------|
| Concurrent Clients | 1 | 1000 | 100 |
| Msg/sec | 1000 | 10000 | 5000 |
| Latency p95 | 10ms | 15ms | 20ms |
| Memory/Client | 50MB | 5MB | 10MB |

## Conclusion

The Wassette-Shadowcat integration provides a powerful platform for:

1. **Development**: Full debugging and inspection capabilities
2. **Security**: Multi-layer defense with audit trail
3. **Operations**: Recording, replay, and monitoring
4. **Flexibility**: Multiple deployment patterns

**Recommended Implementation Path**:
1. Start with stdio proxy (Pattern 1)
2. Add recording and replay
3. Implement interception
4. Expand to HTTP (Pattern 2)
5. Optimize for production

This architecture maintains Wassette's security guarantees while adding Shadowcat's powerful proxy capabilities, creating a best-of-both-worlds solution for MCP tool execution.