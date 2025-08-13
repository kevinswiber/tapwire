# MCP Message Handling Tracker

## Overview

This tracker coordinates the implementation of MCP-aware message handling across Shadowcat's interceptor, recorder, and replay systems. The goal is to enable intelligent processing at the JSON-RPC message level rather than raw transport frames.

**Last Updated**: 2025-08-08  
**Dependencies**: SSE Proxy Integration (in progress)  
**Estimated Duration**: 60-80 hours  
**Priority**: HIGH

## Executive Summary

Currently, Shadowcat's interceptor, recorder, and replay systems operate at the transport frame level. To provide meaningful MCP developer tools, we need to elevate these systems to understand and manipulate JSON-RPC messages, handle request-response correlation, and maintain session context.

### Current State
- ✅ **Transport-level interception**: Can intercept raw frames
- ✅ **Binary recording**: Can record transport bytes
- ✅ **Basic replay**: Can replay recorded bytes
- ❌ **MCP message parsing**: No JSON-RPC understanding
- ❌ **Request-response correlation**: No ID matching
- ❌ **Session-aware recording**: No MCP session context
- ❌ **Semantic interception**: No method-based rules

### Goals
1. Parse and understand MCP JSON-RPC messages
2. Correlate requests with responses using JSON-RPC IDs
3. Enable method-based interception rules
4. Record MCP sessions with full context
5. Support intelligent replay with message modification

## Architecture Overview

### Layered Message Processing

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│                   (MCP Business Logic)                      │
├─────────────────────────────────────────────────────────────┤
│                  MCP Message Layer (NEW)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ JSON-RPC     │  │   Message    │  │   Session    │    │
│  │   Parser     │  │  Correlator  │  │   Context    │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                 Interceptor Layer (ENHANCED)                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   Method     │  │   Param      │  │   Result     │    │
│  │   Matcher    │  │  Inspector   │  │  Modifier    │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                  Recorder Layer (ENHANCED)                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   Message    │  │   Session    │  │    Tape      │    │
│  │   Storage    │  │   Timeline   │  │   Format     │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                    Transport Layer                          │
│              (stdio, HTTP, SSE, WebSocket)                  │
└─────────────────────────────────────────────────────────────┘
```

## Phase 1: MCP Message Parser

### Goal
Create a robust JSON-RPC message parser that can handle all MCP message types, including batches.

### Tasks

#### Task 1.1: Core Message Types
**Duration**: 4 hours  
**Files**: `src/mcp/message.rs` (new)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpMessage {
    Single(JsonRpcMessage),
    Batch(Vec<JsonRpcMessage>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "jsonrpc")]
pub enum JsonRpcMessage {
    #[serde(rename = "2.0")]
    V2(JsonRpcV2Message),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcV2Message {
    Request {
        id: JsonRpcId,
        method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Value>,
    },
    Response {
        id: JsonRpcId,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<JsonRpcError>,
    },
    Notification {
        method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    Number(i64),
    String(String),
}
```

#### Task 1.2: Message Parser
**Duration**: 3 hours  
**Files**: `src/mcp/parser.rs` (new)

- Parse raw bytes/strings into McpMessage
- Validate JSON-RPC structure
- Handle malformed messages gracefully
- Support streaming parsing for SSE

#### Task 1.3: Message Builder
**Duration**: 2 hours  
**Files**: `src/mcp/builder.rs` (new)

- Fluent API for constructing messages
- Automatic ID generation
- Batch message construction
- Error response helpers

## Phase 2: Message Correlation

### Goal
Track and correlate requests with their responses across async boundaries.

### Tasks

#### Task 2.1: Correlation Engine
**Duration**: 5 hours  
**Files**: `src/mcp/correlation.rs` (new)

```rust
pub struct MessageCorrelator {
    pending_requests: HashMap<JsonRpcId, PendingRequest>,
    request_timeout: Duration,
    orphaned_responses: HashMap<JsonRpcId, OrphanedResponse>,
}

struct PendingRequest {
    request: JsonRpcV2Message,
    sent_at: Instant,
    session_id: SessionId,
    correlation_id: Uuid,
    timeout_handle: JoinHandle<()>,
}

struct OrphanedResponse {
    response: JsonRpcV2Message,
    received_at: Instant,
    session_id: SessionId,
}
```

#### Task 2.2: Request-Response Matching
**Duration**: 3 hours  
**Files**: `src/mcp/correlation.rs`

- Match responses to requests by ID
- Handle out-of-order responses
- Track orphaned responses
- Timeout handling for missing responses

#### Task 2.3: Correlation Metrics
**Duration**: 2 hours  
**Files**: `src/mcp/metrics.rs` (new)

