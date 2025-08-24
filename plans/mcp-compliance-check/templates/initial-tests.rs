// Ready-to-copy test file for crates/mcp/tests/integration_test.rs

use serde_json::json;

#[test]
fn test_constants_available() {
    // After extracting constants.rs
    use mcp::constants::*;
    
    assert_eq!(DEFAULT_PROTOCOL_VERSION, "2025-06-18");
    assert!(SUPPORTED_VERSIONS.contains(&"2025-06-18"));
}

#[test]
fn test_version_enum() {
    // After extracting version.rs
    use mcp::version::ProtocolVersion;
    
    let v1 = ProtocolVersion::V20250618;
    let v2 = ProtocolVersion::from_str("2025-06-18").unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn test_json_rpc_id_creation() {
    // After extracting types.rs
    use mcp::types::JsonRpcId;
    
    let id1 = JsonRpcId::Number(42);
    let id2 = JsonRpcId::String("test-id".to_string());
    let id3 = JsonRpcId::Null;
    
    // Test serialization
    assert_eq!(serde_json::to_string(&id1).unwrap(), "42");
    assert_eq!(serde_json::to_string(&id2).unwrap(), "\"test-id\"");
    assert_eq!(serde_json::to_string(&id3).unwrap(), "null");
}

#[test]
fn test_session_id() {
    // After extracting types.rs
    use mcp::types::SessionId;
    
    let session = SessionId::new();
    assert!(!session.to_string().is_empty());
    
    let session2 = SessionId::from("test-session");
    assert_eq!(session2.as_str(), "test-session");
}

#[test]
fn test_parse_real_initialize_request() {
    // After extracting enough to parse JSON
    let json_str = r#"{
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        },
        "id": 1
    }"#;
    
    let value: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["method"], "initialize");
    assert_eq!(value["id"], 1);
}

#[test]
fn test_message_envelope() {
    // After extracting messages.rs
    use mcp::messages::{MessageEnvelope, Direction};
    
    let envelope = MessageEnvelope {
        id: "msg-001".to_string(),
        direction: Direction::Request,
        content: json!({"test": "data"}),
        timestamp: 12345,
    };
    
    assert_eq!(envelope.id, "msg-001");
    assert!(matches!(envelope.direction, Direction::Request));
}

#[test]
fn test_fixtures_parse_as_json() {
    // This should work immediately, even before types are fully extracted
    let fixtures = [
        include_str!("../../../plans/mcp-compliance-check/fixtures/initialize_request.json"),
        include_str!("../../../plans/mcp-compliance-check/fixtures/initialize_response.json"),
        include_str!("../../../plans/mcp-compliance-check/fixtures/error_response.json"),
        include_str!("../../../plans/mcp-compliance-check/fixtures/tool_call_request.json"),
        include_str!("../../../plans/mcp-compliance-check/fixtures/batch_request.json"),
    ];
    
    for (i, fixture) in fixtures.iter().enumerate() {
        let value: serde_json::Value = serde_json::from_str(fixture)
            .unwrap_or_else(|e| panic!("Fixture {} failed to parse: {}", i, e));
        
        // Basic structure validation
        if value.is_object() {
            assert!(value.get("jsonrpc").is_some(), "Missing jsonrpc in fixture {}", i);
        }
    }
}

// Milestone 1: Core types work
#[test]
fn milestone_1_core_types() {
    use mcp::types::*;
    
    // Can create all basic types
    let _ = JsonRpcId::Number(1);
    let _ = SessionId::new();
    let _ = TransportType::Stdio;
    
    println!("✅ Milestone 1: Core types operational");
}

// Performance baseline test
#[test]
#[ignore] // Run with: cargo test -- --ignored
fn performance_baseline() {
    use std::time::Instant;
    
    let iterations = 10000;
    let start = Instant::now();
    
    for i in 0..iterations {
        let _id = mcp::types::JsonRpcId::Number(i);
        let _json = serde_json::to_string(&_id).unwrap();
    }
    
    let duration = start.elapsed();
    let per_op = duration / iterations;
    
    println!("JsonRpcId creation + serialization: {:?} per operation", per_op);
    assert!(per_op.as_micros() < 10, "Should be under 10μs per operation");
}