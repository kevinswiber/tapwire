# Task A.1: Design Test Harness Architecture

## Objective
Design a comprehensive test harness architecture for Shadowcat E2E tests that enables:
- Easy test writing with minimal boilerplate
- Reliable process management
- Comprehensive log analysis
- Deterministic execution
- Fast parallel execution

## Key Design Requirements

### Functional Requirements
1. **Process Management**
   - Spawn upstream MCP servers (stdio, HTTP)
   - Spawn Shadowcat proxy instances
   - Spawn test clients
   - Coordinate startup order
   - Ensure cleanup on failure

2. **Configuration**
   - Flexible proxy configuration
   - Dynamic port allocation
   - Environment variable management
   - Log level control

3. **Observability**
   - Capture all process output
   - Parse structured logs
   - Extract metrics
   - Trace request flows

4. **Assertions**
   - Response validation
   - Log pattern matching
   - Performance assertions
   - Resource usage checks

### Non-Functional Requirements
- Tests complete in <30 seconds each
- Parallel execution safe
- Clear error diagnostics
- Minimal test flakiness
- CI/CD compatible

## Architecture Components

### Core Structures
```rust
// Main test harness
pub struct TestHarness {
    config: TestConfig,
    processes: ProcessManager,
    ports: PortAllocator,
    logs: LogCollector,
    client: TestClient,
    _cleanup: CleanupGuard,
}

// Process abstraction
pub struct ManagedProcess {
    name: String,
    child: Child,
    stdout: OutputCollector,
    stderr: OutputCollector,
    ready_check: Box<dyn Fn() -> BoxFuture<'static, bool>>,
    shutdown: ShutdownMethod,
}

// Test scenario builder
pub struct TestScenario {
    upstream: UpstreamConfig,
    proxy: ProxyConfig,
    client: ClientConfig,
    assertions: Vec<Box<dyn TestAssertion>>,
}
```

### Key Design Decisions

#### 1. Process Lifecycle
```rust
// Option A: Explicit lifecycle management
let upstream = harness.spawn_upstream().await?;
let proxy = harness.spawn_proxy().await?;
// ... test ...
upstream.shutdown().await?;
proxy.shutdown().await?;

// Option B: RAII with Drop
{
    let harness = TestHarness::new().await?;
    // ... test ...
} // Automatic cleanup

// Option C: Hybrid with explicit and automatic
let harness = TestHarness::new().await?;
// ... test ...
harness.shutdown().await?; // Graceful
// Drop handles forceful cleanup if needed
```

**Decision**: [Choose approach with rationale]

#### 2. Port Allocation Strategy
```rust
// Option A: OS-assigned (bind to 0)
let listener = TcpListener::bind("127.0.0.1:0")?;
let port = listener.local_addr()?.port();

// Option B: Port pool management
let port = port_manager.allocate().await?;
defer! { port_manager.release(port); }

// Option C: Library (portpicker)
let port = portpicker::pick_unused_port()?;
```

**Decision**: [Choose approach with rationale]

#### 3. Log Collection
```rust
// Option A: Intercepted via tracing
let (collector, handle) = LogCollector::new();
tracing::subscriber::set_global_default(collector)?;

// Option B: Process stdout/stderr capture
let output = process.stdout.collect().await?;
let logs = parse_logs(&output)?;

// Option C: File-based with tail
process.log_to_file(&log_path)?;
let logs = tail_file(&log_path)?;
```

**Decision**: [Choose approach with rationale]

#### 4. Test Organization
```
tests/e2e/
├── harness/
│   ├── mod.rs           # Public API
│   ├── process.rs       # Process management
│   ├── client.rs        # Test client
│   ├── assertions.rs    # Assertion helpers
│   └── fixtures.rs      # Test data
├── scenarios/
│   ├── basic.rs         # Basic proxy tests
│   ├── streaming.rs     # SSE tests
│   ├── errors.rs        # Error handling
│   └── performance.rs   # Perf tests
└── lib.rs              # Test configuration
```

## API Design

### Test Writing Experience
```rust
#[tokio::test]
async fn test_mcp_request_routing() -> Result<()> {
    // Simple API for common cases
    shadowcat_e2e::test(|harness| async move {
        let response = harness
            .send_mcp_request(json!({
                "jsonrpc": "2.0",
                "method": "test",
                "params": {},
                "id": 1
            }))
            .await?;
        
        assert_eq!(response.status(), 200);
        harness.assert_log("Forwarding request to upstream")?;
        Ok(())
    }).await
}

#[tokio::test] 
async fn test_complex_scenario() -> Result<()> {
    // Builder API for complex cases
    let scenario = TestScenario::builder()
        .upstream(
            McpServer::stdio("~/src/modelcontextprotocol/servers/everything")
                .with_env("DEBUG", "true")
        )
        .proxy(
            ProxyConfig::reverse()
                .with_rate_limiting(10)
                .with_recording("test.tape")
        )
        .client(
            TestClient::with_timeout(Duration::from_secs(30))
        )
        .build();
    
    scenario.run(|ctx| async move {
        // Complex test logic
    }).await
}
```

## Implementation Plan

### Phase 1: Core Infrastructure
1. Process management layer
2. Port allocation system
3. Basic cleanup handling

### Phase 2: Harness Implementation
1. TestHarness structure
2. Builder pattern
3. Configuration management

### Phase 3: Utilities
1. Log collector
2. Assertion helpers
3. Fixture management

### Phase 4: Integration
1. Scenario builder
2. Helper macros
3. Documentation

## Error Handling Strategy
- Use `anyhow::Result` for test functions
- Capture panic output
- Include logs in error context
- Cleanup even on panic

## Performance Considerations
- Reuse processes where possible
- Parallel test execution
- Lazy log parsing
- Resource pooling

## Success Criteria
- [ ] Complete architecture documented
- [ ] All design decisions made with rationale
- [ ] API examples provided
- [ ] Implementation plan clear
- [ ] Compatible with existing tests