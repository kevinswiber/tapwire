# Foundation Component Tasks

These tasks create the shared components needed by both SSE proxy integration and MCP message handling.

## F.1: Protocol Version Manager

**Duration**: 2 hours  
**Dependencies**: None  
**File**: `src/mcp/protocol.rs`

### Implementation

```rust
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolVersion {
    V2025_03_26,
    V2025_06_18,
}

impl ProtocolVersion {
    pub const DEFAULT: Self = Self::V2025_03_26;
    
    pub fn supports_batching(&self) -> bool {
        matches!(self, Self::V2025_03_26)
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V2025_03_26 => "2025-03-26",
            Self::V2025_06_18 => "2025-06-18",
        }
    }
    
    pub fn from_header(header: Option<&str>) -> Self {
        header
            .and_then(|h| Self::from_str(h).ok())
            .unwrap_or(Self::DEFAULT)
    }
}

impl FromStr for ProtocolVersion {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "2025-03-26" => Ok(Self::V2025_03_26),
            "2025-06-18" => Ok(Self::V2025_06_18),
            _ => Err(format!("Unknown protocol version: {}", s)),
        }
    }
}

pub struct VersionNegotiator {
    supported: Vec<ProtocolVersion>,
}

impl VersionNegotiator {
    pub fn new() -> Self {
        Self {
            supported: vec![
                ProtocolVersion::V2025_06_18,
                ProtocolVersion::V2025_03_26,
            ],
        }
    }
    
    pub fn negotiate(&self, client_versions: &[String]) -> Option<ProtocolVersion> {
        for version_str in client_versions {
            if let Ok(version) = ProtocolVersion::from_str(version_str) {
                if self.supported.contains(&version) {
                    return Some(version);
                }
            }
        }
        None
    }
}
```

## F.2: Minimal MCP Parser

**Duration**: 4 hours  
**Dependencies**: None  
**File**: `src/mcp/early_parser.rs`

### Implementation

```rust
use serde_json::Value;
use crate::mcp::protocol::ProtocolVersion;

pub struct MinimalMcpParser {
    version: ProtocolVersion,
}

impl MinimalMcpParser {
    pub fn new(version: ProtocolVersion) -> Self {
        Self { version }
    }
    
    pub fn parse(&self, data: &str) -> Result<ParsedInfo, ParseError> {
        let value: Value = serde_json::from_str(data)?;
        
        Ok(ParsedInfo {
            is_batch: value.is_array(),
            message_count: self.count_messages(&value),
            messages: self.extract_messages(value)?,
            version: self.version,
        })
    }
    
    fn count_messages(&self, value: &Value) -> usize {
        match value {
            Value::Array(arr) => arr.len(),
            _ => 1,
        }
    }
    
    fn extract_messages(&self, value: Value) -> Result<Vec<MinimalMessage>, ParseError> {
        let values = match value {
            Value::Array(arr) => arr,
            single => vec![single],
        };
        
        values.into_iter()
            .map(|v| self.parse_single(v))
            .collect()
    }
    
    fn parse_single(&self, value: Value) -> Result<MinimalMessage, ParseError> {
        // Validate JSON-RPC 2.0
        if value.get("jsonrpc") != Some(&Value::String("2.0".to_string())) {
            return Err(ParseError::InvalidJsonRpc);
        }
        
        let id = value.get("id").cloned();
        let method = value.get("method")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let message_type = match (&id, &method) {
            (Some(_), Some(_)) => MessageType::Request,
            (Some(_), None) => MessageType::Response,
            (None, Some(_)) => MessageType::Notification,
            _ => return Err(ParseError::InvalidStructure),
        };
        
        Ok(MinimalMessage {
            message_type,
            id,
            method,
            raw: value,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ParsedInfo {
    pub is_batch: bool,
    pub message_count: usize,
    pub messages: Vec<MinimalMessage>,
    pub version: ProtocolVersion,
}

#[derive(Debug, Clone)]
pub struct MinimalMessage {
    pub message_type: MessageType,
    pub id: Option<Value>,
    pub method: Option<String>,
    pub raw: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Request,
    Response,
    Notification,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("JSON parsing failed")]
    Json(#[from] serde_json::Error),
    
    #[error("Not a valid JSON-RPC 2.0 message")]
    InvalidJsonRpc,
    
    #[error("Invalid message structure")]
    InvalidStructure,
}
```

