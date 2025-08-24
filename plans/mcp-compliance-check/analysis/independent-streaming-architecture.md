# Independent Compliance Architecture with Streaming Results

## Critical Design Principle: Shared MCP Library, Independent Testing

The compliance checker uses a **shared MCP library** extracted from Shadowcat but remains independent for testing. It only interacts through standard MCP protocols, ensuring objective compliance testing.

## 1. Architecture with Shared MCP Crate

### Shared MCP Library

```rust
// crates/mcp/src/lib.rs - Shared MCP implementation

// Protocol types used by both shadowcat and compliance
#[derive(Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<JsonRpcId>,
    pub method: String,
    pub params: Option<Value>,
}

// MCP client implementation
pub struct McpClient {
    transport: Box<dyn Transport>,
}

// MCP server implementation  
pub struct McpServer {
    handler: Box<dyn Handler>,
}
```

### Compliance Tester Using Shared Library

```rust
// crates/compliance/src/lib.rs

// Import shared MCP types
use mcp::{JsonRpcRequest, McpClient, McpServer};

// Test through network interfaces only
pub struct ComplianceChecker {
    // Uses the shared MCP client to test servers
    client: McpClient,
}

impl ComplianceChecker {
    pub async fn test_server(&self, url: &str) -> Result<Report> {
        // Connect over HTTP/stdio like any external client would
        // Tests shadowcat through its public interface
        // No access to shadowcat internals
    }
}
```

### Testing Shadowcat Objectively

```rust
// We test Shadowcat the same way we'd test any other MCP implementation
#[test]
async fn test_shadowcat_compliance() {
    // Start Shadowcat as a separate process
    let shadowcat = Command::new("shadowcat")
        .args(&["forward", "http", "--port", "8080"])
        .spawn()?;
    
    // Test it through its public interface only
    let checker = ComplianceChecker::new();
    let report = checker.test_server("http://localhost:8080").await?;
    
    // We have no special access to Shadowcat internals
    assert!(report.is_compliant());
}
```

### Workspace Structure

The compliance checker lives in the shadowcat workspace but maintains independence:

```
shadowcat/               # Workspace root
├── src/                # Shadowcat lib/CLI
├── Cargo.toml         # Workspace + shadowcat package
├── crates/
│   ├── mcp/           # Shared MCP implementation
│   │   └── Cargo.toml # name = "mcp"
│   └── compliance/    # Compliance testing
│       └── Cargo.toml # name = "compliance"
└── xtask/             # Build automation
```

Dependencies are carefully managed:

```toml
# crates/compliance/Cargo.toml
[package]
name = "compliance"

[[bin]]
name = "mcpspec"  # Binary named like h2spec, h3spec

[dependencies]
# Shared MCP library
mcp = { path = "../mcp" }

# NO shadowcat dependency!
tokio = { workspace = true }
serde = { workspace = true }
reqwest = "0.11"
async-trait = "0.1"
```

## 2. Streaming Results Architecture

### JSON Lines (JSONL) Streaming Format

We use **JSON Lines** (newline-delimited JSON) for streaming results. This format:
- **Integrates easily** with CLI tools (grep, jq, awk)
- **Streams naturally** - each line is a complete JSON object
- **Adapts to transports** - easily converted to SSE or WebSockets
- **Parses incrementally** - no need to wait for complete output

```rust
use tokio::sync::mpsc;
use futures::Stream;
use serde_json;

/// Test events emitted in real-time as JSON Lines
#[derive(Clone, Debug, Serialize)]
pub enum TestEvent {
    TestStarted {
        name: String,
        category: String,
        total_in_category: usize,
    },
    TestProgress {
        name: String,
        step: String,
        percent: f32,
    },
    TestCompleted {
        name: String,
        result: TestResult,
        duration: Duration,
    },
    CategoryCompleted {
        category: String,
        passed: usize,
        failed: usize,
    },
    SuiteCompleted {
        total_passed: usize,
        total_failed: usize,
        duration: Duration,
    },
}

/// Compliance checker with streaming support
pub struct ComplianceChecker {
    event_tx: Option<mpsc::UnboundedSender<TestEvent>>,
}

impl ComplianceChecker {
    /// Create with event stream
    pub fn with_events() -> (Self, impl Stream<Item = TestEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let checker = Self {
            event_tx: Some(tx),
        };
        let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
        (checker, stream)
    }
    
    /// Run tests with streaming results
    pub async fn test_server_streaming(
        &self,
        url: &str,
        version: &str,
    ) -> Result<ComplianceReport> {
        let mut report = ComplianceReport::new();
        
        for category in self.get_test_categories() {
            self.emit(TestEvent::CategoryStarted {
                category: category.name.clone(),
                test_count: category.tests.len(),
            });
            
            for test in &category.tests {
                self.emit(TestEvent::TestStarted {
                    name: test.name.clone(),
                    category: category.name.clone(),
                });
                
                let start = Instant::now();
                let result = self.run_test(test, url).await;
                
                self.emit(TestEvent::TestCompleted {
                    name: test.name.clone(),
                    result: result.clone(),
                    duration: start.elapsed(),
                });
                
                report.add_result(result);
            }
            
            self.emit(TestEvent::CategoryCompleted {
                category: category.name.clone(),
                passed: category.passed_count(),
                failed: category.failed_count(),
            });
        }
        
        self.emit(TestEvent::SuiteCompleted {
            total_passed: report.passed,
            total_failed: report.failed,
            duration: report.duration,
        });
        
        Ok(report)
    }
    
    fn emit(&self, event: TestEvent) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event);
        }
    }
}
```

