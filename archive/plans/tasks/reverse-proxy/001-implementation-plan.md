# Task 001: Detailed Implementation Plan - Axum HTTP Server Setup & MCP Transport

**Phase:** 5 (Reverse Proxy & Authentication)  
**Created:** August 5, 2025  
**Status:** Ready for Implementation  
**Estimated Time:** 6-8 hours

## Executive Summary

This document provides a detailed, step-by-step implementation plan for Task 001, building the foundation HTTP server using Axum with full MCP 2025-06-18 Streamable HTTP transport support. The implementation integrates seamlessly with existing Phase 4 infrastructure while establishing the core for all subsequent reverse proxy functionality.

## Prerequisites Validation

Before starting implementation:
1. Verify shadowcat is checked out as git submodule
2. Confirm all Phase 4 tests pass: `cd shadowcat && cargo test`
3. Verify Rust toolchain is up to date: `rustup update`
4. Ensure clean working directory: `git status`

## Detailed Implementation Steps

### Step 1: Dependencies & Configuration (30 minutes)

#### 1.1 Update Cargo.toml
```toml
# Add to [dependencies] section, maintaining alphabetical order
hex = "0.4"
ring = "0.17"

# Verify existing dependencies are at correct versions
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }
hyper = "1.5"
```

#### 1.2 Verify Dependency Compatibility
```bash
cd shadowcat
cargo check
cargo tree | grep -E "(axum|tower|hyper|hex|ring)"
```

#### 1.3 Create Feature Flag (Optional)
```toml
[features]
default = ["reverse-proxy"]
reverse-proxy = ["dep:hex", "dep:ring"]
```

**Validation Checkpoint**: All dependencies resolve without conflicts

### Step 2: Error Handling Extensions (30 minutes)

#### 2.1 Extend src/error.rs
```rust
// Add new error variants
#[derive(Error, Debug)]
pub enum ReverseProxyError {
    #[error("Bind failed: {0}")]
    BindFailed(String),
    
    #[error("Invalid MCP headers: {0}")]
    InvalidHeaders(String),
    
    #[error("Session creation failed: {0}")]
    SessionCreationFailed(String),
    
    #[error("Upstream connection failed: {0}")]
    UpstreamConnectionFailed(String),
    
    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    ProtocolVersionMismatch { expected: String, actual: String },
    
    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },
}

// Add HTTP status mapping
impl ReverseProxyError {
    pub fn to_http_status(&self) -> StatusCode {
        match self {
            Self::InvalidHeaders(_) => StatusCode::BAD_REQUEST,
            Self::SessionCreationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ProtocolVersionMismatch { .. } => StatusCode::BAD_REQUEST,
            Self::UpstreamConnectionFailed(_) => StatusCode::BAD_GATEWAY,
            Self::HttpError { status, .. } => StatusCode::from_u16(*status)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// MCP error to HTTP status mapping
pub fn mcp_error_to_http_status(code: i32) -> StatusCode {
    match code {
        -32700 => StatusCode::BAD_REQUEST,          // Parse error
        -32600 => StatusCode::BAD_REQUEST,          // Invalid Request
        -32601 => StatusCode::NOT_FOUND,            // Method not found
        -32602 => StatusCode::BAD_REQUEST,          // Invalid params
        -32603 => StatusCode::INTERNAL_SERVER_ERROR, // Internal error
        -32000..=-32099 => StatusCode::BAD_REQUEST, // Server defined errors
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
```

#### 2.2 Add to ShadowcatError
```rust
#[error("Reverse proxy error: {0}")]
ReverseProxy(#[from] ReverseProxyError),
```

#### 2.3 Write Unit Tests
```rust
#[cfg(test)]
mod reverse_proxy_error_tests {
    use super::*;
    
    #[test]
    fn test_error_to_http_status() {
        // Test each error variant mapping
    }
    
    #[test]
    fn test_mcp_error_mapping() {
        // Test MCP error code mappings
    }
}
```

**Validation Checkpoint**: `cargo test error::reverse_proxy_error_tests`

### Step 3: HTTP Transport Extensions (1.5 hours)

