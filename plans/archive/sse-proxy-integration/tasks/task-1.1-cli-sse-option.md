# Task 1.1: Add SSE Transport CLI Option

## Overview
Add SSE and Streamable HTTP transport options to the forward proxy CLI, enabling Shadowcat to connect to MCP servers using the SSE transport.

**Duration**: 2 hours  
**Priority**: HIGH  
**Prerequisites**: Completed SSE transport implementation (Phase 1)

## Current State

The forward proxy CLI currently supports:
- `stdio` transport for subprocess communication
- `http` transport for basic HTTP communication

Missing:
- SSE transport option
- Streamable HTTP transport option (combined HTTP+SSE)
- URL validation for SSE endpoints
- Header configuration for MCP compliance

## Requirements

### Functional Requirements
1. Add `sse` subcommand to forward proxy
2. Add `streamable-http` subcommand for full MCP 2025-06-18 compliance
3. Parse and validate SSE endpoint URLs
4. Support custom headers configuration
5. Pass SSE-specific configuration to forward proxy

### Non-Functional Requirements
- Maintain backward compatibility with existing CLI
- Clear error messages for invalid URLs
- Help text explaining SSE vs Streamable HTTP differences

## Implementation Plan

### Step 1: Update CLI Module Structure
**Files**: `src/cli.rs`, `src/main.rs`

```rust
#[derive(Debug, Clone, Subcommand)]
pub enum ForwardTransport {
    /// Standard input/output transport
    Stdio {
        /// Command to execute
        command: Vec<String>,
    },
    /// Basic HTTP transport
    Http {
        #[arg(long, default_value = "8080")]
        port: u16,
        #[arg(long)]
        target: String,
    },
    /// Server-Sent Events transport (unidirectional streaming)
    Sse {
        /// SSE endpoint URL (e.g., https://server.com/sse)
        #[arg(long)]
        url: String,
        /// Optional MCP session ID
        #[arg(long)]
        session_id: Option<String>,
        /// MCP protocol version
        #[arg(long, default_value = "2025-06-18")]
        protocol_version: String,
        /// Optional Last-Event-ID for resumption
        #[arg(long)]
        last_event_id: Option<String>,
    },
    /// Streamable HTTP transport (MCP 2025-06-18)
    StreamableHttp {
        /// MCP endpoint URL (e.g., https://server.com/mcp)
        #[arg(long)]
        endpoint: String,
        /// Optional MCP session ID
        #[arg(long)]
        session_id: Option<String>,
        /// MCP protocol version
        #[arg(long, default_value = "2025-06-18")]
        protocol_version: String,
        /// Enable session management
        #[arg(long, default_value = "true")]
        session_management: bool,
    },
}
```

### Step 2: URL Validation
**File**: `src/cli/validation.rs` (new)

```rust
use url::Url;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliValidationError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("URL must use HTTP or HTTPS scheme")]
    InvalidScheme,
    #[error("URL must have a host")]
    MissingHost,
}

pub fn validate_sse_url(url: &str) -> Result<Url, CliValidationError> {
    let parsed = Url::parse(url)
        .map_err(|e| CliValidationError::InvalidUrl(e.to_string()))?;
    
    // Check scheme
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(CliValidationError::InvalidScheme);
    }
    
    // Check host
    if parsed.host().is_none() {
        return Err(CliValidationError::MissingHost);
    }
    
    Ok(parsed)
}
```

### Step 3: Configuration Passing
**File**: `src/main.rs`

