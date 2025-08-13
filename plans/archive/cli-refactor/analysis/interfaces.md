# Module Interfaces

## Common Module (`cli/common.rs`)

### Configuration Management
```rust
use shadowcat::session::SessionConfig;
use shadowcat::rate_limiting::{MultiTierRateLimiter, RateLimitConfig};
use std::sync::Arc;

/// Unified proxy configuration used across commands
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub enable_rate_limit: bool,
    pub rate_limit_rpm: u32,
    pub rate_limit_burst: u32,
    pub session_timeout: u64,
    pub max_sessions: usize,
    pub cleanup_interval: u64,
}

impl ProxyConfig {
    /// Create from CLI arguments (used by main.rs)
    pub fn from_cli_args(
        rate_limit: bool,
        rate_limit_rpm: u32,
        rate_limit_burst: u32,
        session_timeout: u64,
        max_sessions: usize,
        cleanup_interval: u64,
    ) -> Self;
    
    /// Convert to session configuration
    pub fn to_session_config(&self) -> SessionConfig;
    
    /// Create rate limiter if enabled
    pub async fn create_rate_limiter(&self) -> Result<Option<Arc<MultiTierRateLimiter>>>;
    
    /// Create session manager with config
    pub fn create_session_manager(&self) -> Arc<SessionManager>;
}
```

### Utility Functions
```rust
use shadowcat::transport::ProtocolMessage;
use serde_json::Value;

/// Convert JSON to protocol message
pub fn json_to_protocol_message(json: &Value) -> Result<ProtocolMessage>;

/// Convert protocol message to JSON
pub fn protocol_message_to_json(msg: &ProtocolMessage) -> Value;

/// Create standard error response
pub fn create_cli_error(message: &str) -> Error;
```

## Forward Module (`cli/forward.rs`)

### Stdio Forward
```rust
use crate::cli::common::ProxyConfig;

/// Arguments for stdio forward proxy
pub struct StdioForwardArgs {
    pub command: Vec<String>,
    pub config: ProxyConfig,
}

/// Execute stdio forward proxy
pub async fn execute_stdio(args: StdioForwardArgs) -> Result<()>;
```

### HTTP Forward
```rust
/// Arguments for HTTP forward proxy
pub struct HttpForwardArgs {
    pub port: u16,
    pub target: String,
    pub command: Vec<String>,  // For stdio targets
    pub config: ProxyConfig,
}

/// Execute HTTP forward proxy
pub async fn execute_http(args: HttpForwardArgs) -> Result<()>;
```

## Reverse Module (`cli/reverse.rs`)

```rust
use crate::cli::common::ProxyConfig;

/// Arguments for reverse proxy
pub struct ReverseProxyArgs {
    pub bind: String,
    pub upstream: String,
    pub config: ProxyConfig,
}

/// Execute reverse proxy server
pub async fn execute(args: ReverseProxyArgs) -> Result<()>;
```

## Record Module (`cli/record.rs`)

### Stdio Recording
```rust
use std::path::PathBuf;

/// Arguments for stdio recording
pub struct StdioRecordArgs {
    pub output: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub command: Vec<String>,
    pub storage_dir: PathBuf,
}

/// Execute stdio recording
pub async fn execute_stdio(args: StdioRecordArgs) -> Result<()>;
```

### HTTP Recording
```rust
/// Arguments for HTTP recording
pub struct HttpRecordArgs {
    pub output: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub port: u16,
    pub storage_dir: PathBuf,
}

/// Execute HTTP recording server
pub async fn execute_http(args: HttpRecordArgs) -> Result<()>;
```

## Replay Module (`cli/replay.rs`)

```rust
use std::path::PathBuf;

/// Arguments for replay server
pub struct ReplayArgs {
    pub tape_file: String,
    pub port: String,
    pub storage_dir: PathBuf,
    pub rate_limit: bool,
    pub rate_limit_rpm: u32,
    pub rate_limit_burst: u32,
}

/// Execute replay server
pub async fn execute(args: ReplayArgs) -> Result<()>;
```

