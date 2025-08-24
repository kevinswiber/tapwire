# MCP Compliance Checker - Library Architecture Design

## Overview

A **library-first** MCP compliance checker that can be used as:
1. **Rust library** - Integrate into any Rust project
2. **CLI tool** - Standalone command-line testing
3. **Test framework** - Embed in CI/CD pipelines
4. **Proxy validator** - Test proxy-specific behaviors

## Workspace Structure

```
shadowcat/                        # Workspace root
├── src/                         # Shadowcat lib/CLI
├── Cargo.toml                   # Workspace + shadowcat package
├── crates/
│   ├── mcp/                    # Shared MCP implementation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # Protocol types
│   │       ├── client.rs       # MCP client
│   │       └── server.rs       # MCP server
│   └── compliance/             # Compliance crate
│       ├── Cargo.toml          # Library + binary
│       ├── src/
│       │   ├── lib.rs          # Public library API
│       │   ├── main.rs         # CLI binary (mcpspec)
│   │   ├── framework/           # Core engine
│   │   │   ├── mod.rs
│   │   │   ├── runner.rs        # Test orchestration
│   │   │   ├── registry.rs      # Version registry
│   │   │   └── context.rs       # Test context
│   │   ├── tests/               # Test implementations
│   │   │   ├── mod.rs
│   │   │   ├── lifecycle.rs     # Lifecycle tests
│   │   │   ├── transport.rs     # Transport tests
│   │   │   ├── tools.rs         # Tools tests
│   │   │   ├── security.rs      # Security tests
│   │   │   └── proxy.rs         # Proxy-specific tests
│   │   ├── versions/            # Version adapters
│   │   │   ├── mod.rs
│   │   │   ├── v2025_03_26.rs
│   │   │   └── v2025_06_18.rs
│   │   ├── adapters/            # Protocol adapters
│   │   │   ├── mod.rs
│   │   │   ├── client.rs        # Test as MCP client
│   │   │   └── server.rs        # Test as MCP server
│   │   ├── validators/          # Message validators
│   │   │   ├── mod.rs
│   │   │   ├── jsonrpc.rs
│   │   │   └── mcp.rs
│   │   └── reports/             # Report generation
│   │       ├── mod.rs
│   │       ├── json.rs
│   │       ├── markdown.rs
│   │       └── junit.rs
│   ├── tests/                   # Integration tests
│   └── examples/                # Usage examples
└── xtask/                       # Build automation
```

## Public Library API