#### 3.1 Create src/transport/http_mcp.rs
```rust
use super::{Transport, TransportMessage, SessionId, MCP_PROTOCOL_VERSION};
use crate::error::{TransportResult, TransportError, ReverseProxyError};
use async_trait::async_trait;
use axum::extract::{State, HeaderMap};
use axum::http::StatusCode;
use ring::rand::{SystemRandom, SecureRandom};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, instrument};

/// MCP Streamable HTTP server-side transport
pub struct HttpMcpTransport {
    session_id: SessionId,
    protocol_version: String,
    message_tx: mpsc::Sender<TransportMessage>,
    message_rx: mpsc::Receiver<TransportMessage>,
    connected: bool,
}

impl HttpMcpTransport {
    /// Create new HTTP MCP transport with secure session ID
    pub fn new() -> TransportResult<Self> {
        let session_id = generate_secure_session_id()?;
        let (tx, rx) = mpsc::channel(100);
        
        Ok(Self {
            session_id,
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            message_tx: tx,
            message_rx: rx,
            connected: false,
        })
    }
    
    /// Extract MCP headers from HTTP request
    pub fn extract_mcp_headers(headers: &HeaderMap) -> Result<McpHeaders, ReverseProxyError> {
        let session_id = headers
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ReverseProxyError::InvalidHeaders(
                "Missing MCP-Session-Id header".to_string()
            ))?;
            
        let protocol_version = headers
            .get("mcp-protocol-version")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(MCP_PROTOCOL_VERSION);
            
        // Validate protocol version
        if !is_compatible_version(protocol_version) {
            return Err(ReverseProxyError::ProtocolVersionMismatch {
                expected: MCP_PROTOCOL_VERSION.to_string(),
                actual: protocol_version.to_string(),
            });
        }
        
        Ok(McpHeaders {
            session_id: session_id.to_string(),
            protocol_version: protocol_version.to_string(),
        })
    }
}

/// Generate cryptographically secure session ID
pub fn generate_secure_session_id() -> TransportResult<SessionId> {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes)
        .map_err(|_| TransportError::Protocol("Failed to generate random bytes".to_string()))?;
    
    let session_str = hex::encode(bytes);
    // Parse as UUID-like format for compatibility
    let uuid = uuid::Uuid::new_v4();
    Ok(SessionId(uuid))
}

/// Check if protocol version is compatible
fn is_compatible_version(version: &str) -> bool {
    match version {
        "2025-11-05" => true, // Current version
        "2025-06-18" => true, // Streamable HTTP version
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub struct McpHeaders {
    pub session_id: String,
    pub protocol_version: String,
}

// Transport trait implementation
#[async_trait]
impl Transport for HttpMcpTransport {
    async fn connect(&mut self) -> TransportResult<()> {
        self.connected = true;
        Ok(())
    }
    
    async fn send(&mut self, msg: TransportMessage) -> TransportResult<()> {
        self.message_tx.send(msg).await
            .map_err(|_| TransportError::SendFailed("Channel closed".to_string()))
    }
    
    async fn receive(&mut self) -> TransportResult<TransportMessage> {
        self.message_rx.recv().await
            .ok_or(TransportError::Closed)
    }
    
    async fn close(&mut self) -> TransportResult<()> {
        self.connected = false;
        Ok(())
    }
    
    fn session_id(&self) -> &SessionId {
        &self.session_id
    }
    
    fn transport_type(&self) -> super::TransportType {
        super::TransportType::Http
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secure_session_id_generation() {
        let id1 = generate_secure_session_id().unwrap();
        let id2 = generate_secure_session_id().unwrap();
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_version_compatibility() {
        assert!(is_compatible_version("2025-11-05"));
        assert!(is_compatible_version("2025-06-18"));
        assert!(!is_compatible_version("2024-01-01"));
    }
    
    #[test]
    fn test_header_extraction() {
        let mut headers = HeaderMap::new();
        headers.insert("mcp-session-id", "test-session".parse().unwrap());
        headers.insert("mcp-protocol-version", "2025-11-05".parse().unwrap());
        
        let result = HttpMcpTransport::extract_mcp_headers(&headers);
        assert!(result.is_ok());
        
        let mcp_headers = result.unwrap();
        assert_eq!(mcp_headers.session_id, "test-session");
        assert_eq!(mcp_headers.protocol_version, "2025-11-05");
    }
}
```

