# Test Fixtures and Validation Milestones

## Test Fixtures Structure

### 1. Core Protocol Messages
```
crates/mcp/tests/fixtures/
├── core/
│   ├── initialize_request.json
│   ├── initialize_response.json
│   ├── initialized_notification.json
│   └── error_response.json
├── tools/
│   ├── list_tools_request.json
│   ├── list_tools_response.json
│   ├── call_tool_request.json
│   └── call_tool_response.json
├── resources/
│   ├── list_resources_request.json
│   ├── list_resources_response.json
│   ├── read_resource_request.json
│   └── read_resource_response.json
└── edge_cases/
    ├── batch_request.json
    ├── invalid_version.json
    ├── missing_id.json
    └── malformed_json.json
```

### 2. Example Fixture Content

**initialize_request.json**:
```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-06-18",
    "capabilities": {
      "roots": {
        "listChanged": true
      },
      "sampling": {}
    },
    "clientInfo": {
      "name": "test-client",
      "version": "1.0.0"
    }
  },
  "id": 1
}
```

**tool_call_with_streaming.json**:
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "long_running_analysis",
    "arguments": {
      "data": "..."
    }
  },
  "id": "call-123"
}
```

### 3. How to Use Fixtures

```rust
// crates/mcp/tests/protocol_tests.rs

#[cfg(test)]
mod tests {
    use mcp::types::*;
    use serde_json::from_str;

    // Helper to load fixtures
    fn load_fixture(path: &str) -> serde_json::Value {
        let content = include_str!(concat!("fixtures/", path));
        serde_json::from_str(content).expect("Valid JSON fixture")
    }

    #[test]
    fn test_parse_initialize_request() {
        let json = load_fixture("core/initialize_request.json");
        let request: JsonRpcRequest = serde_json::from_value(json).unwrap();
        
        assert_eq!(request.method, "initialize");
        assert_eq!(request.id, Some(JsonRpcId::Number(1)));
    }

    #[test]
    fn test_roundtrip_messages() {
        let json = load_fixture("tools/call_tool_request.json");
        let request: JsonRpcRequest = serde_json::from_value(json.clone()).unwrap();
        let serialized = serde_json::to_value(&request).unwrap();
        
        assert_eq!(json, serialized, "Roundtrip should preserve structure");
    }

    #[test]
    fn test_error_handling() {
        let json = load_fixture("edge_cases/invalid_version.json");
        let result = JsonRpcRequest::validate(&json);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }
}
```

## Validation Milestones

### Milestone 1: Core Types (End of B.0)
```rust
// crates/mcp/tests/milestone_1_core.rs

#[test]
fn milestone_1_can_create_all_types() {
    // ✓ Can create IDs
    let _ = JsonRpcId::Number(1);
    let _ = JsonRpcId::String("test".into());
    
    // ✓ Can create requests
    let req = JsonRpcRequest::new("test", json!({}));
    
    // ✓ Can create responses  
    let resp = JsonRpcResponse::success(JsonRpcId::Number(1), json!({}));
    
    // ✓ Can handle errors
    let err = JsonRpcResponse::error(JsonRpcId::Number(1), -32600, "Invalid Request");
    
    println!("✅ Milestone 1: Core types working");
}

#[test]
fn milestone_1_fixtures_parse() {
    let fixtures = [
        "core/initialize_request.json",
        "core/initialize_response.json",
        "tools/list_tools_request.json",
    ];
    
    for fixture in fixtures {
        let json = load_fixture(fixture);
        let _: JsonRpcRequest = serde_json::from_value(json)
            .expect(&format!("Should parse {}", fixture));
    }
    
    println!("✅ Milestone 1: All fixtures parse");
}
```

### Milestone 2: Builders & Parsers (End of B.1)
```rust
#[test]
fn milestone_2_builder_api() {
    // ✓ Request builder
    let req = RequestBuilder::new("initialize")
        .with_params(json!({
            "protocolVersion": "2025-06-18"
        }))
        .with_id(1)
        .build();
    
    // ✓ Response builder
    let resp = ResponseBuilder::success(1)
        .with_result(json!({"status": "ok"}))
        .build();
    
    // ✓ Parser works
    let parsed = McpParser::parse(req.to_string())?;
    
    println!("✅ Milestone 2: Builders and parsers working");
}
```

### Milestone 3: Transport & Client (End of B.3)
```rust
#[tokio::test]
async fn milestone_3_basic_client() {
    // ✓ Can create transport
    let transport = MockTransport::new();
    
    // ✓ Can create client
    let client = Client::new(transport);
    
    // ✓ Can send initialize
    let response = client.initialize(ClientInfo {
        name: "test".into(),
        version: "1.0".into(),
    }).await.unwrap();
    
    // ✓ Version negotiated
    assert!(response.protocol_version.is_some());
    
    println!("✅ Milestone 3: Client can handshake");
}
```

### Milestone 4: Server Working (End of B.4)
```rust
#[tokio::test]
async fn milestone_4_echo_server() {
    // ✓ Can create handler
    struct EchoHandler;
    impl McpHandler for EchoHandler {
        async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
            JsonRpcResponse::success(req.id.unwrap(), json!({"echo": req.params}))
        }
    }
    
    // ✓ Can create server
    let server = Server::new(EchoHandler);
    
    // ✓ Can handle request
    let req = JsonRpcRequest::new("test", json!({"data": "hello"}));
    let resp = server.handle(req).await;
    
    assert!(resp.result.is_some());
    
    println!("✅ Milestone 4: Server handles requests");
}
```

### Milestone 5: Full Integration (End of Phase C)
```rust
#[tokio::test]
async fn milestone_5_full_integration() {
    // ✓ Start server in background
    let server_handle = tokio::spawn(async {
        let server = Server::new(TestHandler);
        server.listen("127.0.0.1:0").await
    });
    
    // ✓ Connect client
    let client = Client::connect("http://127.0.0.1:PORT").await?;
    
    // ✓ Full handshake
    let init = client.initialize(ClientInfo::default()).await?;
    client.initialized().await?;
    
    // ✓ Call tool
    let result = client.call_tool("test_tool", json!({})).await?;
    
    // ✓ Stream response
    let mut stream = client.call_tool_streaming("long_tool", json!({})).await?;
    while let Some(chunk) = stream.next().await {
        println!("Chunk: {:?}", chunk);
    }
    
    println!("✅ Milestone 5: Full MCP protocol working");
}
```

## Performance Validation Baselines

### Create Baseline Benchmarks
```rust
// crates/mcp/benches/baseline.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_request(c: &mut Criterion) {
    let json = include_str!("../tests/fixtures/core/initialize_request.json");
    
    c.bench_function("parse_initialize_request", |b| {
        b.iter(|| {
            let _: JsonRpcRequest = serde_json::from_str(black_box(json)).unwrap();
        });
    });
}

