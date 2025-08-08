# Task 2.1: Dual-Method `/mcp` Endpoint

## Overview
Implement a unified `/mcp` endpoint in the reverse proxy that supports both POST and GET methods according to the MCP Streamable HTTP specification.

**Duration**: 3 hours  
**Priority**: HIGH  
**Prerequisites**: Reverse proxy HTTP server infrastructure  
**Compatibility Note**: See [MCP 2025-03-26 Compatibility Guide](../compatibility-2025-03-26.md) for batch request handling

## Current State

Existing reverse proxy:
- HTTP server using Axum
- Basic routing for HTTP endpoints
- Session management infrastructure
- Auth gateway integration

Missing:
- Unified `/mcp` endpoint
- Method-based routing (POST vs GET)
- Accept header validation
- Session ID requirement checking
- Proper error responses

## Requirements

### Functional Requirements (MCP 2025-06-18)
1. Single `/mcp` endpoint supporting POST and GET
2. POST: Accept JSON-RPC messages, return JSON or SSE
3. GET: Open SSE stream for server-initiated events
4. Validate Accept headers for content negotiation
5. Check Mcp-Session-Id header requirements
6. Return 405 Method Not Allowed for unsupported methods

### Non-Functional Requirements
- Thread-safe session handling
- Proper CORS headers for browser clients
- Security: Origin validation, DNS rebinding protection
- Clear error messages in responses

## Implementation Plan

### Step 1: Define Endpoint Handler Structure
**File**: `src/proxy/reverse/mcp_endpoint.rs` (new)

```rust
use axum::{
    extract::{State, Query},
    http::{Method, StatusCode, HeaderMap, HeaderValue},
    response::{Response, IntoResponse, Sse},
    Json,
};
use crate::session::{SessionStore, SessionId};
use crate::transport::sse::SseManager;
use serde_json::Value;
use futures::stream::Stream;

/// Handler for the unified /mcp endpoint
pub async fn mcp_handler(
    method: Method,
    headers: HeaderMap,
    State(state): State<McpEndpointState>,
    body: Option<Json<Value>>,
) -> Result<Response, McpError> {
    // Log request
    debug!("MCP endpoint: {} request", method);
    
    // Validate origin for security
    validate_origin(&headers)?;
    
    // Route based on method
    match method {
        Method::POST => handle_post(headers, state, body).await,
        Method::GET => handle_get(headers, state).await,
        Method::DELETE => handle_delete(headers, state).await,
        _ => {
            // Return 405 Method Not Allowed
            Err(McpError::MethodNotAllowed(method.to_string()))
        }
    }
}

#[derive(Clone)]
pub struct McpEndpointState {
    pub session_store: Arc<SessionStore>,
    pub sse_manager: Arc<SseManager>,
    pub upstream_client: Arc<HttpClient>,
    pub config: McpConfig,
}

#[derive(Debug, Clone)]
pub struct McpConfig {
    pub require_session_id: bool,
    pub allowed_origins: Vec<String>,
    pub max_sse_connections: usize,
    pub session_timeout: Duration,
    pub protocol_versions: Vec<String>,
}
```

### Step 2: POST Handler Implementation
**File**: `src/proxy/reverse/mcp_endpoint.rs`

```rust
async fn handle_post(
    headers: HeaderMap,
    state: McpEndpointState,
    body: Option<Json<Value>>,
) -> Result<Response, McpError> {
    // Extract headers
    let accept = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    let session_id = extract_session_id(&headers)?;
    let protocol_version = extract_protocol_version(&headers)?;
    
    // Validate protocol version
    if !state.config.protocol_versions.contains(&protocol_version) {
        return Err(McpError::UnsupportedProtocol(protocol_version));
    }
    
    // Check session requirement
    if state.config.require_session_id && session_id.is_none() {
        return Err(McpError::SessionRequired);
    }
    
    // Parse body
    let json_body = body.ok_or(McpError::InvalidBody("Missing body".into()))?;
    
    // Determine if this is a request, notification, or response
    let message_type = determine_message_type(&json_body.0)?;
    
    match message_type {
        MessageType::Request => {
            // Forward to upstream and determine response type
            let upstream_response = forward_to_upstream(
                &state.upstream_client,
                json_body.0.clone(),
                session_id.clone(),
                protocol_version.clone(),
            ).await?;
            
            // Check if client accepts SSE
            let accepts_sse = accept.contains("text/event-stream");
            let accepts_json = accept.contains("application/json");
            
            // Determine response format based on upstream and client preferences
            if should_stream_response(&upstream_response, accepts_sse) {
                // Return SSE stream
                create_sse_response(state, session_id, upstream_response).await
            } else if accepts_json {
                // Return JSON response
                Ok(Json(upstream_response).into_response())
            } else {
                Err(McpError::NotAcceptable)
            }
        }
        MessageType::Notification | MessageType::Response => {
            // Forward to upstream
            forward_to_upstream(
                &state.upstream_client,
                json_body.0,
                session_id,
                protocol_version,
            ).await?;
            
            // Return 202 Accepted
            Ok(StatusCode::ACCEPTED.into_response())
        }
    }
}

fn determine_message_type(json: &Value) -> Result<MessageType, McpError> {
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

enum MessageType {
    Request,
    Notification,
    Response,
}
```