#### 3.2 Update src/transport/mod.rs
```rust
pub mod http_mcp;
pub use http_mcp::{HttpMcpTransport, McpHeaders, generate_secure_session_id};
```

**Validation Checkpoint**: `cargo test transport::http_mcp`

### Step 4: Reverse Proxy Server Core (2 hours)

#### 4.1 Implement src/proxy/reverse.rs
```rust
use crate::error::{Result, ReverseProxyError};
use crate::session::{SessionManager, SessionId};
use crate::transport::{TransportMessage, HttpMcpTransport, generate_secure_session_id};
use axum::{
    Router,
    routing::{get, post},
    extract::{State, Json},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;
use tracing::{info, instrument, error};

/// Reverse proxy server for MCP protocol
pub struct ReverseProxyServer {
    bind_address: SocketAddr,
    router: Router,
    session_manager: Arc<SessionManager>,
    config: ReverseProxyConfig,
}

/// Reverse proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseProxyConfig {
    pub bind_address: SocketAddr,
    pub session_config: SessionConfig,
    pub cors_enabled: bool,
    pub trace_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_timeout_secs: u64,
    pub max_sessions: usize,
    pub cleanup_interval_secs: u64,
}

impl Default for ReverseProxyConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".parse().unwrap(),
            session_config: SessionConfig {
                session_timeout_secs: 300, // 5 minutes
                max_sessions: 1000,
                cleanup_interval_secs: 60,
            },
            cors_enabled: true,
            trace_enabled: true,
        }
    }
}

/// Application state shared across requests
#[derive(Clone)]
struct AppState {
    session_manager: Arc<SessionManager>,
    config: ReverseProxyConfig,
}

impl ReverseProxyServer {
    /// Create new reverse proxy server
    pub fn new(config: ReverseProxyConfig, session_manager: Arc<SessionManager>) -> Self {
        let app_state = AppState {
            session_manager: session_manager.clone(),
            config: config.clone(),
        };
        
        let router = create_router(app_state, &config);
        
        Self {
            bind_address: config.bind_address,
            router,
            session_manager,
            config,
        }
    }
    
    /// Start the server
    #[instrument(skip(self))]
    pub async fn start(self) -> Result<()> {
        info!("Starting reverse proxy server on {}", self.bind_address);
        
        let listener = TcpListener::bind(self.bind_address)
            .await
            .map_err(|e| ReverseProxyError::BindFailed(e.to_string()))?;
            
        info!("Reverse proxy listening on {}", self.bind_address);
        
        axum::serve(listener, self.router)
            .await
            .map_err(|e| ReverseProxyError::BindFailed(e.to_string()))?;
            
        Ok(())
    }
}

/// Create the Axum router with all endpoints
fn create_router(app_state: AppState, config: &ReverseProxyConfig) -> Router {
    let mut router = Router::new()
        .route("/mcp", post(handle_mcp_request))
        .route("/health", get(handle_health))
        .route("/metrics", get(handle_metrics))
        .with_state(app_state);
        
    // Add middleware layers
    let service_builder = ServiceBuilder::new();
    
    if config.trace_enabled {
        router = router.layer(service_builder.layer(TraceLayer::new_for_http()));
    }
    
    if config.cors_enabled {
        router = router.layer(CorsLayer::permissive());
    }
    
    router
}

/// Handle MCP protocol requests
#[instrument(skip(app_state, headers, body))]
async fn handle_mcp_request(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, ReverseProxyError> {
    // Extract and validate MCP headers
    let mcp_headers = HttpMcpTransport::extract_mcp_headers(&headers)?;
    
    // Parse JSON-RPC message to TransportMessage
    let transport_msg = parse_json_rpc_to_transport(&body)
        .map_err(|e| ReverseProxyError::InvalidHeaders(e.to_string()))?;
    
    // Get or create session
    let session_id = SessionId(uuid::Uuid::parse_str(&mcp_headers.session_id)
        .map_err(|_| ReverseProxyError::InvalidHeaders("Invalid session ID format".to_string()))?);
    
    // Process through session manager
    let response = process_message_through_session(
        &app_state.session_manager,
        session_id,
        transport_msg
    ).await?;
    
    // Convert response back to HTTP
    let json_response = transport_to_json_rpc(&response)?;
    
    Ok(Json(json_response).into_response())
}

/// Health check endpoint
async fn handle_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol_version": crate::transport::MCP_PROTOCOL_VERSION,
    }))
}

/// Basic metrics endpoint
async fn handle_metrics(State(app_state): State<AppState>) -> impl IntoResponse {
    let stats = app_state.session_manager
        .get_session_stats()
        .await
        .unwrap_or_default();
        
    // Return Prometheus-style metrics
    format!(
        "# HELP shadowcat_sessions_total Total number of sessions\n\
         # TYPE shadowcat_sessions_total counter\n\
         shadowcat_sessions_total {}\n\
         # HELP shadowcat_sessions_active Active sessions\n\
         # TYPE shadowcat_sessions_active gauge\n\
         shadowcat_sessions_active {}\n\
         # HELP shadowcat_frames_total Total frames processed\n\
         # TYPE shadowcat_frames_total counter\n\
         shadowcat_frames_total {}\n",
        stats.total, stats.active, stats.total_frames
    )
}

/// Parse JSON-RPC to TransportMessage
fn parse_json_rpc_to_transport(value: &serde_json::Value) -> Result<TransportMessage> {
    // Implementation matches existing pattern in http.rs
    let obj = value.as_object()
        .ok_or_else(|| ReverseProxyError::InvalidHeaders("Not a JSON object".to_string()))?;
        
    if let Some(method) = obj.get("method").and_then(|m| m.as_str()) {
        let id = obj.get("id").map(|id| match id {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => "invalid".to_string(),
        });
        
        let params = obj.get("params").cloned().unwrap_or(serde_json::Value::Null);
        
        if let Some(id) = id {
            Ok(TransportMessage::Request {
                id,
                method: method.to_string(),
                params,
            })
        } else {
            Ok(TransportMessage::Notification {
                method: method.to_string(),
                params,
            })
        }
    } else if let Some(id) = obj.get("id") {
        let id_str = match id {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => return Err(ReverseProxyError::InvalidHeaders("Invalid ID".to_string()).into()),
        };
        
        let result = obj.get("result").cloned();
        let error = obj.get("error").cloned();
        
        Ok(TransportMessage::Response { id: id_str, result, error })
    } else {
        Err(ReverseProxyError::InvalidHeaders("Invalid JSON-RPC message".to_string()).into())
    }
}

/// Convert TransportMessage back to JSON-RPC
fn transport_to_json_rpc(msg: &TransportMessage) -> Result<serde_json::Value> {
    use serde_json::json;
    
    Ok(match msg {
        TransportMessage::Request { id, method, params } => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": method,
                "params": params,
            })
        }
        TransportMessage::Response { id, result, error } => {
            let mut response = json!({
                "jsonrpc": "2.0",
                "id": id,
            });
            
            if let Some(result) = result {
                response["result"] = result.clone();
            }
            if let Some(error) = error {
                response["error"] = error.clone();
            }
            
            response
        }
        TransportMessage::Notification { method, params } => {
            json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
            })
        }
    })
}

/// Process message through session manager
async fn process_message_through_session(
    session_manager: &SessionManager,
    session_id: SessionId,
    message: TransportMessage,
) -> Result<TransportMessage> {
    // Get or create session
    let session = match session_manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(_) => {
            // Create new session
            session_manager.create_session(
                session_id.clone(),
                crate::transport::TransportType::Http
            ).await?
        }
    };
    
    // Record incoming frame
    session_manager.record_frame(
        &session_id,
        crate::transport::Direction::ClientToServer,
        message.clone()
    ).await?;
    
    // For now, echo back the message (will be replaced with actual proxy logic)
    // This is just for testing the infrastructure
    let response = match message {
        TransportMessage::Request { id, .. } => {
            TransportMessage::Response {
                id,
                result: Some(serde_json::json!({
                    "status": "received",
                    "session": session_id.to_string(),
                })),
                error: None,
            }
        }
        _ => message,
    };
    
    // Record outgoing frame
    session_manager.record_frame(
        &session_id,
        crate::transport::Direction::ServerToClient,
        response.clone()
    ).await?;
    
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_json_rpc_parsing() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": "initialize",
            "params": {"capabilities": {}}
        });
        
        let result = parse_json_rpc_to_transport(&request);
        assert!(result.is_ok());
        
        match result.unwrap() {
            TransportMessage::Request { id, method, .. } => {
                assert_eq!(id, "1");
                assert_eq!(method, "initialize");
            }
            _ => panic!("Expected request"),
        }
    }
    
    #[test]
    fn test_transport_to_json_rpc() {
        let msg = TransportMessage::Response {
            id: "1".to_string(),
            result: Some(json!({"status": "ok"})),
            error: None,
        };
        
        let result = transport_to_json_rpc(&msg);
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], "1");
        assert_eq!(json["result"]["status"], "ok");
    }
    
    #[test]
    fn test_config_default() {
        let config = ReverseProxyConfig::default();
        assert_eq!(config.session_config.session_timeout_secs, 300);
        assert_eq!(config.session_config.max_sessions, 1000);
        assert!(config.cors_enabled);
        assert!(config.trace_enabled);
    }
}
```

