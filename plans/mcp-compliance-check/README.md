# MCP Compliance Framework Project

## Executive Summary

We're building a comprehensive MCP (Model Context Protocol) compliance testing framework for Shadowcat, our MCP proxy. After analyzing the existing Python-based mcp-validator and finding it covers only ~12% of spec requirements, we're creating a Rust-native solution that will:

1. **Extract MCP protocol into shared libraries** (mcp-core, mcp-client, mcp-server)
2. **Build a compliance testing framework** with ~250 tests covering all spec requirements
3. **Create a compatibility matrix** testing our implementation against reference implementations
4. **Support three MCP versions**: 2025-03-26, 2025-06-18, and draft (living spec)

**Estimated effort**: 108 hours total  
**Current status**: Phase B complete, Phase C.0-C.1 complete, Transport architecture finalized (v2). Ready for C.5.4 implementation.

## Quick Start for New Developers

### Understanding the Project

1. **Start here**: Read [mcp-compliance-check-tracker.md](mcp-compliance-check-tracker.md) for the full project overview
2. **Understand the problem**: Review [analysis/mcp-validator-findings.md](analysis/mcp-validator-findings.md) to see why we're building this
3. **Architecture**: Study [analysis/architectural-decisions.md](analysis/architectural-decisions.md) for key design decisions
4. **Transport Design**: See [analysis/transport-architecture-final-v2.md](analysis/transport-architecture-final-v2.md) for CURRENT Framed/Sink/Stream architecture
5. **Next steps**: Check Phase C.5.4 in tracker - implement Framed/Sink/Stream transports

### Key Decisions Made

1. **Build our own MCP implementation** - Not depend on external libraries for core functionality
2. **Framed/Sink/Stream architecture** - Message-level unification using standard async traits
3. **Test separation** - Client tests (60) + Server tests (60) + Proxy tests (50) + Protocol tests (80)
4. **Support draft spec** - Stay ahead by testing against in-progress specifications
5. **Performance over ergonomics** - Fast, compliant, low-level implementation first
6. **Streaming results** - Real-time test progress for better UX

## Project Structure

```
mcp-compliance-check/
├── README.md                    # You are here
├── mcp-compliance-check-tracker.md  # Main project tracker
├── next-session-prompt.md       # What to work on next
│
├── analysis/                    # Research and design documents
│   ├── README.md               # Analysis overview
│   ├── mcp-validator-findings.md  # Why mcp-validator is insufficient
│   ├── shadowcat-proxy-validation.md  # Proof Shadowcat works correctly
│   ├── test-requirement-coverage-matrix.md  # Gap analysis (12% coverage!)
│   ├── mcp-compliance-checklist.md  # 233 spec requirements
│   ├── validator-test-catalog.md  # 54 tests from mcp-validator
│   ├── protocol-version-matrix.md  # Version differences
│   ├── library-architecture-design.md  # Initial library design
│   ├── client-server-proxy-separation.md  # Three-way test separation
│   ├── compliance-independence-design.md  # Streaming + independence
│   ├── mcp-core-extraction-architecture.md  # Shared libraries design
│   ├── build-vs-buy-analysis.md  # Our MCP implementation approach
│   └── architectural-decisions.md  # Key architecture decisions
│
└── tasks/                       # Detailed task descriptions
    ├── A.0-extract-validator-tests.md  # ✅ Completed
    └── A.1-analyze-mcp-specs.md       # ✅ Completed
```

## The Problem We're Solving

### Why Not Use mcp-validator?

The Python-based mcp-validator has critical issues:
- **Only 12% coverage** of MCP specification requirements
- **Critical bugs** preventing it from working (HTTP transport, SSE handling, protocol mismatches)
- **Missing areas**: Security (0% coverage), Transport (4% coverage), Proxy scenarios (0% coverage)
- **Not designed for proxies** like Shadowcat

Details: [analysis/test-requirement-coverage-matrix.md](analysis/test-requirement-coverage-matrix.md)

### Why Build Our Own?

1. **Shadowcat is both client AND server** - needs comprehensive testing of both roles
2. **Proxy-specific behaviors** - 50+ scenarios not covered by spec
3. **Performance critical** - Need fast, low-level implementation
4. **Draft spec support** - Stay ahead with early testing

## Architecture Overview

### Workspace Structure (Final)

```
shadowcat/                   # Workspace root
├── src/                    # Shadowcat lib/CLI
├── Cargo.toml             # Workspace + shadowcat package
├── crates/
│   ├── mcp/              # Shared MCP implementation (NEW)
│   └── compliance/       # Compliance testing framework (NEW)
└── xtask/                # Build automation
```

### Three-Way Test Separation

Instead of mixed proxy tests, we separate concerns:

1. **Client Compliance** - Does our MCP client behave correctly?
2. **Server Compliance** - Does our MCP server behave correctly?
3. **Proxy Compliance** - Does Shadowcat correctly bridge client/server?

This provides precise diagnostics when tests fail.

Details: [analysis/client-server-proxy-separation.md](analysis/client-server-proxy-separation.md)

### Compliance Matrix

We test all combinations for maximum compatibility:

```
                    | Our Server | rmcp Server | Reference JS
--------------------|------------|-------------|---------------
Our Client          |     ✅     |     ✅      |      ✅
rmcp Client         |     ✅     |     ✅      |      ✅
Reference JS Client |     ✅     |     ✅      |      ✅
```

Details: [analysis/mcp-core-extraction-architecture.md](analysis/mcp-core-extraction-architecture.md)

## Implementation Strategy

### Phase A: Analysis ✅ COMPLETE
- Extracted 54 test cases from mcp-validator
- Identified 233 spec requirements
- Found only 12% coverage in existing validator
- Designed comprehensive architecture

### Phase B: Core MCP Extraction ✅ COMPLETE
- Extracted types, messages, constants, version modules
- Built MessageBuilder and Parser with validation
- Created Transport trait with stdio and subprocess
- Implemented symmetric Client<T,H> and Server<T,H>

### Phase C: Advanced Components (IN PROGRESS)
- ✅ C.0: HTTP transport with SSE and reconnection
- ✅ C.1: Full interceptor system with chain
- ✅ C.5.0-C.5.3: Transport architecture investigation
- 🔄 C.5.4: Refactor to StreamTransport<R,W> (NEXT)
- ⬜ C.2: Add batch support for v2025-03-26
- ⬜ C.3: Test MCP crate independently

### Phase D: Integration
- Compliance matrix testing
- CI/CD integration
- Performance benchmarks

Details: [mcp-compliance-check-tracker.md](mcp-compliance-check-tracker.md)

## Key Technical Decisions

### 1. Independent MCP Implementation
- **NO dependency on rmcp** (official Rust SDK)
- **Direct, explicit code** - no macro magic like `#[tool_router]`
- **Performance focused** - optimize for proxy use cases
- **Full control** - we own our core infrastructure

Details: [analysis/mcp-implementation-strategy.md](analysis/mcp-implementation-strategy.md)

### 2. Version Support
- **2025-03-26** - Current stable
- **2025-06-18** - Latest release
- **draft** - Living spec for early testing

Specs location: `~/src/modelcontextprotocol/modelcontextprotocol/docs/specification/`

### 3. Streaming Results
- Real-time test progress
- Multiple output formats (CLI, JSON stream, SSE)
- Better CI/CD integration

Details: [analysis/independent-streaming-architecture.md](analysis/independent-streaming-architecture.md)

## Getting Started (Next Session)

### Immediate Tasks

1. **Extract MCP library** (4 hours)
   - Create single `crates/mcp/` crate
   - Extract protocol, client, and server from Shadowcat
   - Make reusable and independent

2. **Create compliance framework** (3 hours)
   - Set up `crates/compliance/` crate
   - Binary named `mcpspec` (like h2spec, h3spec)
   - Implement basic test runner with JSON Lines streaming

See [next-session-prompt.md](next-session-prompt.md) for detailed instructions.

### Commands to Run

```bash
# Navigate to project
cd /Users/kevin/src/tapwire/shadowcat

# Create crates directory and new crates
mkdir -p crates
cargo new --lib crates/mcp
cargo new --lib crates/compliance

# Update workspace
vim Cargo.toml  # Add crates/mcp and crates/compliance to workspace.members
```

## Success Criteria

Our MCP implementation and compliance framework should be:

1. **Faster** than rmcp for proxy use cases
2. **100% compliant** with all spec versions including draft
3. **Compatible** with rmcp and reference implementations
4. **Independent** - no external MCP dependencies
5. **Comprehensive** - 250+ tests covering all requirements

## Resources

### MCP Specifications
- Location: `~/src/modelcontextprotocol/modelcontextprotocol/docs/specification/`
- Versions: 2025-03-26, 2025-06-18, draft

### Reference Implementations
- TypeScript SDK: `~/src/modelcontextprotocol/typescript-sdk/`
- Rust SDK (rmcp): `~/src/modelcontextprotocol/rust-sdk/`
- Example servers: `~/src/modelcontextprotocol/servers/`

### Existing Code
- Shadowcat: `/Users/kevin/src/tapwire/shadowcat/`
- mcp-validator: `/Users/kevin/src/tapwire/tools/mcp-validator/`

## Questions This Plan Answers

1. **Why not use mcp-validator?** Only 12% coverage, critical bugs - [analysis/mcp-validator-findings.md](analysis/mcp-validator-findings.md)
2. **Why not depend on rmcp?** Need control, performance, proxy optimization - [analysis/mcp-implementation-strategy.md](analysis/mcp-implementation-strategy.md)
3. **How many tests needed?** ~250 total - [analysis/test-requirement-coverage-matrix.md](analysis/test-requirement-coverage-matrix.md)
4. **What's the architecture?** Shared MCP libs + compliance framework - [analysis/final-architecture-summary.md](analysis/final-architecture-summary.md)
5. **What's next?** Extract MCP libraries, build framework - [next-session-prompt.md](next-session-prompt.md)

## Contact & Status

- **Created**: 2025-08-23
- **Last Updated**: 2025-08-24
- **Status**: Planning complete, ready for Phase B implementation
- **Estimated Remaining Work**: 80-100 hours

---

*This README is the entry point for understanding the MCP Compliance Framework project. Start here, then dive into specific documents as needed.*