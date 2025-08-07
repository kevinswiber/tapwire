# Phase 1 - Task 1.1: SSE Event Parser Implementation

## Task Overview
Implement a robust Server-Sent Events (SSE) parser for the MCP Streamable HTTP transport that can handle all standard SSE formats, edge cases, and protocol requirements.

**Duration**: 3-4 hours
**Priority**: CRITICAL - Foundation for all SSE functionality
**Dependencies**: Phase 0 complete (version management infrastructure)

## Objectives

### Primary Goals
1. Create a streaming SSE parser that processes incoming byte streams
2. Handle all SSE field types: `data:`, `event:`, `id:`, `retry:`
3. Support multi-line data fields with proper concatenation
4. Implement event emission when double newline is encountered
5. Handle edge cases (comments, malformed data, partial messages)

### Success Criteria
- [ ] Parser correctly processes standard SSE format
- [ ] Multi-line data fields are properly concatenated with newlines
- [ ] Custom event types are preserved
- [ ] Event IDs are tracked for resumability
- [ ] Retry intervals are parsed and stored
- [ ] Comments (lines starting with `:`) are ignored
- [ ] Partial messages are buffered until complete
- [ ] Zero-copy parsing where possible for performance
- [ ] Comprehensive unit tests with 100% coverage of SSE spec

## Technical Requirements