**Validation Checkpoint**: `cargo test proxy::reverse`

### Step 5: Router Setup & Request Handling (2 hours)

This step is integrated into Step 4 above. The key components are:

1. **Router Creation**: The `create_router` function sets up all endpoints with middleware
2. **MCP Request Handler**: The `handle_mcp_request` function processes incoming requests
3. **Health Check**: Simple endpoint for monitoring
4. **Metrics**: Prometheus-format metrics endpoint

Additional implementation details for production readiness:

#### 5.1 Enhanced Error Handling
```rust
/// Custom error response for Axum
impl IntoResponse for ReverseProxyError {
    fn into_response(self) -> Response {
        let status = self.to_http_status();
        let body = Json(serde_json::json!({
            "error": {
                "code": status.as_u16(),
                "message": self.to_string(),
            }
        }));
        
        (status, body).into_response()
    }
}
```

#### 5.2 Request Validation Middleware
```rust
/// Validate incoming requests
async fn validate_request(
    headers: &HeaderMap,
    body: &serde_json::Value,
) -> Result<(), ReverseProxyError> {
    // Check content type
    if let Some(content_type) = headers.get("content-type") {
        if !content_type.to_str().unwrap_or("").starts_with("application/json") {
            return Err(ReverseProxyError::InvalidHeaders(
                "Content-Type must be application/json".to_string()
            ));
        }
    }
    
    // Validate JSON-RPC structure
    if !body.is_object() || !body.get("jsonrpc").is_some() {
        return Err(ReverseProxyError::InvalidHeaders(
            "Invalid JSON-RPC format".to_string()
        ));
    }
    
    Ok(())
}
```

