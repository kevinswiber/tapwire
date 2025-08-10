# Dependency Analysis

## Shared Dependencies

### ProxyConfig
- **Used by**: forward (stdio/HTTP), reverse, replay commands
- **Purpose**: Centralized configuration for rate limiting and session management
- **Duplication**: CLI args parsing repeated for each command variant

### SessionManager
- **Used by**: forward (stdio/HTTP), reverse, recording (stdio/HTTP)
- **Creation Pattern**: Arc::new(SessionManager::with_config())
- **Configuration**: Derived from ProxyConfig
- **Cleanup Task**: Started for forward and reverse proxies

### MultiTierRateLimiter
- **Used by**: forward (stdio/HTTP), reverse, replay
- **Creation Pattern**: Repeated config setup and initialization
- **Configuration**: Only global tier enabled, others disabled
- **Code Duplication**: 3 nearly identical setup blocks

### Transport Types
- **StdioTransport**: Used by forward-stdio, record-stdio
- **HTTP Server (axum)**: Used by forward-http, reverse, record-http, replay

## Command-Specific Dependencies

### Forward Command
- **Stdio variant**:
  - StdioTransport
  - SessionManager
  - Optional RateLimiter
  - Command spawning
  
- **HTTP variant**:
  - axum Router
  - SessionManager  
  - Optional RateLimiter
  - HTTP-to-stdio proxy logic

### Reverse Command
- ReverseProxyServer
- ReverseProxyConfig
- ReverseUpstreamConfig
- SessionManager
- Optional RateLimiter

### Record Command
- **Stdio variant**:
  - TapeRecorder
  - StdioTransport
  - Command spawning
  
- **HTTP variant**:
  - TapeRecorder
  - axum Router
  - HTTP server

### Replay Command
- TapePlayer
- axum Router
- Optional RateLimiter
- Message matching logic

### Tape/Intercept/Session Commands
- **Self-contained**: Each has its own CLI module
- **No shared dependencies**: Direct delegation to module

## Helper Function Dependencies

### JSON Conversion Functions
- **json_to_transport_message()**:
  - Used by: HTTP handlers (implicit)
  - Dependencies: ProtocolMessage, serde_json
  
- **transport_message_to_json()**:
  - Used by: HTTP handlers (implicit)
  - Dependencies: ProtocolMessage, serde_json

### Message Matching
- **messages_match()**:
  - Used by: replay handler only
  - Dependencies: ProtocolMessage

## External Crate Dependencies
- **clap**: CLI parsing
- **axum**: HTTP server framework
- **tokio**: Async runtime (Command spawning)
- **tracing**: Logging framework
- **serde_json**: JSON handling
- **uuid**: Request ID generation

## Coupling Issues

### Tight Coupling
1. **ProxyConfig creation**: Tightly coupled to CLI arg structure
2. **Rate limiter setup**: Duplicated configuration logic
3. **HTTP handlers**: Embedded in main.rs instead of reusable

### Loose Coupling  
1. **Tape/Intercept/Session**: Clean delegation to modules
2. **Transport abstraction**: Clean interface usage

## Shared Patterns

### Initialization Pattern
```rust
1. Parse CLI args
2. Create ProxyConfig (if applicable)
3. Create SessionManager
4. Setup RateLimiter (if enabled)
5. Start transport/server
6. Handle messages
```

### Error Handling Pattern
- All functions return `shadowcat::Result<()>`
- Errors bubble up to main()
- main() logs and calls exit(1)

## Refactoring Opportunities

### High Value Extractions
1. **ProxyConfig + initialization**: Shared by 4 commands
2. **Rate limiter setup**: Duplicated 3 times
3. **HTTP server setup**: Common patterns across commands
4. **Session manager initialization**: Repeated pattern

### Medium Value Extractions
1. **JSON conversion utilities**: Could be in transport module
2. **Command spawning logic**: Shared by stdio variants
3. **Error handling helpers**: Consistent exit patterns