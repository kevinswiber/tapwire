# Next Session: MCP Compliance Framework - Implementation Phase

## Session Goal
Begin implementing the shadowcat-compliance library based on the comprehensive architecture design completed in Phase A.

## Context
- **Tracker**: `plans/mcp-compliance-check/mcp-compliance-check-tracker.md`
- **Current Phase**: B - Core Framework Implementation (Ready to start)
- **Previous Work**: Completed comprehensive analysis, designed library-first architecture

## Critical Findings from Analysis
1. **mcp-validator only covers ~12% of requirements** - inadequate for compliance
2. **Need ~250 total tests** (200 spec-based + 50 proxy-specific)
3. **Focus on versions 2025-03-26 and 2025-06-18 only** (not 2024-11-05)
4. **Library-first design** for reusability across projects

## Key Architecture Decisions Made
1. **Extract MCP libraries**: Create mcp-core, mcp-client, mcp-server crates
2. **Shared protocol code**: Both shadowcat and compliance use same MCP libraries
3. **Compliance matrix**: Test our impl vs reference impl in all combinations
4. **Three-way separation**: Client, server, and proxy tests
5. **Streaming results**: Real-time test progress feedback
6. **Version management**: Pluggable adapters for 2025-03-26 and 2025-06-18

## Tasks for Next Session

### NEW Task: Extract MCP Libraries (4 hours)
**Priority**: Must do first - enables everything else

1. **Create mcp-core crate**
   - Protocol types (JsonRpcRequest, JsonRpcResponse)
   - Transport trait
   - Capability structures
   - Protocol versions

2. **Create mcp-client crate**
   - Generic MCP client implementation
   - Extract from shadowcat's upstream handling
   - Make reusable for any MCP server

3. **Create mcp-server crate**
   - Generic MCP server implementation
   - Extract from shadowcat's downstream handling
   - McpHandler trait for implementations

4. **Refactor shadowcat**
   - Use extracted libraries
   - Remove duplicate code
   - Focus on proxy-specific logic

### Task B.0: Create Compliance Module (2 hours)
**File**: `tasks/B.0-create-module.md`

1. Create shadowcat-compliance crate
2. Depend on mcp-core, mcp-client, mcp-server
3. Implement compliance matrix testing
4. Define test runner architecture

### Task B.1: Implement Test Runner (4 hours)
**File**: `tasks/B.1-test-runner.md`

1. Build test orchestration engine
2. Create version registry:
   - v2025_03_26 module
   - v2025_06_18 module
3. Implement test execution framework
4. Add parallel test support
5. Create test filtering by version/category

## Implementation Priorities
Based on coverage gaps:
1. **Security tests** (0% coverage) - CRITICAL
2. **Transport tests** (4% coverage) - HIGH  
3. **Proxy tests** (0% coverage) - HIGH
4. **Lifecycle tests** (29% coverage) - MEDIUM

## Key Design Documents
- **Library API**: `analysis/library-architecture-design.md`
- **Version system**: `analysis/version-agnostic-architecture.md`
- **Proxy tests**: `analysis/proxy-specific-test-scenarios.md`
- **Coverage gaps**: `analysis/test-requirement-coverage-matrix.md`

## New Workspace Structure

```
shadowcat/
├── mcp-core/                # Protocol types & traits
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── protocol.rs     # JsonRpc types
│       ├── capabilities.rs # Capability structures
│       └── transport.rs    # Transport trait
│
├── mcp-client/              # Generic MCP client
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── client.rs
│
├── mcp-server/              # Generic MCP server
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── server.rs
│       └── handler.rs      # McpHandler trait
│
├── shadowcat/               # Refactored to use MCP libraries
│   ├── Cargo.toml          # Depends on mcp-core, mcp-client, mcp-server
│   └── src/
│       └── proxy.rs        # Proxy-specific logic only
│
└── shadowcat-compliance/    # Compliance testing
    ├── Cargo.toml          # Depends on mcp-core, mcp-client, mcp-server
    └── src/
        ├── lib.rs
        ├── main.rs
        └── matrix.rs       # Compliance matrix testing
```

## Initial API to Implement

```rust
// src/lib.rs
pub struct ComplianceChecker {
    registry: Arc<VersionRegistry>,
    config: CheckerConfig,
}

impl ComplianceChecker {
    pub fn new() -> Self;
    pub async fn test_server(&self, url: &str, version: &str) -> Result<ComplianceReport>;
    pub fn validate_message(&self, msg: &Value, version: &str) -> ValidationResult;
}
```

## Success Criteria
- [ ] shadowcat-compliance crate created and compiles
- [ ] Basic ComplianceChecker API implemented
- [ ] Can import shadowcat modules successfully
- [ ] Version registry with 2025-03-26 and 2025-06-18
- [ ] One simple test runs successfully
- [ ] CLI binary with basic server test command

## Commands Reference

```bash
# Create new crate
cd /Users/kevin/src/tapwire/shadowcat
cargo new --lib shadowcat-compliance

# Update workspace
vim Cargo.toml  # Add to workspace members

# Build and test
cd shadowcat-compliance
cargo build
cargo test

# Run CLI
cargo run -- server http://localhost:3000 -v 2025-06-18
```

## Next Steps After This Session
- Task B.2: Build protocol adapters for client/server testing
- Task B.3: Create report generator (JSON, Markdown, JUnit)
- Phase C: Start implementing the 250 compliance tests

---

**Duration**: 7 hours
**Focus**: Core framework implementation
**Deliverables**: Working shadowcat-compliance crate with basic functionality

*Last Updated: 2025-08-23*
*Ready for: Implementation phase*