```rust
// crates/compliance/src/lib.rs

/// Main compliance checker - the primary public API
#[derive(Clone)]
pub struct ComplianceChecker {
    registry: Arc<VersionRegistry>,
    config: CheckerConfig,
}

impl ComplianceChecker {
    /// Create a new compliance checker with default config
    pub fn new() -> Self {
        Self::with_config(CheckerConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(config: CheckerConfig) -> Self {
        Self {
            registry: Arc::new(VersionRegistry::new()),
            config,
        }
    }
    
    /// Test an MCP server for compliance
    pub async fn test_server(
        &self,
        url: &str,
        version: &str,
        options: TestOptions,
    ) -> Result<ComplianceReport> {
        // Test server directly
    }
    
    /// Test an MCP proxy for compliance and transparency
    pub async fn test_proxy(
        &self,
        proxy_url: &str,
        upstream_url: &str,
        version: &str,
        options: ProxyTestOptions,
    ) -> Result<ProxyComplianceReport> {
        // Test proxy-specific behaviors
    }
    
    /// Validate a single MCP message
    pub fn validate_message(
        &self,
        message: &Value,
        version: &str,
        direction: MessageDirection,
    ) -> ValidationResult {
        // Validate message format and content
    }
    
    /// Run specific test suite
    pub async fn run_suite(
        &self,
        suite: TestSuite,
        target: TestTarget,
    ) -> Result<SuiteReport> {
        // Run a specific category of tests
    }
    
    /// List all available tests for a version
    pub fn list_tests(&self, version: &str) -> Vec<TestInfo> {
        // Return test metadata
    }
    
    /// Get supported versions
    pub fn supported_versions(&self) -> Vec<String> {
        self.registry.list_versions()
    }
}

/// Configuration for the compliance checker
#[derive(Clone, Debug)]
pub struct CheckerConfig {
    pub timeout: Duration,
    pub parallel_tests: bool,
    pub max_parallel: usize,
    pub retry_failed: bool,
    pub verbose: bool,
    pub strict_mode: bool, // Fail on SHOULD violations
}

/// Test execution options
#[derive(Clone, Debug, Default)]
pub struct TestOptions {
    pub categories: Vec<TestCategory>,
    pub skip_tests: Vec<String>,
    pub only_tests: Vec<String>,
    pub fail_fast: bool,
    pub capture_traffic: bool,
}

/// Proxy-specific test options
#[derive(Clone, Debug, Default)]
pub struct ProxyTestOptions {
    pub test_forwarding: bool,
    pub test_session_mapping: bool,
    pub test_error_propagation: bool,
    pub test_performance: bool,
    pub upstream_auth: Option<String>,
}

/// Test results
#[derive(Clone, Debug, Serialize)]
pub struct ComplianceReport {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub target: String,
    pub summary: TestSummary,
    pub categories: HashMap<TestCategory, CategoryResult>,
    pub tests: Vec<TestResult>,
    pub compliance_level: ComplianceLevel,
}

impl ComplianceReport {
    /// Export to various formats
    pub fn to_json(&self) -> String { /* ... */ }
    pub fn to_markdown(&self) -> String { /* ... */ }
    pub fn to_junit(&self) -> String { /* ... */ }
    pub fn to_html(&self) -> String { /* ... */ }
}
```

## Using the Shared MCP Crate

```rust
// crates/compliance/Cargo.toml
[dependencies]
mcp = { path = "../mcp" }
tokio = { workspace = true }
serde = { workspace = true }
anyhow = { workspace = true }

// Use shared MCP components
use mcp::{
    JsonRpcRequest, JsonRpcResponse, ProtocolVersion,
    McpClient, McpServer, Transport,
    ServerCapabilities, ClientCapabilities,
};
```

### Components from Shared MCP Crate

1. **Protocol Layer** (`mcp`)
   - Message types and serialization
   - Protocol version handling
   - JSON-RPC formatting
   - Capability structures

2. **Client Implementation** (`mcp::McpClient`)
   - Connect to any MCP server
   - Send requests and handle responses
   - Capability negotiation
   - Session management

3. **Server Implementation** (`mcp::McpServer`)
   - Handle incoming MCP connections
   - Route requests to handlers
   - Manage server capabilities
   - Session state tracking

4. **Transport Abstraction** (`mcp::Transport`)
   - Common transport interface
   - stdio, HTTP, SSE implementations
   - Pluggable transport support

## Proxy-Specific Test Scenarios

