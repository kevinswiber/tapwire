# Data Flow Architecture

## Request Flow Overview

```mermaid
graph TD
    Client[Client] -->|HTTP/SSE| Router[Router]
    Router --> MW[Middleware Chain]
    MW --> Auth[Auth Middleware]
    Auth --> RL[Rate Limiter]
    RL --> Handler[Request Handler]
    Handler --> SessionMgr[Session Manager]
    SessionMgr --> Transport[Transport Router]
    Transport --> Upstream[Upstream Server]
    
    Upstream -->|Response| Interceptor[Response Interceptor]
    Interceptor --> Recorder[Tape Recorder]
    Recorder --> Response[Response Builder]
    Response -->|HTTP/SSE| Client
```

## Detailed Request Processing

### 1. Initial Request Reception
```mermaid
sequenceDiagram
    participant Client
    participant Router
    participant Middleware
    participant Handler
    
    Client->>Router: HTTP Request
    Router->>Router: Extract path/method
    Router->>Middleware: Pre-process request
    Middleware->>Middleware: Auth check
    Middleware->>Middleware: Rate limit check
    Middleware->>Handler: Authorized request
```

### 2. Session Management Flow
```mermaid
sequenceDiagram
    participant Handler
    participant SessionMgr
    participant Database
    participant Cache
    
    Handler->>SessionMgr: get_or_create(session_id)
    SessionMgr->>Cache: Check cache
    alt Session in cache
        Cache-->>SessionMgr: Return cached session
    else Session not cached
        SessionMgr->>Database: Query session
        alt Session exists
            Database-->>SessionMgr: Session data
            SessionMgr->>Cache: Store in cache
        else New session
            SessionMgr->>Database: Create session
            SessionMgr->>Cache: Cache new session
        end
    end
    SessionMgr-->>Handler: Session object
```

### 3. Transport Selection and Processing
```mermaid
sequenceDiagram
    participant Handler
    participant TransportRouter
    participant Pool
    participant StdioTransport
    participant HttpTransport
    
    Handler->>TransportRouter: route(message, upstream_config)
    TransportRouter->>TransportRouter: Determine transport type
    
    alt Stdio Transport
        TransportRouter->>Pool: acquire_connection()
        Pool-->>TransportRouter: Pooled connection
        TransportRouter->>StdioTransport: process(message)
        StdioTransport-->>TransportRouter: Response
        TransportRouter->>Pool: release_connection()
    else HTTP Transport
        TransportRouter->>HttpTransport: process(message)
        HttpTransport-->>TransportRouter: Response
    end
    
    TransportRouter-->>Handler: Processed response
```

### 4. SSE Streaming Flow
```mermaid
graph TD
    Client[Client] -->|SSE Request| SseHandler[SSE Handler]
    SseHandler --> Session[Get/Create Session]
    Session --> InitCheck{Initialized?}
    
    InitCheck -->|No| Initialize[Send Initialize]
    InitCheck -->|Yes| Stream[Start Stream]
    Initialize --> Stream
    
    Stream --> EventLoop[Event Loop]
    EventLoop --> ReadUpstream[Read from Upstream]
    ReadUpstream --> Transform[Transform Event]
    Transform --> SendClient[Send to Client]
    SendClient --> EventLoop
    
    EventLoop -->|Error/Close| Cleanup[Cleanup Session]
```

## State Management

### Application State Structure
```mermaid
graph TD
    AppState[AppState]
    AppState --> Config[Config - Immutable]
    AppState --> SessionMgr[SessionManager - Arc]
    AppState --> Metrics[Metrics - Arc<Atomic>]
    AppState --> Pools[Connection Pools - Arc]
    AppState --> Auth[AuthGateway - Optional Arc]
    AppState --> RateLimit[RateLimiter - Optional Arc]
    AppState --> Interceptor[InterceptorChain - Arc]
    AppState --> Recorder[TapeRecorder - Optional Arc]
```