### JSON Lines Output for CLI Integration

```rust
// crates/compliance/src/bin/mcpspec.rs

impl ComplianceChecker {
    /// Write events as JSON Lines to stdout
    pub async fn write_jsonl_events<W: Write>(&self, mut writer: W) {
        while let Some(event) = self.events.recv().await {
            // Each event is a complete JSON object on its own line
            if let Ok(json) = serde_json::to_string(&event) {
                writeln!(writer, "{}", json).ok();
                writer.flush().ok();
            }
        }
    }
}
```

Usage with CLI tools:
```bash
# Stream to jq for filtering
mcpspec test http://localhost:8080 --format jsonl | jq 'select(.type == "TestFailed")'

# Count passed tests in real-time
mcpspec test http://localhost:8080 --format jsonl | grep '"type":"TestCompleted"' | wc -l

# Convert to SSE format
mcpspec test http://localhost:8080 --format jsonl | while read line; do
    echo "data: $line"
    echo
done
```

### CLI with Real-Time Output

```rust
// crates/compliance/src/bin/mcpspec.rs

use indicatif::{ProgressBar, ProgressStyle};
use colored::*;

async fn run_with_progress(url: &str) {
    let (checker, mut events) = ComplianceChecker::with_events();
    
    // Start test task
    let test_handle = tokio::spawn(async move {
        checker.test_server_streaming(url, "2025-06-18").await
    });
    
    // Process events in real-time
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("#>-")
    );
    
    while let Some(event) = events.recv().await {
        match event {
            TestEvent::TestStarted { name, .. } => {
                pb.set_message(format!("Testing: {}", name));
            }
            TestEvent::TestCompleted { name, result, duration } => {
                let status = match result {
                    TestResult::Pass => "✅ PASS".green(),
                    TestResult::Fail(_) => "❌ FAIL".red(),
                    TestResult::Skip(_) => "⏭️ SKIP".yellow(),
                };
                println!("{} {} ({:?})", status, name, duration);
                pb.inc(1);
            }
            TestEvent::CategoryCompleted { category, passed, failed } => {
                println!("\n{} - {} passed, {} failed", 
                    category.bold(), 
                    passed.to_string().green(),
                    failed.to_string().red()
                );
            }
            TestEvent::SuiteCompleted { total_passed, total_failed, duration } => {
                pb.finish_with_message("Complete!");
                println!("\n{}", "Final Results:".bold());
                println!("  Passed: {}", total_passed.to_string().green());
                println!("  Failed: {}", total_failed.to_string().red());
                println!("  Duration: {:?}", duration);
            }
        }
    }
    
    let report = test_handle.await??;
    report.write_to_file("compliance-report.json")?;
}
```

### Library Consumer with Streaming

```rust
// External crate using our library

use compliance::{ComplianceChecker, TestEvent};
use futures::StreamExt;

async fn monitor_compliance() {
    let (checker, mut events) = ComplianceChecker::with_events();
    
    // Spawn test task
    tokio::spawn(async move {
        checker.test_server_streaming("http://localhost:3000", "2025-06-18").await
    });
    
    // React to events in real-time
    while let Some(event) = events.next().await {
        match event {
            TestEvent::TestFailed { name, error } => {
                // Alert on failure
                send_alert(&format!("Test {} failed: {}", name, error));
            }
            TestEvent::TestCompleted { name, duration, .. } if duration > Duration::from_secs(5) => {
                // Log slow tests
                log::warn!("Test {} took {:?}", name, duration);
            }
            _ => {}
        }
    }
}
```