```rust
// src/tests/proxy.rs

/// Tests specific to MCP proxy implementations
pub mod proxy_tests {
    use super::*;
    
    /// Verify message forwarding integrity
    pub async fn test_message_forwarding_integrity(
        ctx: &TestContext,
    ) -> TestResult {
        // Send message through proxy
        // Verify exact forwarding (no modification)
        // Check headers preserved/filtered correctly
    }
    
    /// Test session ID mapping
    pub async fn test_dual_session_mapping(
        ctx: &TestContext,
    ) -> TestResult {
        // Create client session
        // Verify proxy creates upstream session
        // Test mapping consistency
        // Verify cleanup on disconnect
    }
    
    /// Test error propagation
    pub async fn test_error_propagation(
        ctx: &TestContext,
    ) -> TestResult {
        // Trigger upstream error
        // Verify proxy forwards error correctly
        // Test error transformation if any
    }
    
    /// Test authentication handling
    pub async fn test_auth_not_forwarded(
        ctx: &TestContext,
    ) -> TestResult {
        // Send request with auth token
        // Verify proxy doesn't forward client token
        // Verify proxy uses its own auth
    }
    
    /// Test connection pooling
    pub async fn test_connection_pooling(
        ctx: &TestContext,
    ) -> TestResult {
        // Multiple client connections
        // Verify connection reuse
        // Test pool limits
        // Verify cleanup
    }
    
    /// Test SSE reconnection
    pub async fn test_sse_reconnection(
        ctx: &TestContext,
    ) -> TestResult {
        // Simulate SSE disconnect
        // Verify proxy reconnects
        // Test message buffering
        // Verify no message loss
    }
    
    /// Test failover
    pub async fn test_upstream_failover(
        ctx: &TestContext,
    ) -> TestResult {
        // Primary upstream fails
        // Verify failover to backup
        // Test session preservation
        // Verify transparent to client
    }
    
    /// Test rate limiting
    pub async fn test_rate_limiting(
        ctx: &TestContext,
    ) -> TestResult {
        // Send burst of requests
        // Verify rate limiting applied
        // Test per-client limits
        // Verify error responses
    }
    
    /// Test circuit breaker
    pub async fn test_circuit_breaker(
        ctx: &TestContext,
    ) -> TestResult {
        // Trigger repeated failures
        // Verify circuit opens
        // Test half-open state
        // Verify recovery
    }
    
    /// Test request/response correlation
    pub async fn test_message_correlation(
        ctx: &TestContext,
    ) -> TestResult {
        // Send multiple concurrent requests
        // Verify correct response routing
        // Test with request IDs
        // Verify no response mixing
    }
}
```

## CLI Interface

```rust
// src/main.rs - Thin wrapper over library

use clap::{Parser, Subcommand};
use compliance::{ComplianceChecker, TestOptions};

#[derive(Parser)]
#[command(name = "mcpspec")]
#[command(about = "MCP Protocol Compliance Checker")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(long, global = true)]
    verbose: bool,
    
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Test an MCP server
    Server {
        /// Server URL or command
        #[arg(value_name = "URL_OR_CMD")]
        target: String,
        
        /// Protocol version
        #[arg(short, long, default_value = "2025-06-18")]
        version: String,
        
        /// Test categories
        #[arg(short, long)]
        categories: Vec<String>,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Test an MCP proxy
    Proxy {
        /// Proxy URL
        #[arg(value_name = "PROXY_URL")]
        proxy: String,
        
        /// Upstream server URL
        #[arg(value_name = "UPSTREAM_URL")]
        upstream: String,
        
        /// Protocol version
        #[arg(short, long, default_value = "2025-06-18")]
        version: String,
        
        /// Include proxy-specific tests
        #[arg(long)]
        proxy_tests: bool,
    },
    
    /// Validate MCP messages from file
    Validate {
        /// Input file (JSON/JSONL)
        #[arg(value_name = "FILE")]
        file: PathBuf,
        
        /// Protocol version
        #[arg(short, long)]
        version: String,
    },
    
    /// List available tests
    List {
        /// Protocol version
        #[arg(short, long)]
        version: Option<String>,
        
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let checker = ComplianceChecker::new();
    
    match cli.command {
        Commands::Server { target, version, .. } => {
            let report = checker.test_server(&target, &version, TestOptions::default()).await?;
            
            if cli.json {
                println!("{}", report.to_json());
            } else {
                println!("{}", report.to_markdown());
            }
        }
        Commands::Proxy { proxy, upstream, version, proxy_tests } => {
            let options = ProxyTestOptions {
                test_forwarding: proxy_tests,
                test_session_mapping: proxy_tests,
                test_error_propagation: proxy_tests,
                ..Default::default()
            };
            
            let report = checker.test_proxy(&proxy, &upstream, &version, options).await?;
            println!("{}", report.to_markdown());
        }
        // ... other commands
    }
    
    Ok(())
}
```