### Step 3: GET Handler Implementation
**File**: `src/proxy/reverse/mcp_endpoint.rs`

```rust
async fn handle_get(
    headers: HeaderMap,
    state: McpEndpointState,
) -> Result<Response, McpError> {
    // Check Accept header
    let accept = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !accept.contains("text/event-stream") {
        return Err(McpError::NotAcceptable);
    }
    
    // Extract session ID (required for GET)
    let session_id = extract_session_id(&headers)?
        .ok_or(McpError::SessionRequired)?;
    
    // Check for Last-Event-ID for resumption
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    // Verify session exists
    if !state.session_store.exists(&session_id).await {
        return Err(McpError::SessionNotFound(session_id));
    }
    
    // Create SSE stream for server-initiated events
    let stream = create_server_event_stream(
        state,
        session_id,
        last_event_id,
    ).await?;
    
    // Return SSE response
    Ok(Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
        .into_response())
}

async fn create_server_event_stream(
    state: McpEndpointState,
    session_id: String,
    last_event_id: Option<String>,
) -> Result<impl Stream<Item = Result<SseEvent, axum::Error>>, McpError> {
    // Get or create event queue for session
    let event_queue = state.sse_manager
        .get_or_create_queue(&session_id)
        .await?;
    
    // If resuming, replay missed events
    if let Some(last_id) = last_event_id {
        event_queue.resume_from(last_id).await;
    }
    
    // Create stream from queue
    let stream = async_stream::stream! {
        let mut receiver = event_queue.subscribe();
        
        while let Ok(event) = receiver.recv().await {
            // Convert internal event to SSE event
            let sse_event = axum::response::sse::Event::default()
                .id(event.id.clone())
                .event(event.event_type.clone())
                .data(event.data);
            
            yield Ok(sse_event);
        }
    };
    
    Ok(stream)
}
```

### Step 4: DELETE Handler (Session Termination)
**File**: `src/proxy/reverse/mcp_endpoint.rs`

```rust
async fn handle_delete(
    headers: HeaderMap,
    state: McpEndpointState,
) -> Result<Response, McpError> {
    // Extract session ID
    let session_id = extract_session_id(&headers)?
        .ok_or(McpError::SessionRequired)?;
    
    // Check if server allows session termination
    if !state.config.allow_client_termination {
        return Ok(StatusCode::METHOD_NOT_ALLOWED.into_response());
    }
    
    // Terminate session
    if state.session_store.terminate(&session_id).await? {
        info!("Session {} terminated by client", session_id);
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(McpError::SessionNotFound(session_id))
    }
}
```

### Step 5: Helper Functions
**File**: `src/proxy/reverse/mcp_endpoint.rs`

```rust
fn extract_session_id(headers: &HeaderMap) -> Result<Option<String>, McpError> {
    headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| {
            // Validate session ID format
            if s.chars().all(|c| c.is_ascii_graphic()) {
                Ok(s.to_string())
            } else {
                Err(McpError::InvalidSessionId(s.to_string()))
            }
        })
        .transpose()
}

fn extract_protocol_version(headers: &HeaderMap) -> Result<String, McpError> {
    headers
        .get("mcp-protocol-version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "2025-03-26".to_string()) // Default for backwards compatibility
        .into()
}

fn validate_origin(headers: &HeaderMap) -> Result<(), McpError> {
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        // Check for DNS rebinding attacks
        if origin.starts_with("http://") && !origin.contains("localhost") && !origin.contains("127.0.0.1") {
            return Err(McpError::InvalidOrigin(origin.to_string()));
        }
    }
    Ok(())
}

fn should_stream_response(response: &Value, accepts_sse: bool) -> bool {
    // Stream if:
    // 1. Client accepts SSE
    // 2. Response indicates streaming is beneficial (e.g., large result set)
    // 3. Response contains subscription/streaming indicators
    
    accepts_sse && (
        response.get("stream").is_some() ||
        response.get("subscription").is_some() ||
        estimate_response_size(response) > 10_000
    )
}

fn estimate_response_size(value: &Value) -> usize {
    // Rough estimate of JSON size
    serde_json::to_string(value).map(|s| s.len()).unwrap_or(0)
}
```

### Step 6: Error Types
**File**: `src/proxy/reverse/mcp_error.rs` (new)