### Web Interface with SSE

```rust
// Streaming results to web UI via Server-Sent Events

use axum::response::sse::{Event, Sse};
use futures::stream::Stream;

async fn compliance_sse_endpoint(
    Query(params): Query<TestParams>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (checker, events) = ComplianceChecker::with_events();
    
    // Start tests
    tokio::spawn(async move {
        checker.test_server_streaming(&params.url, &params.version).await
    });
    
    // Convert to SSE events
    let sse_stream = events.map(|event| {
        Ok(Event::default()
            .event("test-event")
            .json_data(event)
            .unwrap())
    });
    
    Sse::new(sse_stream)
}
```

### Progress Reporting Options

```rust
pub enum ProgressStyle {
    /// No progress reporting
    Silent,
    
    /// Simple line-by-line output
    Simple,
    
    /// Progress bar with current test
    ProgressBar,
    
    /// Full streaming events
    Streaming,
    
    /// JSON events (for machines)
    JsonStream,
}

impl ComplianceChecker {
    pub fn set_progress_style(&mut self, style: ProgressStyle) {
        self.progress_style = style;
    }
}
```

## 3. Spec References for Analysis

The MCP specifications are used during development for analysis and test creation, not consumed by the Rust program:

```bash
# Correct paths for analysis during development
cd ~/src/modelcontextprotocol/modelcontextprotocol/docs/specification/

# Find requirements to create tests
grep -r "MUST\|SHOULD\|MAY" . --include="*.mdx"

# List available versions
ls -la
# Shows: 2025-03-26/ 2025-06-18/ draft/

# Analyze specific version
cd 2025-06-18/
grep -r "MUST" . --include="*.mdx" | wc -l  # Count mandatory requirements
```

## 4. Benefits of This Architecture

### Independence Benefits
1. **Objective testing** - No special access to Shadowcat internals
2. **Reusable** - Can test any MCP implementation
3. **Fair comparison** - All implementations tested equally
4. **Clean separation** - Compliance checker can evolve independently
5. **No coupling** - Changes to Shadowcat don't break tests

### Streaming Benefits
1. **Real-time feedback** - See results as they happen
2. **Early failure detection** - Stop on first critical failure
3. **Progress visibility** - Know how far along testing is
4. **Integration friendly** - Easy to embed in CI/CD dashboards
5. **Debugging aid** - See exactly where testing got stuck

## 5. Implementation Strategy

### Phase 1: Independent Core
```rust
// No shadowcat imports
// Own transport abstractions
// Test through public interfaces only
```

### Phase 2: Basic Testing
```rust
// Implement core compliance tests
// Simple pass/fail reporting
```

### Phase 3: Streaming Support
```rust
// Add event system
// Implement progress reporting
// CLI with real-time output
```

### Phase 4: Advanced Features
```rust
// Web UI with SSE
// Parallel test execution with streaming
// Detailed progress metrics
```

## Example Usage

### CLI with Streaming
```bash
# Real-time progress
mcpspec test http://localhost:8080 --progress

# Machine-readable JSON Lines stream
mcpspec test http://localhost:8080 --format jsonl | jq

# Filter for failures only
mcpspec test http://localhost:8080 --format jsonl | jq 'select(.result == "fail")'

# Web interface
mcpspec serve --port 9090
# Browse to http://localhost:9090 for real-time dashboard
```

### Library with Streaming
```rust
let (checker, events) = ComplianceChecker::with_events();

// Subscribe to events
tokio::spawn(async move {
    events.for_each(|event| {
        println!("Event: {:?}", event);
        future::ready(())
    }).await;
});

// Run tests (events stream automatically)
let report = checker.test_server("http://localhost:8080").await?;
```

## Conclusion

This architecture provides:
1. **Complete independence** from Shadowcat internals
2. **Objective compliance testing** through public interfaces
3. **Real-time streaming results** for better UX
4. **Correct spec references** for accuracy
5. **Maximum reusability** for any MCP implementation

The streaming capability is particularly valuable for:
- CI/CD integration (see failures immediately)
- Long test suites (know progress)
- Debugging (see where tests hang)
- User experience (responsive feedback)

---

*Created: 2025-08-24*
*Key Principle: Test objectively through public interfaces only*
*Key Feature: Real-time streaming results for better UX*