- Response time tracking
- Success/error rates per method
- Timeout statistics
- Orphaned response counts

## Phase 3: MCP-Aware Interceptor

### Goal
Enable interception based on MCP message content rather than raw transport data.

### Tasks

#### Task 3.1: Message Interceptor Interface
**Duration**: 4 hours  
**Files**: `src/interceptor/mcp.rs` (new)

```rust
#[async_trait]
pub trait McpInterceptor: Send + Sync {
    async fn intercept_request(
        &self,
        ctx: &mut InterceptContext,
        request: &JsonRpcV2Message,
    ) -> InterceptDecision;
    
    async fn intercept_response(
        &self,
        ctx: &mut InterceptContext,
        response: &JsonRpcV2Message,
        request: Option<&JsonRpcV2Message>,
    ) -> InterceptDecision;
    
    async fn intercept_notification(
        &self,
        ctx: &mut InterceptContext,
        notification: &JsonRpcV2Message,
    ) -> InterceptDecision;
}

pub enum InterceptDecision {
    Allow,
    Block(JsonRpcError),
    Modify(JsonRpcV2Message),
    Delay(Duration),
    Fork(Vec<JsonRpcV2Message>),
}
```

#### Task 3.2: Method-Based Rules
**Duration**: 5 hours  
**Files**: `src/interceptor/rules/mcp.rs` (new)

```rust
pub struct McpRule {
    pub id: RuleId,
    pub name: String,
    pub condition: McpCondition,
    pub action: McpAction,
    pub priority: i32,
}

pub enum McpCondition {
    MethodEquals(String),
    MethodMatches(Regex),
    MethodIn(Vec<String>),
    ParamContains(String, Value),
    ResultContains(String, Value),
    ErrorCodeEquals(i32),
    And(Box<McpCondition>, Box<McpCondition>),
    Or(Box<McpCondition>, Box<McpCondition>),
    Not(Box<McpCondition>),
}

pub enum McpAction {
    Allow,
    Block,
    ModifyParams(Value),
    ModifyResult(Value),
    InjectError(JsonRpcError),
    Delay(Duration),
    Log(LogLevel),
    Webhook(String),
}
```

#### Task 3.3: Interceptor Chain
**Duration**: 3 hours  
**Files**: `src/interceptor/chain.rs` (modified)

- Integrate MCP interceptors with existing chain
- Preserve message ordering
- Handle batch messages
- Async interceptor execution

## Phase 4: MCP-Aware Recorder

### Goal
Record MCP sessions with full message context and correlation.

### Tasks

