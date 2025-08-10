# Module Architecture Design

## Overall Structure

```
cli/
├── mod.rs           # Public API, re-exports, module organization
├── common.rs        # Shared utilities and configuration
├── forward.rs       # Forward proxy implementations
├── reverse.rs       # Reverse proxy implementation
├── record.rs        # Recording implementations
├── replay.rs        # Replay server implementation
├── handlers.rs      # Shared HTTP request handlers
├── tape.rs          # (existing) Tape management
├── intercept.rs     # (existing) Interception rules
└── session.rs       # (existing) Session management
```

## Module Responsibilities

### cli/mod.rs
- **Purpose**: Public API and module organization
- **Exports**: Command execution functions
- **Pattern**: Similar to existing, but expanded
```rust
pub mod common;
pub mod forward;
pub mod reverse;
pub mod record;
pub mod replay;
pub mod handlers;
// existing modules
pub mod intercept;
pub mod session;
pub mod tape;

// Re-export main types
pub use common::ProxyConfig;
pub use forward::{execute_forward_stdio, execute_forward_http};
pub use reverse::execute_reverse;
// ... etc
```

### cli/common.rs
- **Purpose**: Shared configuration and utilities
- **Responsibilities**:
  - ProxyConfig struct and implementation
  - Rate limiter factory functions
  - Session manager factory functions
  - Shared error handling utilities
  - JSON conversion utilities
  - Common types and constants

**Key Components**:
```rust
pub struct ProxyConfig {
    pub enable_rate_limit: bool,
    pub rate_limit_rpm: u32,
    pub rate_limit_burst: u32,
    pub session_timeout: u64,
    pub max_sessions: usize,
    pub cleanup_interval: u64,
}

impl ProxyConfig {
    pub fn from_cli_args(...) -> Self
    pub fn to_session_config(&self) -> SessionConfig
    pub async fn create_rate_limiter(&self) -> Result<Option<Arc<MultiTierRateLimiter>>>
    pub fn create_session_manager(&self) -> Arc<SessionManager>
}

// Utilities
pub fn json_to_protocol_message(json: &Value) -> Result<ProtocolMessage>
pub fn protocol_message_to_json(msg: &ProtocolMessage) -> Value
```

### cli/forward.rs
- **Purpose**: Forward proxy command implementations
- **Responsibilities**:
  - Stdio forward proxy logic
  - HTTP forward proxy logic
  - Transport setup and management
  - Command spawning for stdio

**Public Interface**:
```rust
pub struct StdioForwardArgs {
    pub command: Vec<String>,
    pub config: ProxyConfig,
}

pub struct HttpForwardArgs {
    pub port: u16,
    pub target: String,
    pub command: Vec<String>,
    pub config: ProxyConfig,
}

pub async fn execute_stdio(args: StdioForwardArgs) -> Result<()>
pub async fn execute_http(args: HttpForwardArgs) -> Result<()>
```

### cli/reverse.rs
- **Purpose**: Reverse proxy server implementation
- **Responsibilities**:
  - Reverse proxy server setup
  - Configuration management
  - Integration with auth/policy

**Public Interface**:
```rust
pub struct ReverseProxyArgs {
    pub bind: String,
    pub upstream: String,
    pub config: ProxyConfig,
}

pub async fn execute(args: ReverseProxyArgs) -> Result<()>
```

### cli/record.rs
- **Purpose**: Session recording implementations
- **Responsibilities**:
  - Stdio recording logic
  - HTTP recording server
  - Tape recorder management
  - Message capture

**Public Interface**:
```rust
pub struct StdioRecordArgs {
    pub output: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub command: Vec<String>,
    pub storage_dir: PathBuf,
}

pub struct HttpRecordArgs {
    pub output: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub port: u16,
    pub storage_dir: PathBuf,
}

pub async fn execute_stdio(args: StdioRecordArgs) -> Result<()>
pub async fn execute_http(args: HttpRecordArgs) -> Result<()>
```

### cli/replay.rs
- **Purpose**: Tape replay server
- **Responsibilities**:
  - Load and parse tapes
  - HTTP server for replay
  - Request matching logic
  - Response playback

**Public Interface**:
```rust
pub struct ReplayArgs {
    pub tape_file: String,
    pub port: String,
    pub storage_dir: PathBuf,
    pub rate_limit: bool,
    pub rate_limit_rpm: u32,
    pub rate_limit_burst: u32,
}

pub async fn execute(args: ReplayArgs) -> Result<()>
```

### cli/handlers.rs
- **Purpose**: Shared HTTP request handlers
- **Responsibilities**:
  - Common HTTP processing logic
  - Request/response conversion
  - Error response formatting
  - Middleware utilities

**Public Interface**:
```rust
pub async fn handle_proxy_request(req: Request, context: ProxyContext) -> Response
pub async fn handle_record_request(req: Request, recorder: Arc<TapeRecorder>) -> Response
pub async fn handle_replay_request(req: Request, player: Arc<TapePlayer>) -> Response
pub fn create_error_response(status: StatusCode, message: &str) -> Response
```

## Design Principles

### 1. Single Responsibility
Each module has a clear, focused purpose. No module tries to do too much.

### 2. Dependency Direction
- Common module has no dependencies on other CLI modules
- Command modules depend on common
- Handlers can be used by command modules
- No circular dependencies

### 3. Configuration Flow
```
main.rs 
  ↓ (parse CLI args)
ProxyConfig::from_cli_args()
  ↓ (pass to module)
Module::execute(args)
  ↓ (use common utilities)
common::create_rate_limiter()
common::create_session_manager()
```

### 4. Error Handling
- All modules use `shadowcat::Result<T>`
- Errors bubble up to main.rs
- Common error formatting in handlers

### 5. Testing Strategy
- Each module can be tested independently
- Common utilities have unit tests
- Command modules have integration tests
- Mock transports for testing

## Module Communication

### Data Types
- **Shared through common.rs**: ProxyConfig, error types
- **From transport module**: ProtocolMessage, Transport traits
- **From session module**: SessionManager, SessionConfig
- **From rate_limiting module**: MultiTierRateLimiter

### No Direct Dependencies
- Forward doesn't know about Reverse
- Record doesn't know about Replay
- All share through common module

### State Management
- Shared state uses Arc<T> for thread safety
- No global mutable state
- Each command creates its own instances

## Migration Benefits

### Before (main.rs)
- 1294 lines in single file
- Mixed responsibilities
- Hard to test
- Code duplication

### After (cli modules)
- main.rs: ~150 lines (just CLI parsing and dispatch)
- Each module: 100-200 lines (focused responsibility)
- Testable units
- Shared utilities (no duplication)

## Compatibility Guarantees

### CLI Interface
- Exact same command structure
- All arguments preserved
- Same error messages
- No breaking changes

### Internal APIs
- New modules don't affect existing tape/intercept/session
- Can migrate incrementally
- Feature flags if needed