fn bench_build_response(c: &mut Criterion) {
    c.bench_function("build_response", |b| {
        b.iter(|| {
            ResponseBuilder::success(black_box(1))
                .with_result(json!({"test": "data"}))
                .build()
        });
    });
}

// Target metrics (from shadowcat baseline):
// - Parse request: < 10 μs
// - Build response: < 5 μs  
// - Full roundtrip: < 100 μs
```

## How These Help Implementation

### 1. Fixtures Provide Confidence
```rust
// When extracting types, immediately verify:
#[test]
fn extracted_types_handle_real_messages() {
    // Load ALL fixtures
    for fixture in glob("tests/fixtures/**/*.json") {
        let json = load_fixture(fixture);
        
        // Should either parse as request or response
        let is_request = JsonRpcRequest::from_value(json.clone()).is_ok();
        let is_response = JsonRpcResponse::from_value(json.clone()).is_ok();
        
        assert!(is_request || is_response, 
                "Fixture {} should parse as request or response", fixture);
    }
}
```

### 2. Milestones Prevent Regression
```rust
// Run after EVERY extraction
cargo test --test milestones

// If a milestone breaks, STOP and fix before continuing
```

### 3. Progressive Validation
Each milestone builds on previous:
- M1: Types exist and parse JSON
- M2: Can build and parse programmatically  
- M3: Can communicate over transport
- M4: Server can handle requests
- M5: Full protocol works end-to-end

## What Makes This Approach Effective

### 1. Real-World Validation
- Fixtures from actual MCP servers (not synthetic)
- Edge cases from shadowcat's experience
- Version-specific differences captured

### 2. Fast Feedback Loop
```bash
# After each file extraction:
cargo test --test milestone_1  # < 1 second

# After each phase:
cargo test --test milestones   # < 5 seconds

# Before commit:
cargo test                      # Full suite
```

### 3. Clear Progress Tracking
```
Day 1: ✅ M1 (types)
Day 2: ✅ M2 (builders) 
Day 3: ✅ M3 (client)
Day 4: ✅ M4 (server)
Week 2: ✅ M5 (integration)
```

### 4. Debugging Aid
When something breaks:
1. Which milestone failed?
2. Which fixture caused it?
3. What's the minimal reproduction?

## Session Integration

### Start of Session
```bash
# Quick health check
cd crates/mcp
cargo test --test milestones 2>&1 | grep "✅"

# Shows:
# ✅ Milestone 1: Core types working
# ✅ Milestone 2: Builders and parsers working
# ⏳ Milestone 3: Not yet implemented
```

### During Extraction
```bash
# After extracting a file
cargo test --test milestone_X --nocapture

# See exactly what works/doesn't
```

### End of Session
```bash
# Generate progress report
cargo test --test milestones --nocapture > progress.txt
git add progress.txt
git commit -m "Progress: Milestone 2 complete"
```

## The Secret Sauce

### What Makes This Work
1. **Immediate validation** - Know within seconds if extraction worked
2. **Real messages** - Not guessing what MCP looks like
3. **Progressive difficulty** - Start simple, build complexity
4. **Clear checkpoints** - Know exactly where you are

### What I Need as Claude
1. **Load fixtures early** - Include in context at session start
2. **Run milestone tests** - After each extraction step
3. **Stop on failure** - Don't continue if milestone breaks
4. **Document issues** - Note what needed special handling

### Success Pattern
```
Extract → Test against fixtures → Verify milestone → Commit → Next file
```

This creates a ratchet effect - always moving forward, never breaking what worked before.

---

*Created: 2025-08-24*
*Purpose: Concrete validation strategy for MCP extraction*
*Key: Test fixtures + milestones = confidence*