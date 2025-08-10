# Data Flow Design

## Configuration Flow

### 1. CLI Parsing to Execution
```
User Input
    ↓
main.rs::Cli::parse()
    ↓
Match command variant
    ↓
Extract CLI arguments
    ↓
ProxyConfig::from_cli_args()
    ↓
Create module arguments struct
    ↓
module::execute(args)
    ↓
Result<()> back to main
    ↓
Exit with status code
```

### 2. ProxyConfig Lifecycle
```
CLI Arguments (bool, u32, etc.)
    ↓
ProxyConfig::from_cli_args()
    ↓
ProxyConfig struct
    ├→ config.to_session_config() → SessionManager
    ├→ config.create_rate_limiter() → Option<RateLimiter>
    └→ Passed to handlers for runtime checks
```

### 3. Session Manager Flow
```
ProxyConfig
    ↓
SessionConfig
    ↓
SessionManager::with_config()
    ↓
Arc<SessionManager>
    ├→ Shared across async tasks
    ├→ Tracks all active sessions
    └→ Cleanup task spawned
```

## Module Communication Patterns

### 1. Forward Proxy Flow (Stdio)
```
forward::execute_stdio()
    ↓
Create SessionManager (via common)
    ↓
Create RateLimiter (via common)
    ↓
Spawn Command process
    ↓
Create StdioTransport
    ↓
Message Loop:
    Client → Transport → RateLimit Check → Server
    Server → Transport → Client
```

### 2. HTTP Forward Proxy Flow
```
forward::execute_http()
    ↓
Create axum Router
    ↓
Share SessionManager + RateLimiter via State
    ↓
HTTP Request arrives
    ↓
handlers::handle_proxy_request()
    ├→ Rate limit check
    ├→ Session lookup/create
    ├→ Forward to target
    └→ Return response
```

### 3. Recording Flow
```
record::execute_stdio/http()
    ↓
Create TapeRecorder
    ↓
For each message:
    ├→ Capture to recorder
    ├→ Forward to destination
    └→ Record response
    ↓
On completion:
    └→ Save tape to storage
```

### 4. Replay Flow
```
replay::execute()
    ↓
Load tape from file
    ↓
Create TapePlayer
    ↓
Start HTTP server
    ↓
For each request:
    ├→ Match against tape
    ├→ Apply rate limiting
    └→ Return recorded response
```

## Shared State Management

### 1. Arc Pattern for Thread Safety
```rust
// Created once, shared across tasks
let session_manager = Arc::new(SessionManager::new());
let rate_limiter = Arc::new(RateLimiter::new());

// Cloned for each async task
let sm_clone = session_manager.clone();
tokio::spawn(async move {
    sm_clone.track_session(...);
});
```

### 2. No Global State
- Each command creates its own instances
- No static/global variables
- All state passed explicitly

### 3. Cleanup and Lifecycle
```
Command starts
    ↓
Create resources (SessionManager, RateLimiter, etc.)
    ↓
Spawn cleanup tasks
    ↓
Run main logic
    ↓
Graceful shutdown on completion/error
    ↓
Drop all Arc references
    ↓
Resources cleaned up
```

## Error Propagation

### 1. Error Flow
```
Deep function error
    ↓
Add context with .context("Failed to X")
    ↓
Return Err up the stack
    ↓
Module function catches
    ↓
Add module-level context
    ↓
Return to main.rs
    ↓
Log error with full chain
    ↓
Exit with status 1
```

### 2. Error Types by Layer
```
Transport Layer: IO errors, connection failures
    ↓
Session Layer: Session limits, timeout errors
    ↓
Rate Limit Layer: Rate limit exceeded
    ↓
CLI Layer: Configuration errors, invalid arguments
    ↓
Main: Logs and exits
```

## Message Processing Pipeline

### 1. Inbound Message Flow
```
External Message Source (stdio/HTTP)
    ↓
Transport Layer (parse/validate)
    ↓
Create MessageEnvelope with context
    ↓
Session Manager (track/validate)
    ↓
Rate Limiter (check limits)
    ↓
Interceptor Chain (optional)
    ↓
Forward to destination
```

### 2. Outbound Message Flow
```
Response from destination
    ↓
Wrap in MessageEnvelope
    ↓
Session tracking
    ↓
Recording (if enabled)
    ↓
Transport layer (serialize)
    ↓
Send to client
```

## Resource Management

### 1. Session Lifecycle
```
New connection
    ↓
Create/retrieve session
    ↓
Track in SessionManager
    ↓
Activity updates touch timestamp
    ↓
Cleanup task checks timeouts
    ↓
Remove expired sessions
```

### 2. Rate Limiter Buckets
```
Request arrives
    ↓
Extract request context
    ↓
Check multiple tiers:
    ├→ Global limit
    ├→ Per-IP limit
    ├→ Per-user limit
    └→ Per-endpoint limit
    ↓
Update token buckets
    ↓
Allow or reject
```

## Testing Data Flow

### 1. Unit Test Flow
```
Create mock dependencies
    ↓
Create module args struct
    ↓
Call module function
    ↓
Assert on mock interactions
    ↓
Verify return value
```

### 2. Integration Test Flow
```
Start test server/process
    ↓
Create real SessionManager
    ↓
Send test messages
    ↓
Verify full pipeline
    ↓
Check recorded results
    ↓
Cleanup test resources
```

## Performance Considerations

### 1. Async Task Spawning
- Commands spawn long-running tasks
- Use tokio::spawn for parallelism
- Avoid blocking operations

### 2. Resource Pooling
- Connection pools for HTTP
- Reuse transports where possible
- Buffer pools for message processing

### 3. Memory Management
- Arc for shared immutable state
- Avoid unnecessary clones
- Stream large responses

## Security Boundaries

### 1. Input Validation
```
CLI args → Validate in main.rs
HTTP requests → Validate in handlers
Protocol messages → Validate in transport
```

### 2. Token/Secret Handling
- Never log sensitive data
- Never forward client tokens upstream
- Sanitize error messages

### 3. Resource Limits
- Max sessions enforced
- Rate limiting applied
- Message size limits
- Timeout enforcement