### Step 6: Configuration & CLI Integration (1 hour)

#### 6.1 Create src/config/reverse_proxy.rs
```rust
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseProxySettings {
    pub server: ServerSettings,
    pub session: SessionSettings,
    pub security: SecuritySettings,
    pub monitoring: MonitoringSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub bind_address: SocketAddr,
    pub worker_threads: Option<usize>,
    pub max_connections: usize,
    pub shutdown_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    pub timeout_secs: u64,
    pub max_sessions: usize,
    pub cleanup_interval_secs: u64,
    pub storage_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub cors_enabled: bool,
    pub cors_origins: Vec<String>,
    pub tls_enabled: bool,
    pub tls_cert_path: Option<PathBuf>,
    pub tls_key_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    pub trace_enabled: bool,
    pub metrics_enabled: bool,
    pub health_check_enabled: bool,
    pub log_level: String,
}

impl Default for ReverseProxySettings {
    fn default() -> Self {
        Self {
            server: ServerSettings {
                bind_address: "127.0.0.1:8080".parse().unwrap(),
                worker_threads: None, // Use tokio default
                max_connections: 1000,
                shutdown_timeout_secs: 30,
            },
            session: SessionSettings {
                timeout_secs: 300,
                max_sessions: 1000,
                cleanup_interval_secs: 60,
                storage_path: None,
            },
            security: SecuritySettings {
                cors_enabled: true,
                cors_origins: vec!["*".to_string()],
                tls_enabled: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
            monitoring: MonitoringSettings {
                trace_enabled: true,
                metrics_enabled: true,
                health_check_enabled: true,
                log_level: "info".to_string(),
            },
        }
    }
}

/// Load configuration from file or environment
pub fn load_config(path: Option<PathBuf>) -> Result<ReverseProxySettings> {
    if let Some(path) = path {
        let contents = std::fs::read_to_string(path)?;
        let config: ReverseProxySettings = serde_yaml::from_str(&contents)?;
        Ok(config)
    } else {
        Ok(ReverseProxySettings::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = ReverseProxySettings::default();
        assert_eq!(config.server.max_connections, 1000);
        assert_eq!(config.session.timeout_secs, 300);
        assert!(config.security.cors_enabled);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = ReverseProxySettings::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: ReverseProxySettings = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.server.bind_address, parsed.server.bind_address);
    }
}
```

