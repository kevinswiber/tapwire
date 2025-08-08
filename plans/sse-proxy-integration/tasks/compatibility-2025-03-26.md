# MCP 2025-03-26 Compatibility Adjustments

## Overview
This document outlines the changes needed to make the SSE proxy integration compatible with both MCP 2025-03-26 and 2025-06-18.

## Key Differences

### 1. JSON-RPC Batching Support

**2025-03-26** allows:
```json
// Batch request
[
  {"jsonrpc": "2.0", "method": "tools/list", "id": 1},
  {"jsonrpc": "2.0", "method": "resources/list", "id": 2}
]

// Batch response
[
  {"jsonrpc": "2.0", "result": [...], "id": 1},
  {"jsonrpc": "2.0", "result": [...], "id": 2}
]
```

**2025-06-18** requires single messages only.

### 2. Default Protocol Version
- 2025-03-26: Default when no `MCP-Protocol-Version` header
- 2025-06-18: Must be explicitly specified

## Required Changes

### Task 1.2 Modification: SSE Transport Wrapper
**File**: `src/transport/sse_transport.rs`

Add batch support to message conversion:

```rust
impl SseTransport {
    /// Convert SSE event to TransportMessage(s)
    fn sse_event_to_transport_messages(
        event: &SseEvent,
        protocol_version: &str,
    ) -> Result<Vec<TransportMessage>, SseError> {
        let json: Value = serde_json::from_str(&event.data)?;
        
        // Check if it's a batch (2025-03-26)
        if protocol_version == "2025-03-26" {
            if let Value::Array(messages) = json {
                return messages.into_iter()
                    .map(|msg| Self::json_to_transport_message(msg))
                    .collect::<Option<Vec<_>>>()
                    .ok_or(SseError::InvalidFormat);
            }
        }
        
        // Single message
        Self::json_to_transport_message(json)
            .map(|msg| vec![msg])
            .ok_or(SseError::InvalidFormat)
    }
    
    /// Convert TransportMessage(s) to JSON for sending
    fn transport_messages_to_json(
        messages: Vec<TransportMessage>,
        protocol_version: &str,
    ) -> Value {
        let json_messages: Vec<Value> = messages.into_iter()
            .map(|msg| match msg {
                TransportMessage::Request { id, method, params } => {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "method": method,
                        "params": params
                    })
                }
                TransportMessage::Response { id, result, error } => {
                    let mut resp = serde_json::json!({"jsonrpc": "2.0", "id": id});
                    if let Some(result) = result {
                        resp["result"] = result;
                    }
                    if let Some(error) = error {
                        resp["error"] = error;
                    }
                    resp
                }
                TransportMessage::Notification { method, params } => {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": method,
                        "params": params
                    })
                }
            })
            .collect();
        
        // Return array for 2025-03-26 if multiple messages, otherwise single
        if protocol_version == "2025-03-26" && json_messages.len() > 1 {
            Value::Array(json_messages)
        } else if json_messages.len() == 1 {
            json_messages.into_iter().next().unwrap()
        } else {
            Value::Array(json_messages)
        }
    }
}
```

### Task 2.1 Modification: Dual-Method Endpoint
**File**: `src/proxy/reverse/mcp_endpoint.rs`

Update message type detection for batches:

```rust
fn determine_message_types(
    json: &Value,
    protocol_version: &str,
) -> Result<Vec<MessageType>, McpError> {
    // Check for batch (2025-03-26)
    if protocol_version == "2025-03-26" {
        if let Value::Array(messages) = json {
            return messages.iter()
                .map(|msg| determine_single_message_type(msg))
                .collect::<Result<Vec<_>, _>>();
        }
    }
    
    // Single message
    determine_single_message_type(json).map(|t| vec![t])
}

fn determine_single_message_type(json: &Value) -> Result<MessageType, McpError> {
    if !json.get("jsonrpc").map_or(false, |v| v == "2.0") {
        return Err(McpError::InvalidJsonRpc);
    }
    
    let has_method = json.get("method").is_some();
    let has_id = json.get("id").is_some();
    
    if has_method && has_id {
        Ok(MessageType::Request)
    } else if has_method {
        Ok(MessageType::Notification)
    } else if has_id {
        Ok(MessageType::Response)
    } else {
        Err(McpError::InvalidJsonRpc)
    }
}

async fn handle_post(
    headers: HeaderMap,
    state: McpEndpointState,
    body: Option<Json<Value>>,
) -> Result<Response, McpError> {
    let protocol_version = extract_protocol_version(&headers)?;
    let message_types = determine_message_types(&json_body.0, &protocol_version)?;
    
    // Check if batch contains any requests
    let has_requests = message_types.iter().any(|t| matches!(t, MessageType::Request));
    
    if has_requests {
        // Need to return responses for all requests
        let responses = process_batch_requests(&json_body.0, &state, &protocol_version).await?;
        
        // Determine response format
        if should_stream_response(&responses, accepts_sse) {
            create_sse_response(state, session_id, responses).await
        } else if accepts_json {
            // Return batched JSON response for 2025-03-26
            if protocol_version == "2025-03-26" && responses.is_array() {
                Ok(Json(responses).into_response())
            } else {
                Ok(Json(responses).into_response())
            }
        } else {
            Err(McpError::NotAcceptable)
        }
    } else {
        // Only notifications/responses - return 202
        forward_batch_to_upstream(&json_body.0, &state, &protocol_version).await?;
        Ok(StatusCode::ACCEPTED.into_response())
    }
}
```