### Session State Lifecycle
```mermaid
stateDiagram-v2
    [*] --> New: Client connects
    New --> Initializing: First request
    Initializing --> Active: Initialize complete
    Active --> Active: Process messages
    Active --> Paused: Pause command
    Paused --> Active: Resume command
    Active --> Closing: Client disconnect
    Closing --> Closed: Cleanup complete
    Closed --> [*]
    
    Active --> Error: Processing error
    Error --> Closing: Unrecoverable
    Error --> Active: Recovered
```

### Per-Request Context
```rust
struct RequestContext {
    // Immutable request data
    request_id: Uuid,
    method: Method,
    path: String,
    headers: HeaderMap,
    
    // Session reference
    session: Arc<Session>,
    
    // Shared app state
    state: Arc<AppState>,
    
    // Request-specific data
    start_time: Instant,
    auth_context: Option<AuthContext>,
    span: tracing::Span,
}
```

## Error Propagation

### Error Flow Paths
```mermaid
graph TD
    Origin[Error Origin] --> Type{Error Type}
    
    Type -->|Transport| TransportError[Transport Error]
    Type -->|Protocol| ProtocolError[Protocol Error]
    Type -->|Auth| AuthError[Auth Error]
    Type -->|Validation| ValidationError[Validation Error]
    
    TransportError --> Recovery{Recoverable?}
    ProtocolError --> Recovery
    AuthError --> ClientResponse[Client Error Response]
    ValidationError --> ClientResponse
    
    Recovery -->|Yes| Retry[Retry Logic]
    Recovery -->|No| ClientResponse
    
    Retry -->|Success| Continue[Continue Processing]
    Retry -->|Failed| ClientResponse
    
    ClientResponse --> Log[Log Error]
    Log --> Metrics[Update Metrics]
    Metrics --> Response[Send Response]
```

### Error Response Generation
```rust
// Error is caught at handler boundary
match process_request(req).await {
    Ok(response) => response,
    Err(e) => {
        // Log error with context
        error!(error = ?e, session_id = ?session_id, "Request failed");
        
        // Update metrics
        metrics.record_error(&e);
        
        // Generate appropriate client response
        match e {
            ReverseProxyError::Auth(_) => {
                Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .json(error_response("Authentication required"))
            }
            ReverseProxyError::RateLimit(_) => {
                Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .header("Retry-After", "60")
                    .json(error_response("Rate limit exceeded"))
            }
            _ => {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(error_response("Internal server error"))
            }
        }
    }
}
```

## Message Interception Points

### Request Interception
```mermaid
graph LR
    Request[Incoming Request] --> PreInt[Pre-Interceptor]
    PreInt --> Check1{Block?}
    Check1 -->|Yes| BlockResp[Block Response]
    Check1 -->|No| Modify[Modify Request]
    Modify --> Process[Process Request]
    Process --> PostInt[Post-Interceptor]
    PostInt --> Check2{Modify?}
    Check2 -->|Yes| ModifyResp[Modify Response]
    Check2 -->|No| Response[Send Response]
    ModifyResp --> Response
```

### Interception Context
```rust
struct InterceptContext {
    session_id: SessionId,
    direction: Direction,
    transport_type: TransportType,
    message_type: MessageType,
    timestamp: Instant,
    metadata: HashMap<String, Value>,
}

enum InterceptAction {
    Continue,           // Continue processing
    Modify(Value),      // Replace with modified message
    Block(Response),    // Block with response
    Pause,             // Pause processing
}
```

## Connection Pooling

### Pool Management Flow
```mermaid
graph TD
    Request[Request] --> NeedConn[Need Connection]
    NeedConn --> CheckPool{Pool Available?}
    
    CheckPool -->|Yes| Acquire[Acquire from Pool]
    CheckPool -->|No| Create[Create New]
    
    Acquire --> Validate{Still Valid?}
    Validate -->|Yes| Use[Use Connection]
    Validate -->|No| Remove[Remove from Pool]
    Remove --> Create
    
    Create --> Use
    Use --> Complete[Request Complete]
    Complete --> ReturnPool{Return to Pool?}
    
    ReturnPool -->|Healthy| Return[Return to Pool]
    ReturnPool -->|Unhealthy| Discard[Discard Connection]
```