## Usage Examples

### As a Library

```rust
// In another Rust project
use compliance::{ComplianceChecker, TestOptions, TestCategory};

#[tokio::test]
async fn test_our_mcp_server() {
    let checker = ComplianceChecker::new();
    
    let report = checker.test_server(
        "http://localhost:3000/mcp",
        "2025-06-18",
        TestOptions {
            categories: vec![TestCategory::Core, TestCategory::Tools],
            fail_fast: true,
            ..Default::default()
        }
    ).await.unwrap();
    
    assert!(report.is_compliant());
    assert!(report.summary.must_pass_rate >= 100.0);
}

// Test a proxy
#[tokio::test]
async fn test_shadowcat_proxy() {
    let checker = ComplianceChecker::new();
    
    let report = checker.test_proxy(
        "http://localhost:8080",
        "http://upstream:3000",
        "2025-06-18",
        ProxyTestOptions {
            test_forwarding: true,
            test_session_mapping: true,
            ..Default::default()
        }
    ).await.unwrap();
    
    assert!(report.proxy_transparent);
}
```

### As a CLI

```bash
# Test a server
mcpspec server http://localhost:3000/mcp -v 2025-06-18

# Test a stdio server
mcpspec server "node my-server.js" -v 2025-03-26

# Test a proxy
mcpspec proxy http://localhost:8080 http://upstream:3000 --proxy-tests

# Validate captured messages
mcpspec validate captured-traffic.jsonl -v 2025-06-18

# List available tests
mcpspec list -v 2025-06-18 -c transport

# Generate reports
mcpspec server http://localhost:3000 --json -o report.json
mcpspec server http://localhost:3000 -o report.md
```

### In CI/CD

```yaml
# GitHub Actions example
- name: Run MCP Compliance Tests
  run: |
    cargo install compliance
    mcpspec server ${{ env.SERVER_URL }} \
      -v 2025-06-18 \
      --json \
      -o compliance-report.json
    
- name: Upload Compliance Report
  uses: actions/upload-artifact@v2
  with:
    name: compliance-report
    path: compliance-report.json
```

## Integration with Shadowcat Workspace

```toml
# shadowcat/Cargo.toml (workspace root)
[workspace]
members = [
    ".",  # shadowcat at root
    "crates/mcp",
    "crates/compliance",
    "xtask",
]

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
async-trait = "0.1"

# crates/compliance/Cargo.toml
[package]
name = "compliance"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "mcpspec"
path = "src/main.rs"

[dependencies]
mcp = { path = "../mcp" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
async-trait = { workspace = true }
clap = { version = "4.0", features = ["derive"] }
chrono = "0.4"

[dev-dependencies]
tempfile = "3.0"
```

## Key Design Decisions

### 1. Library-First
- Clean public API
- No CLI dependencies in lib
- Programmatic access to all features
- Reusable in any Rust project

### 2. Version Agnostic
- Easy to add new MCP versions
- Tests self-select based on version
- Forward compatibility built-in

### 3. Uses Shared MCP Crate
- Reuses MCP client/server implementations
- Shares protocol types
- Consistent behavior
- No code duplication with shadowcat

### 4. Proxy-Aware
- First-class proxy testing
- Tests transparency
- Validates proxy behaviors
- Not just server compliance

### 5. Multiple Output Formats
- JSON for machines
- Markdown for humans
- JUnit for CI/CD
- HTML for reports

## Benefits

1. **Reusable** - Any Rust project can use it
2. **Comprehensive** - Tests specs, not just function
3. **Maintainable** - Leverages existing shadowcat code
4. **Extensible** - Easy to add versions and tests
5. **Professional** - Production-ready library and CLI
6. **Proxy-Focused** - Unique proxy validation capabilities

---

*Design Date: 2025-08-23*
*Purpose: Library-first MCP compliance checker*
*Integration: Shadowcat workspace crate*