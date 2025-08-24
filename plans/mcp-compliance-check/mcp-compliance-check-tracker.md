# MCP Compliance Check Tracker

## Overview

This tracker coordinates the development of a Rust-native MCP compliance testing framework for Shadowcat. After extensive analysis of the Python-based mcp-validator, we've determined that building our own compliance suite will provide better integration, quality control, and proxy-specific testing capabilities.

**Last Updated**: 2025-08-23  
**Total Estimated Duration**: 80-100 hours (increased for library design)  
**Status**: Planning - Architecture Defined

## Goals

1. **Build library-first MCP compliance framework** - Reusable Rust library with CLI wrapper
2. **Targeted protocol coverage** - Support MCP versions 2025-03-26 and 2025-06-18 (Shadowcat's supported versions)
3. **Version-agnostic architecture** - Easily add new MCP versions as they're released (every few months)
4. **Proxy-specific validation** - Test both forward and reverse proxy modes with ~50 proxy-specific tests
5. **Automated testing** - Full CI/CD integration with `cargo test`
6. **Reference implementation validation** - Test against official MCP servers
7. **Shadowcat workspace integration** - Leverage existing transport, protocol, and session code

## Architecture Vision

```
┌─────────────────────────────────────────────────────────────┐
│                    MCP Compliance Framework                  │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Test       │  │   Version    │  │   Reporting  │     │
│  │   Runner     │  │   Registry   │  │   Engine     │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                  │                  │              │
│  ┌──────▼──────────────────▼──────────────────▼───────┐    │
│  │              Core Compliance Engine                 │    │
│  │  - Test orchestration                              │    │
│  │  - Dynamic version loading                         │    │
│  │  - Feature detection                               │    │
│  └──────────────────────┬──────────────────────────┘    │
│                         │                                 │
│  ┌──────────────────────▼──────────────────────────────┐ │
│  │            Pluggable Protocol Adapters              │ │
│  ├────────────────────┬──────────────────────────────┤ │
│  │   v2025_03_26      │      v2025_06_18            │ │
│  │   ┌──────────┐     │     ┌──────────┐           │ │
│  │   │ Features │     │     │ Features │           │ │
│  │   │ - Async  │     │     │ - No batch│          │ │
│  │   │ - Objects│     │     │ - Elicit  │          │ │
│  │   └──────────┘     │     └──────────┘           │ │
│  └────────────────────┴──────────────────────────────┘ │
│                                                           │
│  ┌──────────────────────────────────────────────────────┐ │
│  │                 Test Categories                      │ │
│  ├────────────┬────────────┬────────────┬──────────────┤ │
│  │   Basic    │   Tools    │   Async    │   Proxy      │ │
│  │  Protocol  │   Tests    │Operations │  Specific    │ │
│  └────────────┴────────────┴────────────┴──────────────┘ │
│                                                           │
│  Integration Points:                                      │
│  - Shadowcat Transport Layer                             │
│  - Session Manager                                       │
│  - Interceptor Chain                                     │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

## Work Phases

### Phase A: Analysis & Knowledge Capture (Week 1) - IN PROGRESS
Capture learnings from mcp-validator investigation and design compliance framework

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | **Extract mcp-validator test cases** | 4h | None | ✅ Completed | | [Details](tasks/A.0-extract-validator-tests.md) |
| A.1 | **Analyze MCP spec compliance points** | 3h | None | ✅ Completed | | [Details](tasks/A.1-analyze-mcp-specs.md) |
| A.2 | **Design Rust compliance architecture** | 4h | A.0, A.1 | ⬜ Not Started | | [Details](tasks/A.2-design-architecture.md) |
| A.3 | **Create proxy-specific test scenarios** | 3h | A.1 | ⬜ Not Started | | [Details](tasks/A.3-proxy-test-scenarios.md) |

**Phase A Total**: 14 hours

### Phase B: Core Framework Implementation (Week 1-2)
Build the foundational compliance testing infrastructure

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.0 | **Create compliance test module** | 3h | A.2 | ⬜ Not Started | | [Details](tasks/B.0-create-module.md) |
| B.1 | **Implement test runner** | 4h | B.0 | ⬜ Not Started | | [Details](tasks/B.1-test-runner.md) |
| B.2 | **Build protocol adapters** | 4h | B.0 | ⬜ Not Started | | [Details](tasks/B.2-protocol-adapters.md) |
| B.3 | **Create report generator** | 3h | B.1 | ⬜ Not Started | | [Details](tasks/B.3-report-generator.md) |

**Phase B Total**: 14 hours

### Phase C: Protocol Test Implementation (Week 2)
Implement comprehensive protocol compliance tests

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.0 | **Basic protocol tests** | 4h | B.1, B.2 | ⬜ Not Started | | [Details](tasks/C.0-basic-protocol.md) |
| C.1 | **Tools compliance tests** | 4h | C.0 | ⬜ Not Started | | [Details](tasks/C.1-tools-tests.md) |
| C.2 | **Async operations tests** | 4h | C.1 | ⬜ Not Started | | [Details](tasks/C.2-async-tests.md) |
| C.3 | **SSE transport tests** | 3h | C.0 | ⬜ Not Started | | [Details](tasks/C.3-sse-tests.md) |

**Phase C Total**: 15 hours

### Phase D: Proxy-Specific Testing (Week 2-3)
Implement tests specific to proxy functionality

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.0 | **Session mapping tests** | 3h | C.0 | ⬜ Not Started | | [Details](tasks/D.0-session-mapping.md) |
| D.1 | **Multi-upstream failover tests** | 4h | D.0 | ⬜ Not Started | | [Details](tasks/D.1-failover-tests.md) |
| D.2 | **Connection pooling tests** | 3h | D.0 | ⬜ Not Started | | [Details](tasks/D.2-connection-pooling.md) |
| D.3 | **OAuth forwarding tests** | 3h | C.0 | ⬜ Not Started | | [Details](tasks/D.3-oauth-tests.md) |

**Phase D Total**: 13 hours

### Phase E: Integration & CI/CD (Week 3)
Integrate with Shadowcat and set up automated testing

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| E.0 | **Integrate with cargo test** | 3h | C.0-C.3 | ⬜ Not Started | | [Details](tasks/E.0-cargo-integration.md) |
| E.1 | **Create GitHub Actions workflow** | 2h | E.0 | ⬜ Not Started | | [Details](tasks/E.1-github-actions.md) |
| E.2 | **Performance benchmarks** | 3h | E.0 | ⬜ Not Started | | [Details](tasks/E.2-benchmarks.md) |
| E.3 | **Documentation and examples** | 2h | All | ⬜ Not Started | | [Details](tasks/E.3-documentation.md) |

**Phase E Total**: 10 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (2025-08-23 to 2025-08-30)
- [ ] A.0: Extract mcp-validator test cases
- [ ] A.1: Analyze MCP spec compliance points
- [ ] A.2: Design Rust compliance architecture
- [ ] A.3: Create proxy-specific test scenarios
- [ ] B.0: Create compliance test module
- [ ] B.1: Implement test runner

### Completed Tasks
- None yet

## Success Criteria

### Functional Requirements
- ✅ Test all MCP protocol versions (2024-11-05, 2025-03-26, 2025-06-18)
- ✅ Support both forward and reverse proxy modes
- ✅ Test against official MCP reference servers
- ✅ Generate compliance reports (JSON + Markdown)
- ✅ Integrate with `cargo test`

### Performance Requirements
- ✅ < 10 seconds for full compliance suite
- ✅ < 100MB memory usage during tests
- ✅ Support parallel test execution

### Quality Requirements
- ✅ 100% test coverage for compliance framework
- ✅ No clippy warnings
- ✅ Full rustdoc documentation
- ✅ Integration tests passing
- ✅ CI/CD pipeline configured

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| MCP spec ambiguity | HIGH | Test against official servers, study TypeScript SDK | Active |
| Test flakiness | MEDIUM | Use deterministic test fixtures, avoid timing dependencies | Planned |
| Version compatibility | MEDIUM | Modular protocol adapters, version-specific tests | Planned |
| Maintenance burden | LOW | Automated test generation from spec, clear documentation | Planned |

## Key Findings from mcp-validator Analysis

### What We Learned
1. **Critical Bugs Found**:
   - HTTP transport never calls `start()`, causing all tests to fail
   - Protocol format mismatch: expects `clientCapabilities` instead of spec-compliant `capabilities`
   - No SSE response handling for streamable HTTP servers
   - Reference server doesn't follow MCP specification

2. **Valuable Components**:
   - Test case catalog (54 tests found, not 36 as initially thought)
   - Protocol version differences knowledge (2024-11-05, 2025-03-26, 2025-06-18)
   - Test categorization (8 categories: base protocol, tools, async, specs, version-specific, HTTP, resources, stdio)
   - Report generation patterns (JSON and Markdown output formats)
   - 28 proxy-specific test gaps identified for additional coverage

3. **Integration Issues**:
   - Python-based tool doesn't integrate well with Rust project
   - External dependency with poor maintenance
   - Quality bar mismatch with Shadowcat standards

### Decision: Build Our Own
After extensive debugging, we determined that building a Rust-native compliance suite is the better investment because:
- Native integration with Shadowcat
- Same quality standards and tooling
- Proxy-specific testing capabilities
- Better long-term maintainability

## Specification Analysis Results (Task A.1)

### Compliance Requirements Found
1. **Total Requirements**: 233 across all specification documents
   - MUST requirements: 106 (mandatory for compliance)
   - SHOULD requirements: 91 (recommended for quality)
   - MAY requirements: 36 (optional enhancements)

2. **Key Compliance Areas**:
   - Lifecycle management (initialization, operation, shutdown)
   - Transport requirements (stdio, Streamable HTTP, security)
   - Message format (JSON-RPC 2.0 strict compliance)
   - Capability negotiation (version-specific formats)
   - Error handling (standard codes and formats)

3. **Version Differences Documented**:
   - 2025-03-26 (Baseline): Async tools, object capabilities, Streamable HTTP
   - 2025-06-18 (Current): No batching, structured output, elicitation
   - Future versions: Architecture designed for easy addition

4. **Proxy-Specific Requirements Identified**:
   - Session ID mapping and dual tracking
   - Connection pooling and failover
   - OAuth token handling without forwarding
   - SSE reconnection and buffering

## Test Categories to Implement

### From mcp-validator (Adapted)
1. **Initialization Tests**
   - Protocol version negotiation
   - Capability exchange
   - Server info validation

2. **Tools Tests**
   - Tool listing
   - Tool invocation
   - Parameter validation
   - Error handling

3. **Async Operations**
   - Async tool calls
   - Result polling
   - Cancellation

4. **Resource Tests**
   - Resource listing
   - Resource fetching
   - Subscription management

### Proxy-Specific (New)
1. **Session Management**
   - Dual session ID tracking
   - Session persistence
   - Session cleanup

2. **Transport Handling**
   - SSE reconnection
   - Connection pooling
   - Failover scenarios

3. **Security**
   - OAuth token handling
   - Header forwarding
   - Authentication flows

## Key Architecture Decisions

Based on analysis and requirements, we've made the following architecture decisions:

### 1. Library-First Design
- **Primary artifact**: Reusable Rust library (`compliance` crate)
- **Secondary artifact**: CLI binary (`mcpspec`) as thin wrapper
- **Public API**: Clean, documented, suitable for external use
- **Integration**: Can be embedded in any Rust project's test suite
- **Binary naming**: Following h2spec/h3spec pattern for protocol tools

### 2. Shared MCP Library
- **Extract from Shadowcat**: Single `mcp` crate containing protocol, client, and server
- **Location**: `crates/mcp/` in workspace
- **Dependencies**: Both shadowcat and compliance use shared MCP library
- **Benefits**: Code reuse, simpler maintenance, clean architecture
- **Testing approach**: Test our implementation vs reference implementations

### 3. Comprehensive Test Coverage
- **Spec-based tests**: ~200 tests covering 233 MCP requirements
- **Proxy-specific tests**: ~50 tests for proxy behaviors
- **Total tests**: ~250 comprehensive compliance tests
- **Coverage target**: 100% of MUST requirements, 80%+ of SHOULD

### 4. Version Management Strategy
- **Supported versions**: 2025-03-26, 2025-06-18, and draft (living spec)
- **Architecture**: Pluggable version modules with feature detection
- **Future-proof**: New versions added via configuration, minimal code
- **Test selection**: Tests auto-detect applicable versions
- **Draft support**: Early testing against in-progress specifications

### 5. Test Categories Revised
Based on spec analysis, not just mcp-validator:
- **Lifecycle**: 20-25 tests (vs 4 in validator)
- **Transport**: 35-40 tests (vs 1 in validator)
- **Security**: 15-20 tests (vs 0 in validator)
- **Proxy**: 50 tests (new category)
- **Total**: ~250 tests (vs 54 in validator)

### 6. Compliance Matrix Testing
With our shared MCP library, we test against:
- **Our Client vs rmcp Server**: Validate compatibility with official SDK
- **rmcp Client vs Our Server**: Ensure we accept official SDK clients
- **Our Client vs Reference JS Server**: Test against TypeScript reference
- **Reference JS Client vs Our Server**: Accept reference implementation
- **Our Client vs Our Server**: Internal consistency
- **Shadowcat vs All**: Proxy compliance in all combinations
- **Result**: Comprehensive 3x3 compatibility matrix

### 7. Implementation Philosophy
- **Fast & Compliant**: Performance and spec adherence over ergonomics
- **No macro magic**: Direct, explicit implementation (no #[tool_router])
- **Independent**: No dependency on rmcp or external MCP libraries
- **Low-level control**: Optimize for proxy use cases
- **Future enhancements**: Add ergonomics later if needed

## Session Planning Guidelines

### Next Session Prompt
See `next-session-prompt.md` for the current session setup.

### Optimal Session Structure
1. **Review** (5 min): Check this tracker and relevant task files
2. **Implementation** (2-3 hours): Complete the task deliverables
3. **Testing** (30 min): Run tests, fix issues
4. **Documentation** (15 min): Update tracker, create PR if needed
5. **Handoff** (10 min): Update next-session-prompt.md

## Critical Implementation Guidelines

### Integration Requirements
- Must work with existing Shadowcat transport layer
- Must support all transport types (stdio, HTTP, SSE)
- Must integrate with session manager
- Must respect interceptor chain

### Testing Strategy
- Unit tests for each compliance test
- Integration tests with real MCP servers
- Performance benchmarks
- CI/CD automation

## Related Documents

### Primary References
- [mcp-validator Issues Analysis](../../shadowcat/docs/mcp-validator-issues.md)
- [MCP Specification](https://modelcontextprotocol.io/specification)
- [Shadowcat Architecture](../../shadowcat/docs/architecture.md)

### Task Files
- [Analysis Tasks](tasks/)
- [Test Scripts](../../shadowcat/scripts/)

### External Resources
- [Official MCP Servers](~/src/modelcontextprotocol/servers/)
- [mcp-validator Source](../../tools/mcp-validator/)

## Next Actions

1. **Complete A.0**: Extract and document all test cases from mcp-validator
2. **Start A.1**: Analyze MCP specifications for compliance points
3. **Begin framework design**: Create architecture proposal

## Notes

- We have working test scripts that prove Shadowcat correctly forwards MCP messages
- The everything server from modelcontextprotocol is our reference implementation
- Focus on test quality over quantity initially
- Consider using property-based testing for protocol compliance

---

**Document Version**: 1.0  
**Created**: 2025-08-23  
**Last Modified**: 2025-08-23  
**Author**: Development Team

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-23 | 1.0 | Initial plan creation based on mcp-validator analysis | Team |