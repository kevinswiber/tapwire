# Task 1.1: SSE Event Parser

## Task Metadata
- **Phase**: 1 (Core SSE Implementation)
- **Priority**: CRITICAL
- **Estimated Duration**: 3-4 hours
- **Dependencies**: Phase 0 complete
- **Status**: ⏳ Not Started

## Problem Statement

The current SSE implementation in `shadowcat/src/transport/http.rs:148-216` only handles basic message events. It lacks proper SSE format parsing for the full specification including event IDs, event types, retry directives, and multi-line data.

### Current Limitations
- Only processes `Event::Message` types
- No support for event IDs (for resumption)
- No handling of retry directives
- Incomplete multi-line data parsing
- Missing custom event type support

### SSE Format Specification
```
event: message\n
id: 123\n
retry: 10000\n
data: {"jsonrpc":"2.0",\n
data: "method":"ping",\n
data: "id":1}\n
\n
```

## Objectives

1. Implement complete SSE format parser
2. Support all SSE field types (data, event, id, retry)
3. Handle multi-line data fields correctly
4. Parse and validate JSON from data fields
5. Track event IDs for resumption support

## Implementation Plan

### Step 1: Define SSE Event Types
Create comprehensive event types:

```rust
// In shadowcat/src/transport/sse/parser.rs
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum SseField {
    Data(String),
    Event(String),
    Id(String),
    Retry(Duration),
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event_type: Option<String>,
    pub id: Option<String>,
    pub data: Vec<String>,  // Multiple data lines
    pub retry: Option<Duration>,
}

impl SseEvent {
    pub fn new() -> Self {
        Self {
            event_type: None,
            id: None,
            data: Vec::new(),
            retry: None,
        }
    }
    
    /// Parse complete data as JSON
    pub fn parse_json_data(&self) -> Result<Value, SseError> {
        let combined = self.data.join("\n");
        serde_json::from_str(&combined)
            .map_err(|e| SseError::InvalidJson(e))
    }
    
    /// Check if this is an MCP message
    pub fn is_mcp_message(&self) -> bool {
        self.event_type.as_deref() == Some("message") ||
        self.event_type.is_none()  // Default is message
    }
}
```

### Step 2: Implement SSE Parser
Create the parser state machine:

```rust
// In shadowcat/src/transport/sse/parser.rs
pub struct SseParser {
    current_event: SseEvent,
    last_event_id: Option<String>,
}

impl SseParser {
    pub fn new() -> Self {
        Self {
            current_event: SseEvent::new(),
            last_event_id: None,
        }
    }
    
    /// Parse a single SSE line
    pub fn parse_line(&mut self, line: &str) -> Result<Option<SseEvent>, SseError> {
        // Empty line signals end of event
        if line.is_empty() {
            if !self.current_event.data.is_empty() || 
               self.current_event.event_type.is_some() {
                let event = std::mem::replace(&mut self.current_event, SseEvent::new());
                
                // Update last event ID if present
                if let Some(ref id) = event.id {
                    self.last_event_id = Some(id.clone());
                }
                
                return Ok(Some(event));
            }
            return Ok(None);
        }
        
        // Parse field
        if let Some((field, value)) = self.parse_field(line)? {
            match field {
                "data" => {
                    self.current_event.data.push(value.to_string());
                }
                "event" => {
                    self.current_event.event_type = Some(value.to_string());
                }
                "id" => {
                    self.current_event.id = Some(value.to_string());
                }
                "retry" => {
                    if let Ok(ms) = value.parse::<u64>() {
                        self.current_event.retry = Some(Duration::from_millis(ms));
                    }
                }
                _ => {
                    // Ignore unknown fields per spec
                    debug!("Ignoring unknown SSE field: {}", field);
                }
            }
        }
        
        Ok(None)
    }
    
    /// Parse field:value format
    fn parse_field<'a>(&self, line: &'a str) -> Result<Option<(&'a str, &'a str)>, SseError> {
        // Comment lines start with :
        if line.starts_with(':') {
            return Ok(None);
        }
        
        // Find colon separator
        if let Some(colon_pos) = line.find(':') {
            let field = &line[..colon_pos];
            let mut value = &line[colon_pos + 1..];
            
            // Remove optional space after colon
            if value.starts_with(' ') {
                value = &value[1..];
            }
            
            Ok(Some((field, value)))
        } else {
            // Field with no value
            Ok(Some((line, "")))
        }
    }
    
    pub fn get_last_event_id(&self) -> Option<&str> {
        self.last_event_id.as_deref()
    }
}
```

### Step 3: Integrate with HTTP Transport
Update the HTTP transport to use the parser:

```rust
// In shadowcat/src/transport/http.rs
use crate::transport::sse::parser::{SseParser, SseEvent};

impl HttpTransport {
    async fn process_sse_stream(
        &mut self,
        mut event_source: EventSource,
        response_tx: mpsc::Sender<TransportMessage>,
    ) {
        let mut parser = SseParser::new();
        
        // Restore from last event ID if reconnecting
        if let Some(last_id) = &self.last_event_id {
            debug!("Resuming SSE from event ID: {}", last_id);
        }
        
        while let Some(event_result) = event_source.next().await {
            match event_result {
                Ok(Event::Message(msg)) => {
                    // Parse SSE message
                    if let Ok(Some(sse_event)) = parser.parse_line(&msg.data) {
                        self.handle_sse_event(sse_event, &response_tx).await;
                    }
                }
                Ok(Event::Open) => {
                    info!("SSE connection established");
                    
                    // Send Last-Event-ID if we have one
                    if let Some(last_id) = parser.get_last_event_id() {
                        self.last_event_id = Some(last_id.to_string());
                    }
                }
                Err(e) => {
                    warn!("SSE error: {}", e);
                    self.handle_sse_error(e).await;
                }
            }
        }
    }
    
    async fn handle_sse_event(
        &mut self,
        event: SseEvent,
        tx: &mpsc::Sender<TransportMessage>,
    ) {
        // Store event ID for resumption
        if let Some(ref id) = event.id {
            self.last_event_id = Some(id.clone());
            trace!("Received SSE event with ID: {}", id);
        }
        
        // Handle retry directive
        if let Some(retry) = event.retry {
            self.reconnect_delay = retry;
            debug!("SSE server requested retry delay: {:?}", retry);
        }
        
        // Process MCP messages
        if event.is_mcp_message() {
            match event.parse_json_data() {
                Ok(json_value) => {
                    if let Ok(transport_msg) = Self::json_to_transport_message(&json_value) {
                        if let Err(e) = tx.send(transport_msg).await {
                            warn!("Failed to send SSE message: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse SSE data as JSON: {}", e);
                }
            }
        } else {
            // Handle custom event types
            debug!("Received custom SSE event type: {:?}", event.event_type);
        }
    }
}
```

### Step 4: Add Stream Processing
Handle SSE stream properly:

```rust
// In shadowcat/src/transport/sse/stream.rs
use futures_util::StreamExt;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct SseStream {
    reader: BufReader<Box<dyn AsyncRead + Send + Unpin>>,
    parser: SseParser,
}

impl SseStream {
    pub fn new<R>(reader: R) -> Self 
    where
        R: AsyncRead + Send + Unpin + 'static,
    {
        Self {
            reader: BufReader::new(Box::new(reader)),
            parser: SseParser::new(),
        }
    }
    
    pub async fn next_event(&mut self) -> Result<SseEvent, SseError> {
        let mut lines = String::new();
        
        loop {
            lines.clear();
            match self.reader.read_line(&mut lines).await {
                Ok(0) => return Err(SseError::ConnectionClosed),
                Ok(_) => {
                    let line = lines.trim_end_matches('\n').trim_end_matches('\r');
                    
                    if let Some(event) = self.parser.parse_line(line)? {
                        return Ok(event);
                    }
                }
                Err(e) => return Err(SseError::Io(e)),
            }
        }
    }
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_message() {
        let mut parser = SseParser::new();
        
        assert!(parser.parse_line("data: {\"test\":1}").unwrap().is_none());
        let event = parser.parse_line("").unwrap().unwrap();
        
        assert_eq!(event.data, vec!["{\"test\":1}"]);
        assert!(event.event_type.is_none());
    }
    
    #[test]
    fn test_parse_multi_line_data() {
        let mut parser = SseParser::new();
        
        parser.parse_line("data: {").unwrap();
        parser.parse_line("data:   \"test\": 1").unwrap();
        parser.parse_line("data: }").unwrap();
        
        let event = parser.parse_line("").unwrap().unwrap();
        assert_eq!(event.data.len(), 3);
        
        let json = event.parse_json_data().unwrap();
        assert_eq!(json["test"], 1);
    }
    
    #[test]
    fn test_parse_with_event_id() {
        let mut parser = SseParser::new();
        
        parser.parse_line("id: msg-123").unwrap();
        parser.parse_line("event: message").unwrap();
        parser.parse_line("data: test").unwrap();
        
        let event = parser.parse_line("").unwrap().unwrap();
        assert_eq!(event.id, Some("msg-123".to_string()));
        assert_eq!(event.event_type, Some("message".to_string()));
        assert_eq!(parser.get_last_event_id(), Some("msg-123"));
    }
    
    #[test]
    fn test_parse_retry_directive() {
        let mut parser = SseParser::new();
        
        parser.parse_line("retry: 5000").unwrap();
        parser.parse_line("data: test").unwrap();
        
        let event = parser.parse_line("").unwrap().unwrap();
        assert_eq!(event.retry, Some(Duration::from_millis(5000)));
    }
    
    #[test]
    fn test_ignore_comments() {
        let mut parser = SseParser::new();
        
        parser.parse_line(": this is a comment").unwrap();
        parser.parse_line("data: actual data").unwrap();
        
        let event = parser.parse_line("").unwrap().unwrap();
        assert_eq!(event.data, vec!["actual data"]);
    }
}
```

## Files to Create/Modify

1. **Create**: `shadowcat/src/transport/sse/mod.rs` - SSE module
2. **Create**: `shadowcat/src/transport/sse/parser.rs` - Parser implementation
3. **Create**: `shadowcat/src/transport/sse/stream.rs` - Stream processing
4. **Modify**: `shadowcat/src/transport/http.rs:148-216` - Use new parser
5. **Create**: `shadowcat/tests/sse_parser_test.rs` - Integration tests

## Acceptance Criteria

- [ ] Parse all SSE field types (data, event, id, retry)
- [ ] Handle multi-line data fields
- [ ] Track last event ID for resumption
- [ ] Parse JSON from combined data lines
- [ ] Handle retry directives
- [ ] Ignore comments and unknown fields
- [ ] All tests passing
- [ ] No clippy warnings

## Performance Considerations

- Use streaming parser to avoid buffering entire events
- Minimize string allocations with careful lifetime management
- Consider using `bytes::Bytes` for zero-copy parsing

## Notes for Next Session

- Task 1.2 will build on this parser for connection management
- Task 1.3 will add reconnection using Last-Event-ID
- Consider adding metrics for SSE event types and sizes

## References

- SSE Specification: https://html.spec.whatwg.org/multipage/server-sent-events.html
- MCP HTTP Transport: `/specs/mcp/docs/specification/2025-06-18/basic/transports.mdx`
- Current Implementation: `shadowcat/src/transport/http.rs:148-216`