## Handlers Module (`cli/handlers.rs`)

```rust
use axum::{extract::Request, response::Response};
use shadowcat::recorder::{TapeRecorder, TapePlayer};
use std::sync::Arc;

/// Context for proxy request handling
pub struct ProxyContext {
    pub session_manager: Arc<SessionManager>,
    pub rate_limiter: Option<Arc<MultiTierRateLimiter>>,
    pub target: String,
}

/// Handle incoming proxy request
pub async fn handle_proxy_request(
    req: Request,
    context: ProxyContext,
) -> Response;

/// Handle recording request
pub async fn handle_record_request(
    req: Request,
    recorder: Arc<TapeRecorder>,
) -> Response;

/// Handle replay request
pub async fn handle_replay_request(
    req: Request,
    player: Arc<TapePlayer>,
) -> Response;

/// Create standard error response
pub fn create_error_response(
    status: StatusCode,
    message: &str,
) -> Response;
```

## Main.rs Interface

### Simplified Command Dispatch
```rust
use shadowcat::cli::{forward, reverse, record, replay};
use shadowcat::cli::common::ProxyConfig;

// In main() function, after parsing CLI:
match cli.command {
    Commands::Forward { transport } => match transport {
        ForwardTransport::Stdio { /* args */ } => {
            let config = ProxyConfig::from_cli_args(/* ... */);
            let args = forward::StdioForwardArgs { command, config };
            forward::execute_stdio(args).await
        }
        ForwardTransport::Http { /* args */ } => {
            let config = ProxyConfig::from_cli_args(/* ... */);
            let args = forward::HttpForwardArgs { port, target, command, config };
            forward::execute_http(args).await
        }
    },
    Commands::Reverse { /* args */ } => {
        let config = ProxyConfig::from_cli_args(/* ... */);
        let args = reverse::ReverseProxyArgs { bind, upstream, config };
        reverse::execute(args).await
    },
    // ... similar for other commands
}
```

## Error Handling Contract

### Module Errors
- All public functions return `shadowcat::Result<T>`
- Errors include context via `anyhow::Context`
- No direct process exits (leave to main.rs)

### Error Types
```rust
use shadowcat::error::ShadowcatError;

// All modules use the same error type
pub type Result<T> = std::result::Result<T, ShadowcatError>;
```

## Testing Interfaces

### Mock Builders
```rust
// In common.rs for testing
#[cfg(test)]
pub mod test_helpers {
    pub fn mock_proxy_config() -> ProxyConfig;
    pub fn mock_session_manager() -> Arc<SessionManager>;
    pub fn mock_rate_limiter() -> Arc<MultiTierRateLimiter>;
}
```

### Module Testing
```rust
// Each module can be tested independently
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::common::test_helpers::*;
    
    #[tokio::test]
    async fn test_execute_stdio() {
        let args = StdioForwardArgs {
            command: vec!["echo".to_string()],
            config: mock_proxy_config(),
        };
        // Test implementation
    }
}
```

## Dependency Rules

### Allowed Dependencies
- **common.rs**: Only shadowcat core modules (no cli modules)
- **Command modules**: Can use common.rs and handlers.rs
- **handlers.rs**: Can use common.rs
- **main.rs**: Can use all cli modules

### Forbidden Dependencies
- No circular dependencies between modules
- No direct command-to-command dependencies
- No reaching into module internals (only public API)

## Versioning Strategy

### Public API Stability
- Public functions marked with doc comments
- Internal functions not exposed
- Breaking changes require major version bump

### Migration Path
```rust
// Can add compatibility shims if needed
#[deprecated(since = "2.0.0", note = "Use execute_stdio instead")]
pub async fn run_stdio_forward(/* old signature */) -> Result<()> {
    // Adapter to new interface
}
```