## Metrics Collection Points

### Metric Collection Flow
```mermaid
graph TD
    Start[Request Start] --> Timer[Start Timer]
    Timer --> Process[Process Request]
    
    Process --> Success{Success?}
    Success -->|Yes| RecordSuccess[Record Success Metrics]
    Success -->|No| RecordError[Record Error Metrics]
    
    RecordSuccess --> RecordTime[Record Duration]
    RecordError --> RecordTime
    RecordTime --> RecordSize[Record Response Size]
    RecordSize --> UpdateCounters[Update Counters]
    
    UpdateCounters --> Aggregate[Aggregate Stats]
    Aggregate --> Export[Export to Monitoring]
```

### Metrics Data Structure
```rust
struct MetricsData {
    // Counters
    requests_total: AtomicU64,
    requests_success: AtomicU64,
    requests_failed: AtomicU64,
    
    // Gauges
    active_sessions: AtomicU64,
    pool_connections: AtomicU64,
    
    // Histograms (protected by mutex)
    response_times: Mutex<Histogram>,
    response_sizes: Mutex<Histogram>,
    
    // Per-endpoint metrics
    endpoint_metrics: DashMap<String, EndpointMetrics>,
}
```

## Recording/Replay Flow

### Recording Process
```mermaid
sequenceDiagram
    participant Handler
    participant Recorder
    participant Storage
    
    Handler->>Recorder: record_request(envelope)
    Recorder->>Recorder: Serialize message
    Recorder->>Recorder: Add timestamp
    Recorder->>Storage: Write to tape
    
    Handler->>Handler: Process request
    
    Handler->>Recorder: record_response(envelope)
    Recorder->>Recorder: Serialize response
    Recorder->>Recorder: Calculate delta time
    Recorder->>Storage: Append to tape
```

### Replay Process
```mermaid
sequenceDiagram
    participant Player
    participant Tape
    participant Client
    
    Player->>Tape: Open tape file
    Player->>Tape: Read next message
    
    loop For each message
        Tape-->>Player: Message + timing
        Player->>Player: Wait for timing
        Player->>Client: Send message
        Client-->>Player: Response
        Player->>Player: Validate response
    end
    
    Player->>Player: Generate report
```

## Shutdown Sequence

### Graceful Shutdown Flow
```mermaid
graph TD
    Signal[Shutdown Signal] --> Notify[Notify Components]
    Notify --> StopAccept[Stop Accepting Requests]
    StopAccept --> WaitActive[Wait for Active Requests]
    
    WaitActive --> Timeout{Timeout?}
    Timeout -->|No| Complete[Requests Complete]
    Timeout -->|Yes| Force[Force Terminate]
    
    Complete --> CloseConns[Close Connections]
    Force --> CloseConns
    
    CloseConns --> FlushRec[Flush Recordings]
    FlushRec --> SaveState[Save Session State]
    SaveState --> CloseDB[Close Database]
    CloseDB --> Exit[Exit Process]
```

## Performance Considerations

### Critical Path Optimization
1. **Session Cache**: In-memory LRU cache reduces database queries
2. **Connection Pooling**: Reuse connections for stdio transports
3. **Buffer Pooling**: Reuse byte buffers for serialization
4. **Lazy Initialization**: Defer expensive operations until needed
5. **Async Processing**: Non-blocking I/O throughout

### Bottleneck Mitigation
```rust
// Example: Parallel upstream requests
let futures = upstreams.iter().map(|upstream| {
    async move {
        process_upstream(upstream).await
    }
});

let results = futures::future::join_all(futures).await;
```