```rust
use axum::response::{Response, IntoResponse};
use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Method {0} not allowed")]
    MethodNotAllowed(String),
    
    #[error("Session ID required")]
    SessionRequired,
    
    #[error("Session {0} not found")]
    SessionNotFound(String),
    
    #[error("Invalid session ID: {0}")]
    InvalidSessionId(String),
    
    #[error("Unsupported protocol version: {0}")]
    UnsupportedProtocol(String),
    
    #[error("Invalid JSON-RPC message")]
    InvalidJsonRpc,
    
    #[error("Invalid request body: {0}")]
    InvalidBody(String),
    
    #[error("Not acceptable - client must accept application/json or text/event-stream")]
    NotAcceptable,
    
    #[error("Invalid origin: {0}")]
    InvalidOrigin(String),
    
    #[error("Upstream error: {0}")]
    UpstreamError(String),
}

impl IntoResponse for McpError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            McpError::MethodNotAllowed(_) => (StatusCode::METHOD_NOT_ALLOWED, self.to_string()),
            McpError::SessionRequired | McpError::InvalidSessionId(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            McpError::SessionNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            McpError::UnsupportedProtocol(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            McpError::InvalidJsonRpc | McpError::InvalidBody(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            McpError::NotAcceptable => (StatusCode::NOT_ACCEPTABLE, self.to_string()),
            McpError::InvalidOrigin(_) => (StatusCode::FORBIDDEN, self.to_string()),
            McpError::UpstreamError(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
        };
        
        // Return JSON-RPC error response with no ID
        let error_response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32600,
                "message": message
            }
        });
        
        (status, Json(error_response)).into_response()
    }
}
```

### Step 7: Router Integration
**File**: `src/proxy/reverse.rs` (modified)

```rust
use crate::proxy::reverse::mcp_endpoint::{mcp_handler, McpEndpointState, McpConfig};

pub fn create_reverse_proxy_router(config: ReverseProxyConfig) -> Router {
    let mcp_state = McpEndpointState {
        session_store: Arc::new(SessionStore::new()),
        sse_manager: Arc::new(SseManager::new()),
        upstream_client: Arc::new(HttpClient::new()),
        config: McpConfig {
            require_session_id: true,
            allowed_origins: vec!["http://localhost".into(), "https://localhost".into()],
            max_sse_connections: 1000,
            session_timeout: Duration::from_secs(3600),
            protocol_versions: vec!["2025-06-18".into(), "2025-03-26".into()],
        },
    };
    
    Router::new()
        .route("/mcp", any(mcp_handler))
        .with_state(mcp_state)
        // Other routes...
}
```

## Testing Plan

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, Method};
    
    #[test]
    fn test_extract_session_id() {
        let mut headers = HeaderMap::new();
        headers.insert("mcp-session-id", "test-123".parse().unwrap());
        
        let result = extract_session_id(&headers).unwrap();
        assert_eq!(result, Some("test-123".to_string()));
    }
    
    #[test]
    fn test_determine_message_type() {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "test",
            "id": 1
        });
        assert!(matches!(
            determine_message_type(&request).unwrap(),
            MessageType::Request
        ));
        
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "test"
        });
        assert!(matches!(
            determine_message_type(&notification).unwrap(),
            MessageType::Notification
        ));
    }
    
    #[test]
    fn test_validate_origin() {
        let mut headers = HeaderMap::new();
        
        // Valid localhost
        headers.insert("origin", "http://localhost:3000".parse().unwrap());
        assert!(validate_origin(&headers).is_ok());
        
        // Invalid external HTTP
        headers.insert("origin", "http://evil.com".parse().unwrap());
        assert!(validate_origin(&headers).is_err());
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_post_json_response() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/mcp")
                .header("accept", "application/json")
                .header("mcp-protocol-version", "2025-06-18")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"jsonrpc":"2.0","method":"test","id":1}"#))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_sse_stream() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/mcp")
                .header("accept", "text/event-stream")
                .header("mcp-session-id", "test-session")
                .build()
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/event-stream"
    );
}

#[tokio::test]
async fn test_unsupported_method() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/mcp")
                .build()
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}
```

## Success Criteria

- [ ] `/mcp` endpoint handles POST and GET methods
- [ ] Content negotiation working (JSON vs SSE)
- [ ] Session ID validation implemented
- [ ] Origin validation prevents DNS rebinding
- [ ] 405 returned for unsupported methods
- [ ] All tests passing
- [ ] Performance: < 5ms overhead for routing

## Dependencies

- Axum HTTP server framework
- Session management system
- SSE manager implementation
- Upstream HTTP client

## Notes

- Consider rate limiting per session
- May need WebSocket support in future
- CORS headers should be configurable
- Monitor for connection limit exhaustion
- Coordinate with Task 2.2 for SSE response handling