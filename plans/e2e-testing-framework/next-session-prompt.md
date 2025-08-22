# E2E Testing Framework - Next Session Prompt

## Session Focus
Begin Phase A: Research and design the end-to-end testing infrastructure for Shadowcat.

## Tasks for This Session
1. **A.0**: Research Rust E2E testing patterns (2h)
2. **A.1**: Design test harness architecture (3h)
3. **A.2**: Create process management layer (3h)

## Context
You are building a comprehensive E2E testing framework for Shadowcat that will:
- Launch real upstream MCP servers
- Spawn Shadowcat proxy instances with various configurations
- Use test clients to send real MCP traffic
- Capture and analyze debug/trace logs
- Verify end-to-end functionality

## Key Investigation Areas

### Research Focus (A.0)
- Study how major Rust projects structure E2E tests:
  - **Linkerd2-proxy**: `tests/` directory structure
  - **Vector**: Integration test harness
  - **Tokio**: Process spawning patterns
  - **Cargo**: CLI testing approaches
- Identify best practices for:
  - Process lifecycle management
  - Port allocation strategies
  - Log capture mechanisms
  - Test parallelization
  - CI integration

### Architecture Design (A.1)
- Design the `TestHarness` structure
- Plan the test scenario builder pattern
- Define the process abstraction layer
- Design log collection and analysis
- Plan fixture and test data management

### Initial Implementation (A.2)
- Implement `ManagedProcess` struct
- Create process spawning utilities
- Build stdout/stderr capture
- Implement graceful shutdown
- Add health check mechanisms

## Key Files to Review
- `shadowcat/tests/` - Existing integration tests
- `shadowcat/src/process/` - Process management code
- `shadowcat/examples/` - Example servers to test against
- `~/src/modelcontextprotocol/servers/` - Real MCP servers for testing

## Deliverables

1. **Research Document** (`plans/e2e-testing-framework/analysis/rust-e2e-patterns.md`)
   - Survey of Rust E2E testing approaches
   - Recommended libraries and tools
   - Best practices and anti-patterns
   - Decision rationale

2. **Architecture Document** (`plans/e2e-testing-framework/analysis/test-harness-design.md`)
   - TestHarness structure design
   - Process management architecture
   - Log collection strategy
   - Port allocation approach
   - Test organization structure

3. **Initial Implementation** (`shadowcat/tests/e2e/common/`)
   - `process.rs` - Process management utilities
   - `harness.rs` - Basic TestHarness structure
   - `ports.rs` - Port allocation manager

## Commands to Run
```bash
# Examine existing test structure
find shadowcat/tests -type f -name "*.rs" | head -20
rg "#\[tokio::test\]" shadowcat/tests --count

# Look for process spawning patterns
rg "Command::new|spawn|Child" shadowcat/src --type rust

# Check what example servers exist
ls ~/src/modelcontextprotocol/servers/

# Review existing integration test patterns
cat shadowcat/tests/e2e_basic_integration_test.rs | head -100
```

## Success Criteria
- [ ] Clear understanding of Rust E2E testing best practices
- [ ] Detailed test harness architecture designed
- [ ] Basic process management utilities implemented
- [ ] Can spawn and cleanly shut down a test process
- [ ] Can capture stdout/stderr from spawned processes
- [ ] Plan validated against similar Rust projects

## Example Test Structure Goal
```rust
#[tokio::test]
async fn test_complete_mcp_flow() {
    // Setup
    let harness = TestHarness::builder()
        .with_upstream_mcp_server("everything")
        .with_proxy_config(test_config())
        .with_log_level("shadowcat=debug")
        .build()
        .await
        .expect("Failed to setup test harness");
    
    // Execute
    let response = harness.client()
        .send_request(test_mcp_request())
        .await
        .expect("Request failed");
    
    // Verify
    assert_eq!(response.status(), 200);
    assert!(response.json::<McpResponse>().is_valid());
    
    // Check logs
    harness.assert_log_contains("Processing MCP request");
    harness.assert_no_errors();
    
    // Cleanup happens automatically via Drop
}
```

## Notes
- Focus on making tests deterministic and fast
- Consider both happy path and error scenarios
- Plan for parallel test execution from the start
- Ensure proper cleanup even on test failure/panic
- Make diagnostic output clear for CI debugging