```rust
async fn handle_forward_command(args: ForwardArgs) -> Result<()> {
    match args.transport {
        ForwardTransport::Stdio { command } => {
            // Existing stdio handling
        }
        ForwardTransport::Http { port, target } => {
            // Existing HTTP handling
        }
        ForwardTransport::Sse { url, session_id, protocol_version, last_event_id } => {
            // Validate URL
            let endpoint = validate_sse_url(&url)?;
            
            // Create SSE configuration
            let config = SseTransportConfig {
                url: endpoint,
                session_id,
                protocol_version,
                last_event_id,
                max_connections: 10,
                reconnect: true,
            };
            
            // Initialize forward proxy with SSE transport
            let proxy = ForwardProxy::new_with_sse(config).await?;
            proxy.run().await?;
        }
        ForwardTransport::StreamableHttp { endpoint, session_id, protocol_version, session_management } => {
            // Validate endpoint
            let url = validate_sse_url(&endpoint)?;
            
            // Create Streamable HTTP configuration
            let config = StreamableHttpConfig {
                endpoint: url,
                session_id,
                protocol_version,
                session_management,
                accept_types: vec!["application/json", "text/event-stream"],
            };
            
            // Initialize forward proxy with Streamable HTTP
            let proxy = ForwardProxy::new_with_streamable_http(config).await?;
            proxy.run().await?;
        }
    }
    Ok(())
}
```

### Step 4: Help Text and Documentation
**File**: `src/cli.rs`

Add comprehensive help text:
```rust
/// Forward proxy for MCP connections
///
/// Examples:
///   # Standard stdio transport
///   shadowcat forward stdio -- node server.js
///   
///   # SSE transport for streaming
///   shadowcat forward sse --url https://mcp.server.com/sse
///   
///   # Streamable HTTP (full MCP 2025-06-18)
///   shadowcat forward streamable-http --endpoint https://server.com/mcp
///   
/// Transport Types:
///   stdio          - Subprocess communication via stdin/stdout
///   http           - Basic HTTP request/response
///   sse            - Server-Sent Events for unidirectional streaming
///   streamable-http - Full MCP Streamable HTTP with bidirectional support
```

## Testing Plan

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_sse_url_valid() {
        assert!(validate_sse_url("https://example.com/sse").is_ok());
        assert!(validate_sse_url("http://localhost:8080/mcp").is_ok());
    }
    
    #[test]
    fn test_validate_sse_url_invalid() {
        assert!(validate_sse_url("ws://example.com/sse").is_err());
        assert!(validate_sse_url("not-a-url").is_err());
        assert!(validate_sse_url("file:///path/to/file").is_err());
    }
    
    #[test]
    fn test_cli_parsing() {
        let args = vec![
            "shadowcat",
            "forward",
            "sse",
            "--url",
            "https://server.com/sse",
            "--session-id",
            "test-123",
        ];
        
        let parsed = Cli::parse_from(args);
        // Verify parsing succeeds and values are correct
    }
}
```

### Integration Tests
1. Test SSE URL validation with various formats
2. Test configuration passing to forward proxy
3. Test error handling for invalid URLs
4. Test help text generation

## CLI Examples

```bash
# Basic SSE connection
shadowcat forward sse --url https://mcp.example.com/sse

# SSE with session resumption
shadowcat forward sse \
  --url https://mcp.example.com/sse \
  --session-id abc123 \
  --last-event-id evt-456

# Streamable HTTP with full features
shadowcat forward streamable-http \
  --endpoint https://server.com/mcp \
  --protocol-version 2025-06-18 \
  --session-management true

# SSE with custom protocol version
shadowcat forward sse \
  --url https://old.server.com/sse \
  --protocol-version 2025-03-26
```

## Success Criteria

- [ ] CLI accepts `sse` and `streamable-http` subcommands
- [ ] URL validation prevents invalid endpoints
- [ ] Configuration correctly passed to forward proxy
- [ ] Help text clearly explains transport options
- [ ] Backward compatibility maintained
- [ ] All tests passing

## Dependencies

- Existing CLI framework (clap)
- URL validation library (url crate)
- Forward proxy implementation (to be extended in Task 1.3)

## Notes

- Consider adding environment variable support for common settings
- May want to add config file support in future
- Should coordinate with Task 1.2 (SSE Transport Wrapper) for configuration structure
- Streamable HTTP option should be preferred for new implementations