### SSE Format Specification
According to the [SSE standard](https://html.spec.whatwg.org/multipage/server-sent-events.html):

1. **Field Format**: `field: value\n`
2. **Supported Fields**:
   - `data:` - The message payload (can span multiple lines)
   - `event:` - Custom event type (default: "message")
   - `id:` - Event ID for resumability
   - `retry:` - Reconnection time in milliseconds
3. **Special Cases**:
   - Empty line (`\n\n`) triggers event dispatch
   - Lines starting with `:` are comments
   - Field without `:` is treated as field name with empty value
   - BOM at stream start should be ignored

### MCP-Specific Requirements
From the MCP Streamable HTTP specification:

1. **Event IDs**: Must be globally unique within a session
2. **Resumability**: Support `Last-Event-ID` header for reconnection
3. **JSON-RPC Messages**: Data fields contain JSON-RPC payloads
4. **Session Context**: Events may need session ID association
5. **Multiple Streams**: Parser instances must be independent

## Implementation Plan

### Module Structure
```
src/transport/sse/
├── mod.rs           # Module exports and public API
├── parser.rs        # Core SSE parsing logic
├── event.rs         # SSE event types and structures
├── buffer.rs        # Buffering and stream handling
└── tests/
    ├── mod.rs       # Test module
    ├── parser.rs    # Parser unit tests
    └── fixtures.rs  # SSE test fixtures
```

### Core Components

#### 1. Event Structure (`event.rs`)
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct SseEvent {
    pub id: Option<String>,
    pub event_type: String,  // Default: "message"
    pub data: String,
    pub retry: Option<u64>,
}

#[derive(Debug)]
pub enum SseField {
    Data(String),
    Event(String),
    Id(String),
    Retry(u64),
    Comment(String),
}
```

#### 2. Parser State Machine (`parser.rs`)
```rust
pub struct SseParser {
    buffer: Vec<u8>,
    current_event: EventBuilder,
    position: usize,
}

impl SseParser {
    pub fn new() -> Self;
    pub fn feed(&mut self, data: &[u8]) -> Vec<SseEvent>;
    fn parse_line(&mut self, line: &str) -> Option<SseField>;
    fn dispatch_event(&mut self) -> Option<SseEvent>;
}
```

#### 3. Stream Adapter (`buffer.rs`)
```rust
pub struct SseStream<S> {
    stream: S,
    parser: SseParser,
}

impl<S: AsyncRead> Stream for SseStream<S> {
    type Item = Result<SseEvent, SseError>;
    // Stream implementation
}
```

### Parsing Algorithm

1. **Input Processing**:
   - Accept byte chunks from network
   - Buffer partial lines
   - Split on newline boundaries

2. **Line Parsing**:
   ```
   FOR each line:
     IF line is empty:
       Dispatch current event
     ELSE IF line starts with ':':
       Ignore (comment)
     ELSE IF line contains ':':
       Split on first ':'
       Process field and value
     ELSE:
       Treat as field with empty value
   ```

3. **Field Processing**:
   - `data:` - Append to event data (with newline if not first)
   - `event:` - Set event type
   - `id:` - Set event ID
   - `retry:` - Parse as integer, set retry interval

4. **Event Dispatch**:
   - Triggered by empty line
   - Reset event builder
   - Emit if data is non-empty

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum SseError {
    #[error("Invalid UTF-8 in SSE stream")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    
    #[error("Invalid retry value: {0}")]
    InvalidRetry(String),
    
    #[error("Stream terminated unexpectedly")]
    UnexpectedEof,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Test Cases

### Unit Tests

1. **Basic Parsing**:
   ```
   data: hello\n\n
   → Event { data: "hello", event_type: "message" }
   ```

2. **Multi-line Data**:
   ```
   data: first line\n
   data: second line\n\n
   → Event { data: "first line\nsecond line" }
   ```

3. **Custom Event Type**:
   ```
   event: custom\n
   data: payload\n\n
   → Event { event_type: "custom", data: "payload" }
   ```

4. **Event ID and Retry**:
   ```
   id: 123\n
   retry: 5000\n
   data: test\n\n
   → Event { id: Some("123"), retry: Some(5000), data: "test" }
   ```

5. **Comments**:
   ```
   : this is a comment\n
   data: actual data\n\n
   → Event { data: "actual data" }
   ```

6. **Edge Cases**:
   - Field without colon
   - Field with empty value
   - Multiple colons in value
   - Partial message buffering
   - Invalid UTF-8 handling
   - BOM handling

### Integration Tests

1. **Streaming Parse**: Feed data in chunks, verify correct assembly
2. **Large Messages**: Parse multi-megabyte events
3. **Rapid Events**: High-frequency event parsing
4. **Malformed Input**: Graceful error recovery

## Performance Considerations

1. **Zero-Copy**: Use string slices where possible
2. **Buffering**: Efficient buffer management with growth strategy
3. **Allocation**: Pre-allocate for common event sizes
4. **Streaming**: Process data as it arrives, don't wait for complete messages

## Dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["io-util"] }
bytes = "1"
thiserror = "2"
tracing = "0.1"
futures = "0.3"

[dev-dependencies]
tokio = { version = "1", features = ["rt", "macros", "test-util"] }
pretty_assertions = "1"
```

## Files to Modify

1. **Create new files**:
   - `src/transport/sse/mod.rs`
   - `src/transport/sse/parser.rs`
   - `src/transport/sse/event.rs`
   - `src/transport/sse/buffer.rs`
   - `src/transport/sse/tests/mod.rs`
   - `src/transport/sse/tests/parser.rs`
   - `src/transport/sse/tests/fixtures.rs`

2. **Update existing**:
   - `src/transport/mod.rs` - Export SSE module
   - `Cargo.toml` - Add bytes dependency if not present

## Verification Steps

1. **Unit Tests**:
   ```bash
   cargo test sse::parser
   ```

2. **Doc Tests**:
   ```bash
   cargo test --doc sse
   ```

3. **Benchmarks** (optional):
   ```bash
   cargo bench sse_parser
   ```

4. **Example Usage**:
   ```rust
   let mut parser = SseParser::new();
   let events = parser.feed(b"data: hello\n\n");
   assert_eq!(events[0].data, "hello");
   ```

## Integration Points

This parser will integrate with:
1. **HTTP Transport**: Parse SSE responses from POST/GET
2. **Session Manager**: Associate events with sessions
3. **Interceptor**: Allow event interception/modification
4. **Recorder**: Capture SSE events for replay

## Next Steps

After completing this task:
1. Task 1.2: SSE Connection Management - Use parser in HTTP client
2. Task 1.3: SSE Reconnection - Implement retry and resumption
3. Task 1.4: SSE Session Integration - Link with session management
4. Task 1.5: SSE Performance - Optimize and benchmark

## Notes

- The parser must be reusable across multiple connections
- Consider implementing as a tokio codec for efficiency
- Ensure thread-safety for concurrent usage
- Follow Rust idioms and error handling patterns
- Document public API with examples