# MCP Compliance Check Tracker

## Overview

This tracker coordinates the development of a Rust-native MCP compliance testing framework for Shadowcat. After extensive analysis of the Python-based mcp-validator, we've determined that building our own compliance suite will provide better integration, quality control, and proxy-specific testing capabilities.

**Last Updated**: 2025-08-24 (Paused for Transport Architecture Review)  
**Total Estimated Duration**: 99 hours (16 + 15 + 11 + 9 + 14 + 12 + 10 + 12)  
**Status**: Phase B & C.0-C.1 Complete - PAUSED for architecture review  
**Strategy**: Copy-first extraction - Build clean MCP API, integrate shadowcat later  
**Work Location**: Git worktree at `/Users/kevin/src/tapwire/shadowcat-mcp-compliance` (branch: `feat/mcpspec`)

**CURRENT STATUS**: Pausing implementation to investigate Transport architecture design. Need to determine:
1. Should Transport be split into Incoming/Outgoing traits?
2. Should MCP Client manage subprocesses or accept AsyncRead/AsyncWrite?
3. What patterns does the official Rust SDK use?

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
│  Leverage Existing Code (~70% reusable):                 │
│  - shadowcat/src/mcp/* protocol implementation           │
│  - See shadowcat-mcp-extraction-inventory.md             │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

## Work Phases

### Phase A: Analysis & Knowledge Capture (Week 1) - ✅ COMPLETED
Capture learnings from mcp-validator investigation and design compliance framework

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | **Extract mcp-validator test cases** | 4h | None | ✅ Completed | | 54 tests cataloged |
| A.1 | **Analyze MCP spec compliance points** | 3h | None | ✅ Completed | | 233 requirements found |
| A.2 | **Design Rust compliance architecture** | 4h | A.0, A.1 | ✅ Completed | | See architectural-decisions.md |
| A.3 | **Create proxy-specific test scenarios** | 3h | A.1 | ✅ Completed | | 50 proxy tests identified |
| A.4 | **Inventory existing code for extraction** | 2h | A.2 | ✅ Completed | | ~70% code reusable |

**Phase A Total**: 16 hours (actual)

### Phase B: MCP Library Extraction (Week 1)
Extract and refactor MCP protocol implementation from shadowcat

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.0 | **Extract core types and messages** | 2h | None | ✅ Completed | | types.rs, messages.rs, constants.rs, version.rs |
| B.1 | **Extract builders and parsers** | 3h | B.0 | ✅ Completed | | builder.rs, parser.rs, validation.rs |
| B.2 | **Create Transport trait and stdio** | 4h | B.0 | ✅ Completed | | Transport trait + stdio::Transport (also added subprocess) |
| B.3 | **Build Client struct** | 3h | B.2 | ✅ Completed | | Client<T: Transport, H: Handler> with symmetric design |
| B.4 | **Build Server struct** | 3h | B.2 | ✅ Completed | | Server<T: Transport, H: Handler> with symmetric design |

**Phase B Total**: 15 hours

### Phase C: Additional MCP Components (Week 2)
Complete MCP library with advanced features

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.0 | **Add HTTP transport with SSE** | 4h | B.2 | ✅ Completed | | http::Transport + streaming::sse with reconnection |
| C.1 | **Extract interceptor system** | 3h | B.3, B.4 | ✅ Completed | | Interceptor trait + Chain from shadowcat |
| C.2 | **Add batch support** | 2h | B.1 | ⬜ Not Started | | Batch handling from batch.rs |
| C.3 | **Test MCP crate independently** | 2h | C.0-C.2 | ⬜ Not Started | | Integration tests, examples |

**Phase C Total**: 11 hours

### Phase C.5: Transport Architecture Investigation (INSERTED)
Re-evaluate transport design based on implementation discoveries

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.5.0 | **Investigate official Rust SDK patterns** | 2h | None | ⬜ Not Started | | Review ~/src/modelcontextprotocol/rust-sdk |
| C.5.1 | **Analyze Incoming vs Outgoing split** | 1h | C.5.0 | ⬜ Not Started | | Determine if Transport should be split |
| C.5.2 | **Subprocess management decision** | 1h | C.5.0 | ⬜ Not Started | | Client spawning vs AsyncRead/AsyncWrite |
| C.5.3 | **Document architectural decision** | 1h | C.5.1, C.5.2 | ⬜ Not Started | | Update architectural-decisions.md |
| C.5.4 | **Refactor transports if needed** | 3h | C.5.3 | ⬜ Not Started | | Implement new design |

**Phase C.5 Total**: 8 hours

### Phase D: Compliance Framework (Week 2)
Build the compliance testing framework using extracted MCP library

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.0 | **Create compliance crate structure** | 2h | B.3, B.4 | ⬜ Not Started | | crates/compliance with mcpspec bin |
| D.1 | **Implement test runner core** | 3h | D.0 | ⬜ Not Started | | Test orchestration + streaming events |
| D.2 | **Build version registry** | 2h | D.0 | ⬜ Not Started | | Version detection + feature flags |
| D.3 | **Create report generator** | 2h | D.1 | ⬜ Not Started | | JSON + Markdown reports |

**Phase D Total**: 9 hours

### Phase E: Protocol Compliance Tests (Week 3)
Implement the actual compliance test suites

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| E.0 | **Basic protocol tests** | 3h | D.1 | ⬜ Not Started | | Init, version negotiation, capabilities |
| E.1 | **Tools compliance tests** | 3h | E.0 | ⬜ Not Started | | Tool listing, invocation, params |
| E.2 | **Resource tests** | 3h | E.0 | ⬜ Not Started | | Resource listing, fetching, subscriptions |
| E.3 | **Transport tests** | 3h | E.0 | ⬜ Not Started | | stdio, HTTP, SSE behaviors |
| E.4 | **Error handling tests** | 2h | E.0 | ⬜ Not Started | | Error codes, formats, recovery |

**Phase E Total**: 14 hours

### Phase F: Proxy & Advanced Tests (Week 3)
Proxy-specific and advanced compliance scenarios

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| F.0 | **Session management tests** | 3h | E.0 | ⬜ Not Started | | Dual session tracking, persistence |
| F.1 | **SSE reconnection tests** | 3h | E.3 | ⬜ Not Started | | Reconnect, buffering, failover |
| F.2 | **Security tests** | 3h | E.0 | ⬜ Not Started | | OAuth, headers, token handling |
| F.3 | **Async operations tests** | 3h | E.1 | ⬜ Not Started | | Async tools, polling, cancellation |

**Phase F Total**: 12 hours

### Phase G: Integration & CI/CD (Week 4)
Production readiness and automation

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| G.0 | **Integrate with cargo test** | 2h | F.0-F.3 | ⬜ Not Started | | Add to workspace tests |
| G.1 | **Create GitHub Actions workflow** | 2h | G.0 | ⬜ Not Started | | CI/CD automation |
| G.2 | **Performance benchmarks** | 3h | G.0 | ⬜ Not Started | | Latency, throughput metrics |
| G.3 | **Documentation and examples** | 3h | All | ⬜ Not Started | | README, examples, API docs |

**Phase G Total**: 10 hours

### Phase H: Shadowcat Integration (Week 4-5)
Integrate MCP crate back into shadowcat (can be done after MVP)

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| H.0 | **Replace shadowcat MCP module** | 4h | C.3 | ⬜ Not Started | | Update imports, fix breaking changes |
| H.1 | **Update proxy logic** | 3h | H.0 | ⬜ Not Started | | Adapt to new API |
| H.2 | **Fix integration tests** | 3h | H.1 | ⬜ Not Started | | Update test expectations |
| H.3 | **Performance validation** | 2h | H.2 | ⬜ Not Started | | Ensure no regression |

**Phase H Total**: 12 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (2025-08-23 to 2025-08-30)
- [x] A.0: Extract mcp-validator test cases
- [x] A.1: Analyze MCP spec compliance points
- [x] A.2: Design Rust compliance architecture
- [x] A.3: Create proxy-specific test scenarios
- [x] A.4: Inventory existing code for extraction
- [x] B.0: Extract core types and messages
- [x] B.1: Extract builders and parsers
- [x] B.2: Create Transport trait and stdio
- [x] B.3: Build Client struct
- [x] B.4: Build Server struct
- [x] C.0: Add HTTP transport with SSE
- [x] C.1: Extract interceptor system

### Completed Tasks
- **Phase A**: All analysis and architecture complete (16 hours)
  - 54 validator tests cataloged
  - 233 spec requirements identified
  - Architecture designed with key decisions documented
  - 50 proxy-specific tests defined
  - ~70% existing code inventoried for reuse

- **Phase B**: Core MCP library extraction complete (15 hours)
  - Extracted types, messages, constants, version modules
  - Built MessageBuilder and Parser with validation
  - Created Transport trait with stdio and subprocess implementations
  - Implemented symmetric Client<T, H> and Server<T, H> architecture

- **Phase C (Partial)**: Advanced components (7 hours of 11)
  - HTTP transport with SSE and reconnection support
  - Full interceptor system with chain and metrics
  - **PAUSED**: Need to investigate transport architecture before continuing

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

### Extraction Strategy (Copy-First in Worktree)
- **Work in git worktree**: `/Users/kevin/src/tapwire/shadowcat-mcp-compliance`
- **Branch**: `feat/mcpspec` (separate from main)
- **Phase B-C**: Copy code from shadowcat to create standalone MCP crate
- **Focus on clean API**: Design without backward compatibility constraints
- **Keep main shadowcat unchanged**: Work only in worktree
- **Phase H (later)**: Integrate MCP crate back into shadowcat
- **Benefits**: Freedom to design, reduced risk, cleaner architecture

### Integration Requirements (for Phase H)
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

1. **Start Phase B**: Begin MCP library extraction (in worktree)
   - Navigate to: `/Users/kevin/src/tapwire/shadowcat-mcp-compliance`
   - B.0: Extract core types and messages (2h)
   - B.1: Extract builders and parsers (3h)
   - B.2: Create Transport trait and stdio implementation (4h)

2. **Key Focus Areas**:
   - Work in git worktree on `feat/mcpspec` branch
   - Use extraction inventories as guides
   - Keep main shadowcat unchanged
   - Test extraction early with simple examples

3. **Session Planning**:
   - Start by: `cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance`
   - Each task is sized for one Claude session
   - Clear inputs/outputs defined
   - Dependencies mapped for proper sequencing

## Notes

- We have working test scripts that prove Shadowcat correctly forwards MCP messages
- The everything server from modelcontextprotocol is our reference implementation
- Focus on test quality over quantity initially
- Consider using property-based testing for protocol compliance

---

**Document Version**: 2.0  
**Created**: 2025-08-23  
**Last Modified**: 2025-08-24  
**Author**: Development Team

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-23 | 1.0 | Initial plan creation based on mcp-validator analysis | Team |
| 2025-08-24 | 2.0 | Restructured phases for extraction strategy, completed Phase A | Team |