#### 6.2 Update src/config/mod.rs
```rust
pub mod reverse_proxy;
pub use reverse_proxy::{ReverseProxySettings, load_config};
```

#### 6.3 Update src/main.rs CLI Integration
```rust
// Add to Commands enum
#[command(about = "Run reverse proxy with authentication")]
ReverseProxy {
    #[arg(long, help = "Bind address", default_value = "127.0.0.1:8080")]
    bind: String,
    
    #[arg(long, help = "Configuration file path")]
    config: Option<PathBuf>,
    
    #[arg(long, help = "Enable debug mode")]
    debug: bool,
},

// Add handler in main()
Commands::ReverseProxy { bind, config, debug } => {
    let settings = config::reverse_proxy::load_config(config)?;
    
    // Override bind address if provided
    let bind_addr = if bind != "127.0.0.1:8080" {
        bind.parse()?
    } else {
        settings.server.bind_address
    };
    
    // Create session manager
    let session_manager = Arc::new(SessionManager::new()
        .with_timeout(Duration::from_secs(settings.session.timeout_secs)));
    
    // Create and configure reverse proxy
    let config = ReverseProxyConfig {
        bind_address: bind_addr,
        session_config: SessionConfig {
            session_timeout_secs: settings.session.timeout_secs,
            max_sessions: settings.session.max_sessions,
            cleanup_interval_secs: settings.session.cleanup_interval_secs,
        },
        cors_enabled: settings.security.cors_enabled,
        trace_enabled: settings.monitoring.trace_enabled,
    };
    
    let server = ReverseProxyServer::new(config, session_manager);
    
    info!("Starting Shadowcat reverse proxy on {}", bind_addr);
    server.start().await?;
}
```

**Validation Checkpoint**: `cargo run -- reverse-proxy --help`

### Step 7: Integration Testing (1.5 hours)

