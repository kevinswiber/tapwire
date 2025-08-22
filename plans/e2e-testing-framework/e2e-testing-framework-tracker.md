# End-to-End Testing Framework Tracker

## Overview
Implement comprehensive end-to-end testing infrastructure for Shadowcat that validates complete proxy flows with real upstream servers, clients, and full message routing. Tests should launch actual processes, verify logs, and ensure the entire system works correctly under realistic conditions.

## Latest Updates (2025-08-22)
- ✅ **Phase A Complete**: All infrastructure foundation tasks completed
- ✅ **Phase B Complete**: Test harness fully implemented with process management, port allocation, and cleanup
- ✅ **MCP Validator Integration**: Successfully integrated MCP validator for protocol compliance testing
- ✅ **Authentication Tests**: Working with proper Bearer token authentication
- 🔶 **Phase C Started**: Basic proxy flow and authentication tests passing
- 📝 **Key Achievements**:
  - TestHarness with automatic process cleanup via Drop trait
  - Dynamic port allocation with OS-assigned ports
  - MCP compliance tests passing with correct protocol version (2025-03-26)
  - Conditional tracing_subscriber for clean test output
  - Proper error handling per MCP spec (400 for missing session)

## Goals
- [ ] Test complete proxy flows with real MCP servers and clients
- [ ] Validate all transport types (stdio, HTTP, SSE) end-to-end
- [ ] Capture and analyze debug/trace logs for correctness
- [ ] Test error scenarios and recovery behaviors
- [ ] Verify performance characteristics under load
- [ ] Ensure resource cleanup and no process leaks
- [ ] Enable CI/CD integration with comprehensive coverage

## Success Criteria
- E2E tests catch 90%+ of integration issues before release
- Tests run in <5 minutes for full suite
- Zero flaky tests (100% deterministic)
- Parallel execution where possible
- Clear failure diagnostics with log capture
- Works in CI environment (GitHub Actions)

## Rust E2E Testing Patterns

### Common Structures in Rust Projects
```
tests/
├── e2e/                    # End-to-end tests
│   ├── common/            # Shared test infrastructure
│   │   ├── harness.rs     # Test harness and helpers
│   │   ├── fixtures.rs    # Test data and configs
│   │   └── processes.rs   # Process management
│   ├── scenarios/         # Test scenarios
│   │   ├── basic_proxy.rs
│   │   ├── sse_streaming.rs
│   │   └── error_recovery.rs
│   └── mod.rs
├── integration/           # Existing integration tests
└── fixtures/             # Test data files
    ├── configs/
    └── messages/
```

### Key Libraries Used
- **Process Management**: `tokio::process`, `assert_cmd`, `duct`
- **Port Allocation**: `portpicker`, `port_check`
- **Log Capture**: `tracing-subscriber`, `test-log`, `env_logger`
- **Assertions**: `assert_cmd`, `predicates`, `pretty_assertions`
- **Test Organization**: `serial_test`, `test-context`, `rstest`
- **Cleanup**: `tempfile`, `scopeguard`

## Phases

### Phase A: Infrastructure Foundation (12 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| A.0 | Research Rust E2E patterns | 2h | ✅ | None | 2025-08-22 |
| A.1 | Design test harness architecture | 3h | ✅ | A.0 | 2025-08-22 |
| A.2 | Create process management layer | 3h | ✅ | A.1 | 2025-08-22 |
| A.3 | Implement log capture system | 2h | ✅ | A.1 | 2025-08-22 |
| A.4 | Build port allocation manager | 2h | ✅ | A.1 | 2025-08-22 |

### Phase B: Test Harness Implementation (16 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| B.0 | Create TestHarness struct | 3h | ✅ | A.1-A.4 | 2025-08-22 |
| B.1 | Implement upstream server spawning | 3h | ✅ | B.0 | 2025-08-22 |
| B.2 | Implement proxy spawning | 3h | ✅ | B.0 | 2025-08-22 |
| B.3 | Create test client builder | 3h | ✅ | B.0 | 2025-08-22 |
| B.4 | Add timing coordination | 2h | ✅ | B.1-B.3 | 2025-08-22 |
| B.5 | Implement cleanup handlers | 2h | ✅ | B.1-B.3 | 2025-08-22 |

### Phase C: Core Test Scenarios (20 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| C.0 | Basic proxy flow test | 3h | ✅ | B.0-B.5 | 2025-08-22 |
| C.1 | SSE streaming test | 4h | ⬜ | B.0-B.5 | |
| C.2 | Connection pooling test | 3h | ⬜ | B.0-B.5 | |
| C.3 | Error recovery test | 3h | 🔶 | B.0-B.5 | Partial |
| C.4 | Multiple clients test | 3h | ⬜ | B.0-B.5 | |
| C.5 | Rate limiting test | 2h | ⬜ | B.0-B.5 | |
| C.6 | Authentication flow test | 2h | ✅ | B.0-B.5 | 2025-08-22 |

### Phase D: Advanced Scenarios (16 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| D.0 | Graceful shutdown test | 2h | ⬜ | C.0 | |
| D.1 | Circuit breaker test | 3h | ⬜ | C.0 | |
| D.2 | Recording/replay test | 3h | ⬜ | C.0 | |
| D.3 | Load/performance test | 4h | ⬜ | C.0-C.4 | |
| D.4 | Chaos testing (kill processes) | 2h | ⬜ | C.0 | |
| D.5 | Memory leak detection | 2h | ⬜ | C.0-C.4 | |