## F.3: Batch Handler

**Duration**: 3 hours  
**Dependencies**: F.1, F.2  
**File**: `src/mcp/batch.rs`

### Implementation

```rust
use serde_json::Value;
use crate::mcp::protocol::ProtocolVersion;
use crate::mcp::early_parser::MinimalMessage;

pub struct BatchHandler {
    version: ProtocolVersion,
}

impl BatchHandler {
    pub fn new(version: ProtocolVersion) -> Self {
        Self { version }
    }
    
    pub fn should_batch(&self, messages: &[MinimalMessage]) -> bool {
        self.version.supports_batching() && messages.len() > 1
    }
    
    pub fn split_if_batch(&self, value: Value) -> Vec<Value> {
        match value {
            Value::Array(arr) if self.version.supports_batching() => arr,
            single => vec![single],
        }
    }
    
    pub fn combine_if_needed(&self, messages: Vec<Value>) -> Value {
        if self.should_batch_values(&messages) {
            Value::Array(messages)
        } else {
            messages.into_iter().next().unwrap_or(Value::Null)
        }
    }
    
    fn should_batch_values(&self, messages: &[Value]) -> bool {
        self.version.supports_batching() && messages.len() > 1
    }
    
    pub fn group_by_type(&self, messages: Vec<MinimalMessage>) -> GroupedMessages {
        let mut requests = Vec::new();
        let mut responses = Vec::new();
        let mut notifications = Vec::new();
        
        for msg in messages {
            match msg.message_type {
                MessageType::Request => requests.push(msg),
                MessageType::Response => responses.push(msg),
                MessageType::Notification => notifications.push(msg),
            }
        }
        
        GroupedMessages {
            requests,
            responses,
            notifications,
        }
    }
}

pub struct GroupedMessages {
    pub requests: Vec<MinimalMessage>,
    pub responses: Vec<MinimalMessage>,
    pub notifications: Vec<MinimalMessage>,
}

impl GroupedMessages {
    pub fn has_requests(&self) -> bool {
        !self.requests.is_empty()
    }
    
    pub fn total_count(&self) -> usize {
        self.requests.len() + self.responses.len() + self.notifications.len()
    }
}
```

## F.4: Unified Event ID Generator ✅

**Duration**: 2 hours  
**Dependencies**: None  
**File**: `src/mcp/event_id.rs`  
**Status**: ✅ Completed (2025-08-10)

### Implementation Notes

**Completed Features**:
- Thread-safe ID generation using AtomicU64
- UUID-based node ID (8 char prefix) for uniqueness
- Session ID and JSON-RPC ID correlation support
- SSE compatibility (newlines replaced with underscores)
- Robust correlation extraction handling dashes in IDs
- 17 comprehensive tests including thread safety

**Key Improvements from Design**:
- More robust correlation extraction using pattern matching for node ID
- Proper handling of session IDs and JSON-RPC IDs containing dashes
- SSE newline compatibility built-in
- Better edge case handling

### Original Design (for reference)

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

pub struct UnifiedEventIdGenerator {
    node_id: String,
    counter: AtomicU64,
}

impl UnifiedEventIdGenerator {
    pub fn new() -> Self {
        Self {
            node_id: format!("{:x}", Uuid::new_v4().as_u128() & 0xFFFF),
            counter: AtomicU64::new(0),
        }
    }
    
    /// Generate an event ID that includes correlation information
    pub fn generate(
        &self,
        session_id: &str,
        json_rpc_id: Option<&serde_json::Value>,
    ) -> String {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        
        match json_rpc_id {
            Some(id) => {
                let id_str = match id {
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => "unknown".to_string(),
                };
                format!("{}-{}-{}-{}", session_id, self.node_id, id_str, count)
            }
            None => {
                format!("{}-{}-notif-{}", session_id, self.node_id, count)
            }
        }
    }
    