#### 7.1 Create tests/integration/reverse_proxy_basic.rs
```rust
use shadowcat::proxy::reverse::{ReverseProxyServer, ReverseProxyConfig};
use shadowcat::session::SessionManager;
use shadowcat::transport::TransportMessage;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_server_startup_shutdown() {
    let config = ReverseProxyConfig::default();
    let session_manager = Arc::new(SessionManager::new());
    
    let server = ReverseProxyServer::new(config, session_manager);
    
    // Start server in background
    let handle = tokio::spawn(async move {
        server.start().await
    });
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Abort the server
    handle.abort();
    assert!(handle.await.unwrap_err().is_cancelled());
}

#[tokio::test]
async fn test_health_check_endpoint() {
    let (addr, _handle) = start_test_server().await;
    
    let client = Client::new();
    let response = client
        .get(format!("http://{}/health", addr))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
    assert!(body["protocol_version"].is_string());
}

#[tokio::test]
async fn test_mcp_request_processing() {
    let (addr, _handle) = start_test_server().await;
    
    let client = Client::new();
    let request_body = json!({
        "jsonrpc": "2.0",
        "id": "test-1",
        "method": "initialize",
        "params": {
            "capabilities": {}
        }
    });
    
    let response = client
        .post(format!("http://{}/mcp", addr))
        .header("Content-Type", "application/json")
        .header("MCP-Session-Id", "test-session-123")
        .header("MCP-Protocol-Version", "2025-11-05")
        .json(&request_body)
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], "test-1");
    assert!(body["result"].is_object());
}

#[tokio::test]
async fn test_missing_headers_error() {
    let (addr, _handle) = start_test_server().await;
    
    let client = Client::new();
    let request_body = json!({
        "jsonrpc": "2.0",
        "id": "test-1",
        "method": "test"
    });
    
    // Missing MCP-Session-Id header
    let response = client
        .post(format!("http://{}/mcp", addr))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 400);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["error"]["message"].as_str().unwrap().contains("MCP-Session-Id"));
}

#[tokio::test]
async fn test_protocol_version_mismatch() {
    let (addr, _handle) = start_test_server().await;
    
    let client = Client::new();
    let request_body = json!({
        "jsonrpc": "2.0",
        "id": "test-1",
        "method": "test"
    });
    
    let response = client
        .post(format!("http://{}/mcp", addr))
        .header("Content-Type", "application/json")
        .header("MCP-Session-Id", "test-session")
        .header("MCP-Protocol-Version", "2024-01-01") // Invalid version
        .json(&request_body)
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 400);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["error"]["message"].as_str().unwrap().contains("version"));
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let (addr, _handle) = start_test_server().await;
    
    // Make a request to create a session
    let client = Client::new();
    let _ = client
        .post(format!("http://{}/mcp", addr))
        .header("Content-Type", "application/json")
        .header("MCP-Session-Id", "metrics-test")
        .header("MCP-Protocol-Version", "2025-11-05")
        .json(&json!({"jsonrpc": "2.0", "id": "1", "method": "test"}))
        .send()
        .await
        .unwrap();
    
    // Check metrics
    let response = client
        .get(format!("http://{}/metrics", addr))
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200);
    
    let body = response.text().await.unwrap();
    assert!(body.contains("shadowcat_sessions_total"));
    assert!(body.contains("shadowcat_sessions_active"));
    assert!(body.contains("shadowcat_frames_total"));
}

#[tokio::test]
async fn test_concurrent_requests() {
    let (addr, _handle) = start_test_server().await;
    
    let client = Arc::new(Client::new());
    let mut handles = vec![];
    
    // Send 10 concurrent requests
    for i in 0..10 {
        let client = client.clone();
        let addr = addr.clone();
        
        let handle = tokio::spawn(async move {
            let response = client
                .post(format!("http://{}/mcp", addr))
                .header("Content-Type", "application/json")
                .header("MCP-Session-Id", format!("concurrent-{}", i))
                .header("MCP-Protocol-Version", "2025-11-05")
                .json(&json!({
                    "jsonrpc": "2.0",
                    "id": format!("req-{}", i),
                    "method": "test"
                }))
                .send()
                .await
                .unwrap();
                
            assert_eq!(response.status(), 200);
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

// Helper function to start test server
async fn start_test_server() -> (String, tokio::task::JoinHandle<()>) {
    let port = get_available_port();
    let addr = format!("127.0.0.1:{}", port);
    let bind_addr = addr.parse().unwrap();
    
    let config = ReverseProxyConfig {
        bind_address: bind_addr,
        ..Default::default()
    };
    
    let session_manager = Arc::new(SessionManager::new());
    let server = ReverseProxyServer::new(config, session_manager);
    
    let handle = tokio::spawn(async move {
        server.start().await.unwrap();
    });
    
    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    (addr, handle)
}

fn get_available_port() -> u16 {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}
```

**Validation Checkpoint**: `cargo test --test reverse_proxy_basic`

### Step 8: Performance Benchmarking (1 hour)

#### 8.1 Create benches/reverse_proxy_bench.rs
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use shadowcat::transport::{TransportMessage, generate_secure_session_id};
use shadowcat::proxy::reverse::{parse_json_rpc_to_transport, transport_to_json_rpc};
use serde_json::json;
use std::time::Duration;

fn bench_session_id_generation(c: &mut Criterion) {
    c.bench_function("generate_secure_session_id", |b| {
        b.iter(|| {
            let _ = generate_secure_session_id();
        });
    });
}