#### Task 4.1: MCP Tape Format
**Duration**: 4 hours  
**Files**: `src/recorder/mcp_tape.rs` (new)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct McpTape {
    pub id: TapeId,
    pub session_id: SessionId,
    pub mcp_session_id: Option<String>,
    pub protocol_version: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub messages: Vec<McpTapeEntry>,
    pub metadata: TapeMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpTapeEntry {
    pub timestamp: DateTime<Utc>,
    pub direction: MessageDirection,
    pub message: McpMessage,
    pub correlation_id: Option<Uuid>,
    pub transport: TransportInfo,
    pub intercepted: bool,
    pub modified: Option<McpMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TapeMetadata {
    pub total_requests: usize,
    pub total_responses: usize,
    pub total_notifications: usize,
    pub methods_called: HashMap<String, usize>,
    pub error_count: usize,
    pub average_response_time: Duration,
}
```

#### Task 4.2: Session Recorder
**Duration**: 5 hours  
**Files**: `src/recorder/mcp_recorder.rs` (new)

- Record all messages in a session
- Maintain request-response correlation
- Calculate session statistics
- Support filtering and sampling

#### Task 4.3: Storage Backend
**Duration**: 3 hours  
**Files**: `src/recorder/storage/mcp.rs` (new)

- SQLite schema for MCP messages
- Indexed by session, method, timestamp
- Full-text search on message content
- Compression for large messages

## Phase 5: MCP-Aware Replay

### Goal
Enable intelligent replay of recorded MCP sessions with modification capabilities.

### Tasks

#### Task 5.1: Replay Engine
**Duration**: 5 hours  
**Files**: `src/replay/mcp_engine.rs` (new)

```rust
pub struct McpReplayEngine {
    tape: McpTape,
    replay_config: ReplayConfig,
    message_transformer: Box<dyn MessageTransformer>,
    timing_controller: TimingController,
}

pub struct ReplayConfig {
    pub speed_multiplier: f64,
    pub skip_notifications: bool,
    pub filter_methods: Option<Vec<String>>,
    pub transform_ids: bool,
    pub update_timestamps: bool,
    pub error_injection: Option<ErrorInjectionConfig>,
}

#[async_trait]
pub trait MessageTransformer: Send + Sync {
    async fn transform_request(&self, request: &mut JsonRpcV2Message);
    async fn transform_response(&self, response: &mut JsonRpcV2Message);
}
```

#### Task 5.2: Replay Controller
**Duration**: 4 hours  
**Files**: `src/replay/controller.rs` (new)

- Play/pause/stop/seek functionality
- Variable speed playback
- Breakpoint support
- Step-by-step execution

#### Task 5.3: Replay Modifications
**Duration**: 3 hours  
**Files**: `src/replay/transform.rs` (new)

- ID regeneration for new sessions
- Parameter value substitution
- Response result modification
- Error injection for testing

## Phase 6: Integration

### Goal
Integrate MCP-aware components with existing proxy infrastructure.

### Tasks

#### Task 6.1: Forward Proxy Integration
**Duration**: 4 hours  
**Files**: `src/proxy/forward.rs` (modified)

- Parse messages from transport
- Apply MCP interceptors
- Record MCP messages
- Maintain correlation

#### Task 6.2: Reverse Proxy Integration
**Duration**: 4 hours  
**Files**: `src/proxy/reverse.rs` (modified)

- Parse incoming MCP messages
- Apply server-side interceptors
- Record server interactions
- Generate MCP-aware responses

#### Task 6.3: Transport Abstraction
**Duration**: 3 hours  
**Files**: `src/transport/mcp_aware.rs` (new)

- Wrap existing transports with MCP parsing
- Handle transport-specific framing
- Support batch message splitting/combining

## Implementation Guidelines

### Design Principles
1. **Separation of Concerns**: MCP logic separate from transport
2. **Zero-Copy Where Possible**: Minimize message copying
3. **Async Throughout**: No blocking operations
4. **Graceful Degradation**: Fall back to transport-level on parse errors
5. **Extensibility**: Plugin architecture for custom interceptors

### Performance Requirements
- < 1ms overhead for message parsing
- < 5ms overhead for interception chain
- Support 10,000+ messages/second
- < 100MB memory for 10,000 message correlation

### Testing Strategy
1. **Unit Tests**: Each component in isolation
2. **Integration Tests**: Full message flow
3. **Performance Tests**: Benchmark overhead
4. **Conformance Tests**: MCP specification compliance
5. **Fuzzing**: Malformed message handling

## Success Criteria

### Functional Requirements
- [ ] Parse all valid MCP JSON-RPC messages
- [ ] Correlate 100% of request-response pairs
- [ ] Support method-based interception rules
- [ ] Record complete MCP sessions
- [ ] Replay sessions with modifications

### Non-Functional Requirements
- [ ] < 5% performance overhead
- [ ] Handle malformed messages gracefully
- [ ] Support both MCP versions (2025-03-26, 2025-06-18)
- [ ] Maintain message ordering
- [ ] Thread-safe concurrent access

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Performance overhead | HIGH | Careful optimization, caching |
| Message correlation complexity | HIGH | Timeout-based cleanup, orphan handling |
| Storage growth | MEDIUM | Compression, rotation, sampling |
| Backwards compatibility | MEDIUM | Version detection, graceful fallback |

## Dependencies

- SSE Proxy Integration (in progress) - See [Integration Coordination](../integration-coordination.md)
- Existing interceptor framework
- Existing recorder infrastructure
- Transport abstraction layer

## Coordination with SSE Integration

Key synergies identified in [Integration Coordination Guide](../integration-coordination.md):
- Shared MCP parser for immediate SSE message understanding
- Unified batch handling for MCP 2025-03-26
- Common event ID generation for correlation
- Session context sharing between SSE and MCP layers

## Timeline Estimate

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: Parser | 9 hours | None |
| Phase 2: Correlation | 10 hours | Phase 1 |
| Phase 3: Interceptor | 12 hours | Phase 1, 2 |
| Phase 4: Recorder | 12 hours | Phase 1, 2 |
| Phase 5: Replay | 12 hours | Phase 4 |
| Phase 6: Integration | 11 hours | All phases |
| Testing & Documentation | 14 hours | All phases |
| **Total** | **80 hours** | |

## Next Steps

1. Complete SSE proxy integration
2. Implement Phase 1 (Parser) as foundation
3. Build correlation engine for request-response tracking
4. Enhance interceptor with MCP awareness
5. Upgrade recorder and replay systems

## References

- [MCP Specification 2025-06-18](../../specs/mcp/docs/specification/2025-06-18/)
- [MCP Specification 2025-03-26](../../specs/mcp/docs/specification/2025-03-26/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [SSE Proxy Integration Tracker](../sse-proxy-integration/sse-proxy-integration-tracker.md)