    /// Extract correlation information from an event ID
    pub fn extract_correlation(&self, event_id: &str) -> Option<CorrelationInfo> {
        let parts: Vec<&str> = event_id.split('-').collect();
        
        if parts.len() >= 4 {
            let session_id = parts[0].to_string();
            let node_id = parts[1].to_string();
            
            let json_rpc_id = if parts[2] != "notif" {
                Some(parts[2].to_string())
            } else {
                None
            };
            
            Some(CorrelationInfo {
                session_id,
                node_id,
                json_rpc_id,
                sequence: parts[3].parse().ok(),
            })
        } else {
            None
        }
    }
    
    /// Generate a simple ID without correlation
    pub fn generate_simple(&self) -> String {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        format!("{}-{}", self.node_id, count)
    }
}

#[derive(Debug, Clone)]
pub struct CorrelationInfo {
    pub session_id: String,
    pub node_id: String,
    pub json_rpc_id: Option<String>,
    pub sequence: Option<u64>,
}
```

## F.5: Message Context Structure

**Duration**: 2 hours  
**Dependencies**: F.1  
**File**: `src/mcp/context.rs`

### Implementation

```rust
use std::time::Instant;
use crate::mcp::protocol::ProtocolVersion;

#[derive(Debug, Clone)]
pub struct McpMessageContext {
    // Identity
    pub session_id: String,
    pub mcp_session_id: Option<String>,
    pub correlation_id: Option<String>,
    
    // Protocol
    pub protocol_version: ProtocolVersion,
    
    // Transport
    pub transport: TransportType,
    pub direction: MessageDirection,
    
    // Timing
    pub timestamp: Instant,
    pub processing_start: Option<Instant>,
    pub processing_end: Option<Instant>,
    
    // Processing flags
    pub intercepted: bool,
    pub modified: bool,
    pub recorded: bool,
    pub replayed: bool,
    
    // Metadata
    pub metadata: HashMap<String, Value>,
}

impl McpMessageContext {
    pub fn new(session_id: String, transport: TransportType) -> Self {
        Self {
            session_id,
            mcp_session_id: None,
            correlation_id: None,
            protocol_version: ProtocolVersion::DEFAULT,
            transport,
            direction: MessageDirection::Inbound,
            timestamp: Instant::now(),
            processing_start: None,
            processing_end: None,
            intercepted: false,
            modified: false,
            recorded: false,
            replayed: false,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_protocol(mut self, version: ProtocolVersion) -> Self {
        self.protocol_version = version;
        self
    }
    
    pub fn with_mcp_session(mut self, id: String) -> Self {
        self.mcp_session_id = Some(id);
        self
    }
    
    pub fn start_processing(&mut self) {
        self.processing_start = Some(Instant::now());
    }
    
    pub fn end_processing(&mut self) {
        self.processing_end = Some(Instant::now());
    }
    
    pub fn processing_duration(&self) -> Option<Duration> {
        match (self.processing_start, self.processing_end) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    Stdio,
    Http,
    Sse,
    WebSocket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDirection {
    Inbound,   // Client -> Proxy
    Outbound,  // Proxy -> Server
    Return,    // Server -> Proxy
    Response,  // Proxy -> Client
}
```

## Testing

Each component should have comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_version_default() {
        let version = ProtocolVersion::from_header(None);
        assert_eq!(version, ProtocolVersion::V2025_03_26);
    }
    
    #[test]
    fn test_minimal_parser_single() {
        let parser = MinimalMcpParser::new(ProtocolVersion::V2025_06_18);
        let result = parser.parse(r#"{"jsonrpc":"2.0","method":"test","id":1}"#).unwrap();
        assert!(!result.is_batch);
        assert_eq!(result.message_count, 1);
    }
    
    #[test]
    fn test_batch_handler() {
        let handler = BatchHandler::new(ProtocolVersion::V2025_03_26);
        let messages = vec![/* ... */];
        assert!(handler.should_batch(&messages));
    }
    
    #[test]
    fn test_event_id_correlation() {
        let gen = UnifiedEventIdGenerator::new();
        let id = gen.generate("session-1", Some(&json!(42)));
        let info = gen.extract_correlation(&id).unwrap();
        assert_eq!(info.json_rpc_id, Some("42".to_string()));
    }
}
```