fn bench_json_parsing(c: &mut Criterion) {
    let request = json!({
        "jsonrpc": "2.0",
        "id": "123",
        "method": "test",
        "params": {"foo": "bar", "baz": 42}
    });
    
    c.bench_function("parse_json_rpc_to_transport", |b| {
        b.iter(|| {
            let _ = parse_json_rpc_to_transport(black_box(&request));
        });
    });
}

fn bench_message_conversion(c: &mut Criterion) {
    let msg = TransportMessage::Request {
        id: "123".to_string(),
        method: "test".to_string(),
        params: json!({"foo": "bar"}),
    };
    
    c.bench_function("transport_to_json_rpc", |b| {
        b.iter(|| {
            let _ = transport_to_json_rpc(black_box(&msg));
        });
    });
}

fn bench_concurrent_session_creation(c: &mut Criterion) {
    use shadowcat::session::SessionManager;
    use std::sync::Arc;
    
    let mut group = c.benchmark_group("concurrent_sessions");
    
    for num_sessions in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_sessions),
            num_sessions,
            |b, &num| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async move {
                        let manager = Arc::new(SessionManager::new());
                        let mut handles = vec![];
                        
                        for _ in 0..num {
                            let manager = manager.clone();
                            let handle = tokio::spawn(async move {
                                let session_id = generate_secure_session_id().unwrap();
                                manager.create_session(
                                    session_id,
                                    shadowcat::transport::TransportType::Http
                                ).await.unwrap();
                            });
                            handles.push(handle);
                        }
                        
                        for handle in handles {
                            handle.await.unwrap();
                        }
                    });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_session_id_generation,
    bench_json_parsing,
    bench_message_conversion,
    bench_concurrent_session_creation
);
criterion_main!(benches);
```

#### 8.2 Update Cargo.toml for benchmarks
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }

[[bench]]
name = "reverse_proxy_bench"
harness = false
```

#### 8.3 Run Performance Tests
```bash
cargo bench --bench reverse_proxy_bench

# Generate flamegraph for profiling
cargo install flamegraph
cargo flamegraph --bench reverse_proxy_bench
```

**Validation Checkpoint**: Benchmark results show < 1ms HTTP overhead

## Validation & Testing Strategy

### Progressive Validation
1. **Compile Check**: After each step, run `cargo check`
2. **Unit Tests**: Run module-specific tests after implementation
3. **Integration Tests**: Run full integration suite after Step 7
4. **Performance Tests**: Validate targets in Step 8

### Test Coverage Goals
- Unit test coverage > 80%
- All error paths tested
- All public APIs have tests
- Integration tests cover happy path and error scenarios

### Manual Testing Commands
```bash
# Start the reverse proxy
cargo run -- reverse-proxy --bind 127.0.0.1:8080 --debug

# Test health check
curl http://localhost:8080/health

# Test MCP request
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -H "MCP-Session-Id: test-session-123" \
  -H "MCP-Protocol-Version: 2025-11-05" \
  -d '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{}}'

# Test metrics
curl http://localhost:8080/metrics
```

## Troubleshooting Guide

### Common Issues

1. **Port Already in Use**
   - Error: `BindFailed: Address already in use`
   - Solution: Change port or kill existing process

2. **Missing Dependencies**
   - Error: `cannot find crate`
   - Solution: Run `cargo update` and check Cargo.toml

3. **Session ID Generation Fails**
   - Error: `Failed to generate random bytes`
   - Solution: Ensure ring is properly installed, check system entropy

4. **Protocol Version Mismatch**
   - Error: `ProtocolVersionMismatch`
   - Solution: Update MCP_PROTOCOL_VERSION constant or accept multiple versions

## Next Steps

Upon successful completion of Task 001:
1. Commit all changes with comprehensive test coverage
2. Update task tracker to mark Task 001 as complete
3. Proceed to Task 002: OAuth 2.1 Flow Implementation
4. The HTTP server foundation will support all subsequent authentication features

## Success Metrics

- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ Performance benchmarks meet targets (< 1ms overhead)
- ✅ Manual testing validates all endpoints
- ✅ Code coverage > 80%
- ✅ No security vulnerabilities in dependencies
- ✅ Documentation complete and accurate