### Task 2.2 Modification: SSE Response Handler
**File**: `src/proxy/reverse/sse_handler.rs`

Handle batch conversion in SSE events:

```rust
async fn convert_to_sse_event(
    msg: TransportMessage,
    event_gen: &EventIdGenerator,
    max_size: usize,
    protocol_version: &str,
) -> Result<Event, SseConversionError> {
    // For single message (same as before)
    let json = transport_message_to_json(msg);
    let data = serde_json::to_string(&json)?;
    
    if data.len() > max_size {
        return Err(SseConversionError::TooLarge(data.len(), max_size));
    }
    
    Ok(Event::default()
        .id(event_gen.generate_simple())
        .event("message")
        .data(data))
}

async fn convert_batch_to_sse_event(
    messages: Vec<TransportMessage>,
    event_gen: &EventIdGenerator,
    max_size: usize,
    protocol_version: &str,
) -> Result<Event, SseConversionError> {
    if protocol_version != "2025-03-26" {
        // 2025-06-18 doesn't support batching
        return Err(SseConversionError::BatchNotSupported);
    }
    
    let json_array: Vec<Value> = messages.into_iter()
        .map(transport_message_to_json)
        .collect();
    
    let data = serde_json::to_string(&Value::Array(json_array))?;
    
    if data.len() > max_size {
        return Err(SseConversionError::TooLarge(data.len(), max_size));
    }
    
    Ok(Event::default()
        .id(event_gen.generate_simple())
        .event("batch")
        .data(data))
}
```

## Configuration Updates

### CLI Configuration
**File**: `src/cli.rs`

```rust
#[derive(Debug, Clone, Subcommand)]
pub enum ForwardTransport {
    Sse {
        // ... existing fields ...
        
        /// MCP protocol version (2025-03-26 or 2025-06-18)
        #[arg(long, default_value = "2025-03-26")]
        protocol_version: String,
        
        /// Enable batch support (2025-03-26 only)
        #[arg(long, default_value = "true")]
        batch_support: bool,
    },
    StreamableHttp {
        // ... existing fields ...
        
        /// MCP protocol version
        #[arg(long, default_value = "2025-03-26")]
        protocol_version: String,
    },
}
```

### Version Negotiation
**File**: `src/protocol/versions.rs`

```rust
pub const V_2025_03_26: &str = "2025-03-26";
pub const V_2025_06_18: &str = "2025-06-18";

pub fn supports_batching(version: &str) -> bool {
    version == V_2025_03_26
}

pub fn default_version() -> &'static str {
    V_2025_03_26  // For backwards compatibility
}
```

## Testing Updates

### Batch Testing
```rust
#[tokio::test]
async fn test_batch_request_2025_03_26() {
    let batch = serde_json::json!([
        {"jsonrpc": "2.0", "method": "tools/list", "id": 1},
        {"jsonrpc": "2.0", "method": "resources/list", "id": 2}
    ]);
    
    let response = client
        .post("/mcp")
        .header("MCP-Protocol-Version", "2025-03-26")
        .json(&batch)
        .send()
        .await
        .unwrap();
    
    // Should return batch response
    let result: Value = response.json().await.unwrap();
    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_no_batch_2025_06_18() {
    let batch = serde_json::json!([
        {"jsonrpc": "2.0", "method": "tools/list", "id": 1}
    ]);
    
    let response = client
        .post("/mcp")
        .header("MCP-Protocol-Version", "2025-06-18")
        .json(&batch)
        .send()
        .await
        .unwrap();
    
    // Should reject batch for 2025-06-18
    assert_eq!(response.status(), 400);
}
```

## Backwards Compatibility Strategy

1. **Default to 2025-03-26** when no version header is present
2. **Support batch detection** - automatically handle arrays for 2025-03-26
3. **Version-specific behavior** - use protocol version to determine features
4. **Clear error messages** - inform clients when using incompatible features

## Summary

With these modifications, the SSE proxy integration will support both:
- **MCP 2025-03-26**: Full Streamable HTTP with JSON-RPC batching
- **MCP 2025-06-18**: Streamable HTTP without batching

The implementation gracefully handles version differences and provides clear feedback when version-specific features are used incorrectly.