### Phase E: Log Analysis & Diagnostics (10 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| E.0 | Log pattern matchers | 3h | ⬜ | A.3 | |
| E.1 | Trace flow analyzer | 3h | ⬜ | E.0 | |
| E.2 | Performance metrics extraction | 2h | ⬜ | E.0 | |
| E.3 | Error diagnostics formatter | 2h | ⬜ | E.0 | |

### Phase F: CI/CD Integration (8 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| F.0 | GitHub Actions workflow | 2h | ⬜ | C.0-C.6 | |
| F.1 | Test parallelization strategy | 2h | ⬜ | F.0 | |
| F.2 | Artifact collection (logs, dumps) | 2h | ⬜ | F.0 | |
| F.3 | Flaky test detection | 2h | ⬜ | F.0 | |

## Total Estimated Time: 82 hours

## Test Scenarios Detail

### Critical Path Tests
1. **Basic MCP Flow**: Client sends request → proxy routes → server responds → proxy returns
2. **SSE Streaming**: Long-lived SSE connections with reconnection
3. **Connection Pooling**: Verify subprocess reuse and cleanup
4. **Error Recovery**: Upstream failures, network issues, invalid messages
5. **Concurrent Clients**: Multiple simultaneous connections
6. **Rate Limiting**: Verify limits are enforced correctly
7. **Authentication**: OAuth flows, JWT validation

### Performance Tests
1. **Latency**: Measure proxy overhead (<5% target)
2. **Throughput**: Messages per second capacity
3. **Memory**: No leaks under sustained load
4. **CPU**: Efficient resource usage

### Chaos Tests
1. **Kill upstream**: Verify failover/circuit breaking
2. **Kill proxy**: Verify graceful shutdown
3. **Network partition**: Split-brain scenarios
4. **Resource exhaustion**: OOM, file descriptor limits

## Key Decisions
- [ ] Test framework: Built-in vs external harness
- [ ] Process management: tokio::process vs duct vs std::process
- [ ] Log capture: tracing-subscriber vs env_logger
- [ ] Port allocation: portpicker vs manual management
- [ ] CI runner: GitHub Actions vs other
- [ ] Parallelization: How many tests can run concurrently
- [ ] Test data: Real MCP servers vs mock servers

## Implementation Patterns

### Test Harness Example
```rust
struct TestHarness {
    upstream: UpstreamProcess,
    proxy: ProxyProcess,
    client: TestClient,
    log_collector: LogCollector,
    port_manager: PortManager,
}

impl TestHarness {
    async fn new() -> Result<Self> {
        // Allocate ports
        // Spawn processes
        // Wait for ready
        // Return harness
    }
    
    async fn scenario() -> TestScenario {
        TestScenario::builder()
            .with_upstream(UpstreamConfig::default())
            .with_proxy(ProxyConfig::default())
            .with_client(ClientConfig::default())
            .build()
    }
}

#[tokio::test]
async fn test_basic_proxy_flow() {
    let harness = TestHarness::new().await.unwrap();
    
    let response = harness.client
        .send_mcp_request(test_request())
        .await
        .unwrap();
    
    assert_eq!(response, expected_response());
    
    // Verify logs
    harness.log_collector
        .assert_pattern("Forwarding request to upstream")
        .assert_no_errors();
}
```

### Process Management Pattern
```rust
struct ManagedProcess {
    child: Child,
    stdout_reader: JoinHandle<Vec<String>>,
    stderr_reader: JoinHandle<Vec<String>>,
    ready_checker: Box<dyn Fn() -> bool>,
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        // Graceful shutdown
        // Kill if needed
        // Collect logs
    }
}
```

## Risks & Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| Flaky tests | High | Proper timing, retries, deterministic setup |
| Port conflicts | Medium | Dynamic allocation, cleanup |
| CI resource limits | Medium | Optimize parallelization, use containers |
| Process leaks | High | Robust cleanup, Drop implementations |
| Slow test suite | Medium | Parallel execution, test sharding |

## Dependencies
- Existing test infrastructure in `tests/`
- Real MCP servers (from ~/src/modelcontextprotocol/servers/)
- Test fixtures and sample data
- CI/CD environment setup

## Resources
- [Rust E2E Testing Best Practices](https://github.com/rust-lang/rust-clippy/tree/master/tests)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- Example: [Vector E2E Tests](https://github.com/vectordotdev/vector/tree/master/tests)
- Example: [Linkerd2 Proxy E2E](https://github.com/linkerd/linkerd2-proxy/tree/main/tests)

## Notes
- Consider using actual MCP reference servers for realistic testing
- Log capture is critical for debugging CI failures
- Need strategy for large SSE streams and long-running tests
- Consider separate `e2e` test profile with optimizations
- May need `#[serial]` attribute for some tests

## Progress Log
- 2025-08-22: Plan created, initial architecture designed
- 2025-08-22: Integrated MCP Validator as git submodule
  - Added `tools/mcp-validator` with reference MCP servers
  - Set up Python environment with uv
  - Tested HTTP reference server
  - Created testing documentation
  - ✅ **Successfully tested Shadowcat reverse proxy** with validator
  - Key insight: `--upstream` must include full path (e.g., `http://localhost:8088/mcp`)
- 2025-08-22: Completed Phase A - Infrastructure Foundation
  - ✅ Researched existing E2E framework in `tests/integration/e2e_framework.rs`
  - ✅ Designed comprehensive test harness architecture
  - ✅ Created process management strategy with health checks
  - ✅ Designed port allocation system with OS assignment
  - ✅ Planned log capture and analysis framework
  - ✅ Defined test organization and naming conventions
  - ✅ Created detailed implementation roadmap
  - 📁 All designs documented in `analysis/` directory