# Next Session Prompt - mcpspec Compliance Framework

**Last Updated**: 2025-08-26  
**Current Phase**: Phase D - Compliance Framework Structure  
**Status**: MCP Library COMPLETE! Ready to build mcpspec tool

## 🎉 Major Achievement: MCP Library Foundation Complete!

### What We Just Completed (2025-08-26)

✅ **Production Pool Integration** - Shadowcat's battle-tested pool
- Tagged pool architecture with protocol-aware partitioning
- McpConnectionKey for HTTP, WebSocket, stdio
- EventListener pattern (fixed 5-second shutdown delays)
- Fast path optimization (~200ns hot path)
- 16 pool tests passing, 2 benchmarks operational

✅ **MCP Library Foundation** - 100% Complete
- Connection trait with protocol awareness
- HTTP/1.1 and HTTP/2 support
- WebSocket bidirectional communication
- Stdio singleton management
- Production-ready pooling with per-protocol limits
- Comprehensive test suite

## 🚨 IMPORTANT: Working in Git Worktree

**Work Directory**: `/Users/kevin/src/tapwire/shadowcat-mcp-compliance`
- Git worktree on branch `feat/mcpspec`
- Main shadowcat remains untouched
- Commit to `feat/mcpspec` branch

## 🎯 Primary Goal: Build mcpspec Compliance Tool

Now that the MCP library foundation is complete, we can focus on our main deliverable: the mcpspec compliance testing framework.

## Phase D: Compliance Framework Structure (9 hours)

### D.0: Create mcpspec Crate (2 hours)

**Location**: `crates/mcpspec/`

**Structure**:
```
crates/mcpspec/
├── Cargo.toml           # CLI and framework dependencies
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Framework library
│   ├── cli/             # Command-line interface
│   │   ├── mod.rs
│   │   ├── commands.rs  # test, validate, report
│   │   └── args.rs      # Argument parsing
│   ├── framework/       # Test framework core
│   │   ├── mod.rs
│   │   ├── runner.rs    # Test runner
│   │   ├── suite.rs     # Test suite definition
│   │   └── context.rs   # Test context and fixtures
│   ├── tests/           # Test definitions
│   │   ├── mod.rs
│   │   ├── protocol/    # Protocol compliance tests
│   │   ├── proxy/       # Proxy-specific tests
│   │   └── versions/    # Version-specific tests
│   └── reports/         # Report generation
│       ├── mod.rs
│       ├── json.rs      # JSON output
│       └── markdown.rs  # Human-readable reports
```

**Commands**:
```bash
# Create mcpspec crate
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance/crates
cargo new --lib mcpspec

# Add dependencies
cd mcpspec
# Edit Cargo.toml with required dependencies
```

### D.1: Create Test Framework (3 hours)

**Test Suite Definition**:
```rust
// src/framework/suite.rs
pub struct TestSuite {
    name: String,
    version: McpVersion,
    tests: Vec<Test>,
}

pub struct Test {
    id: String,
    description: String,
    category: TestCategory,
    runner: Box<dyn TestRunner>,
}

pub enum TestCategory {
    Protocol,      // Core protocol compliance
    Transport,     // Transport-specific
    Proxy,        // Proxy behavior
    Performance,  // Performance requirements
}

#[async_trait]
pub trait TestRunner: Send + Sync {
    async fn run(&self, context: &TestContext) -> TestResult;
}
```

**Test Context**:
```rust
// src/framework/context.rs
pub struct TestContext {
    client: mcp::Client<Box<dyn mcp::Connection>>,
    server_url: Url,
    version: McpVersion,
    fixtures: HashMap<String, Value>,
}
```

### D.2: Implement Test Runner (2 hours)

**Runner Implementation**:
```rust
// src/framework/runner.rs
pub struct ComplianceRunner {
    suites: Vec<TestSuite>,
    config: RunnerConfig,
}

impl ComplianceRunner {
    pub async fn run_all(&self) -> ComplianceReport {
        // Run all test suites
        // Collect results
        // Generate report
    }
    
    pub async fn run_suite(&self, suite_name: &str) -> SuiteResult {
        // Run specific suite
    }
    
    pub async fn run_test(&self, test_id: &str) -> TestResult {
        // Run individual test
    }
}
```

### D.3: Create Report Generator (2 hours)

**Report Types**:
```rust
// src/reports/mod.rs
pub struct ComplianceReport {
    timestamp: DateTime<Utc>,
    version: McpVersion,
    results: Vec<SuiteResult>,
    summary: Summary,
}

pub struct Summary {
    total: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
    duration: Duration,
}

// JSON output for CI/CD
impl ComplianceReport {
    pub fn to_json(&self) -> Result<String> { ... }
    pub fn to_markdown(&self) -> Result<String> { ... }
}
```

## Phase D Success Criteria

- [ ] mcpspec crate created with proper structure
- [ ] CLI accepts test/validate/report commands
- [ ] Test framework can define and run tests
- [ ] Test runner executes tests with proper context
- [ ] Reports generated in JSON and Markdown
- [ ] Basic test example working end-to-end

## Commands to Get Started

```bash
# Navigate to crates directory
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance/crates

# Create mcpspec crate
cargo new --lib mcpspec
cd mcpspec

# Add to workspace
# Edit ../Cargo.toml to add mcpspec to workspace members

# Set up initial structure
mkdir -p src/{cli,framework,tests,reports}
touch src/{cli,framework,tests,reports}/mod.rs

# Start with CLI skeleton
# Edit src/main.rs for CLI entry point

# Run initial compilation
cargo check
```

## After Phase D: What's Next

### Phase E: Protocol Compliance Tests (14 hours)
- Port tests from mcp-validator
- Create comprehensive protocol test suite
- Version-specific test implementations

### Phase F: Proxy-Specific Tests (12 hours)
- Forward proxy validation
- Reverse proxy validation
- Session management tests
- Connection pooling validation

### Phase G: CI/CD Integration (10 hours)
- GitHub Actions workflow
- Test matrix for versions
- Badge generation
- Release automation

## Key Resources

**MCP Specifications**:
- `/Users/kevin/src/modelcontextprotocol/modelcontextprotocol/specs/`
- Version 2025-03-26 and 2025-06-18 schemas

**Reference Implementation**:
- `~/src/modelcontextprotocol/typescript-sdk/` - Official TypeScript SDK
- `~/src/modelcontextprotocol/servers/everything/` - Test server with all features

**Our MCP Library**:
- `crates/mcp/` - Complete MCP protocol implementation
- Use this as the foundation for compliance testing

## Architecture Decisions

**Why Separate mcpspec Crate?**
- Clean separation of concerns
- Can be published independently
- Reusable by other projects
- Clear CLI interface

**Test Organization**:
- Group by category (protocol, transport, proxy)
- Version-aware test selection
- Parallel test execution where possible
- Fixture sharing for efficiency

## Performance Targets

- Test suite completion: < 30 seconds
- Individual test timeout: 5 seconds
- Memory usage: < 100MB
- Support for 1000+ tests

---

*This session begins the mcpspec compliance framework - our primary deliverable. The MCP library foundation is complete, providing everything needed to build comprehensive compliance testing.*

*Duration estimate for Phase D: 9 hours*  
*Priority: CRITICAL - Main project deliverable*