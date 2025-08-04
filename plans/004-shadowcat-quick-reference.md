# Shadowcat Quick Reference

## Key Commands

```bash
# Development
cargo watch -x check -x test -x "run -- --help"
cargo test -- --nocapture
cargo test transport::stdio::tests
RUST_LOG=shadowcat=debug cargo run

# Benchmarking
cargo bench
cargo flamegraph

# Code Quality
cargo fmt
cargo clippy -- -D warnings
cargo doc --open
```

## Core Types Reference

```rust
// Transport Messages
TransportMessage::Request { id, method, params }
TransportMessage::Response { id, result, error }
TransportMessage::Notification { method, params }

// Directions
Direction::ClientToServer
Direction::ServerToClient

// Transport Types
TransportType::Stdio
TransportType::Http
TransportType::Sse

// Session States
AuthState::Unauthenticated
AuthState::Authenticated(token)

// Intercept Actions
InterceptAction::Continue
InterceptAction::Modify(msg)
InterceptAction::Block(reason)
InterceptAction::Pause { resume_tx }
```

## MCP Protocol Constants

```rust
const MCP_PROTOCOL_VERSION: &str = "2025-11-05";
const SESSION_HEADER: &str = "Mcp-Session-Id";
const VERSION_HEADER: &str = "MCP-Protocol-Version";
```

## Common Patterns

### Result Type
```rust
use crate::error::Result;
pub async fn my_function() -> Result<()> {
    // Use ? for error propagation
    Ok(())
}
```

### Instrumented Functions
```rust
#[instrument(skip(large_param), fields(session_id = %id))]
pub async fn process(id: &SessionId, large_param: &Data) -> Result<()> {
    debug!("Processing started");
    Ok(())
}
```

### Testing Helpers
```rust
#[tokio::test]
async fn test_something() {
    // Your test
}

// Mock creation
let mut mock = MockTransport::new();
mock.expect_send()
    .times(1)
    .returning(|_| Ok(()));
```

### Error Context
```rust
use anyhow::Context;

something.await
    .context("Failed to do something")?;
    
something.await
    .with_context(|| format!("Failed with id: {}", id))?;
```

## File Structure

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Public API
├── error.rs         # Error types
├── transport/
│   ├── mod.rs       # Transport trait
│   ├── stdio.rs     # Process transport
│   └── http.rs      # HTTP/SSE transport
├── proxy/
│   ├── mod.rs       # Proxy traits
│   ├── forward.rs   # Client → Shadowcat → Server
│   └── reverse.rs   # Client → Shadowcat (auth) → Server
├── session/
│   ├── mod.rs       # Session types
│   ├── manager.rs   # Session lifecycle
│   └── store.rs     # Persistence
├── interceptor/
│   ├── mod.rs       # Interceptor trait
│   ├── engine.rs    # Chain processing
│   └── rules.rs     # Rule matching
├── recorder/
│   ├── mod.rs       # Recording types
│   ├── tape.rs      # Tape format
│   └── replay.rs    # Playback engine
└── auth/
    ├── mod.rs       # Auth types
    ├── oauth.rs     # OAuth 2.1
    └── validator.rs # Token validation
```

## Environment Variables

```bash
RUST_LOG=shadowcat=debug      # Logging level
RUST_BACKTRACE=1             # Show backtraces
SHADOWCAT_CONFIG=config.yaml  # Config file path
```

## Cargo Features

```toml
[features]
default = ["stdio", "http"]
stdio = []
http = ["dep:axum", "dep:tower", "dep:tower-http"]
recording = ["dep:sqlx"]
metrics = ["dep:prometheus", "dep:opentelemetry"]
debug-ui = ["dep:ratatui"]
```

## Common Imports

```rust
// External
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, error, info, instrument, warn};

// Internal
use crate::error::{Result, ShadowcatError};
use crate::transport::{Transport, TransportMessage, TransportType};
use crate::session::{Session, SessionId, SessionManager};
```

## Debug Macros

```rust
// Quick debug print for dev
macro_rules! dbg_msg {
    ($msg:expr) => {
        eprintln!("[{}:{}] {:?}", file!(), line!(), $msg);
    };
}

// Conditional compilation
#[cfg(debug_assertions)]
eprintln!("Debug: {:?}", value);
```

## Testing Utilities

```rust
// Create test transport
fn test_transport() -> Box<dyn Transport> {
    Box::new(MockTransport::new())
}

// Create test session
fn test_session() -> Session {
    Session {
        id: SessionId::new(),
        transport: TransportType::Stdio,
        // ...
    }
}

// Assert async errors
assert!(matches!(
    result.await,
    Err(ShadowcatError::Transport(_))
));
```