# Task 003: Add Request Size Limits

## Overview
Implement configurable request size limits to prevent memory exhaustion attacks and ensure system stability.

## Context
The [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md) identified missing request size limits as a security vulnerability that could lead to OOM conditions.

## Requirements

1. Add configurable max request size (default: 10MB)
2. Add configurable max response size (default: 10MB) 
3. Reject oversized requests early with clear error
4. Apply limits to all transport types (stdio, HTTP)
5. Make limits configurable via config file

## Implementation Plan

### Step 1: Add Configuration Fields

**File**: `src/config.rs`

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct TransportConfig {
    // ... existing fields ...
    
    #[serde(default = "default_max_request_size")]
    pub max_request_size: usize,
    
    #[serde(default = "default_max_response_size")]
    pub max_response_size: usize,
    
    #[serde(default = "default_max_header_size")]
    pub max_header_size: usize,
}

fn default_max_request_size() -> usize {
    10 * 1024 * 1024  // 10MB
}

fn default_max_response_size() -> usize {
    10 * 1024 * 1024  // 10MB
}

fn default_max_header_size() -> usize {
    8 * 1024  // 8KB
}
```

### Step 2: Create Size Validator

**File**: `src/transport/size_limiter.rs` (new file)

```rust
use crate::error::TransportError;

/// Validates message sizes against configured limits
pub struct SizeLimiter {
    max_request_size: usize,
    max_response_size: usize,
    max_header_size: usize,
}

impl SizeLimiter {
    pub fn new(config: &TransportConfig) -> Self {
        Self {
            max_request_size: config.max_request_size,
            max_response_size: config.max_response_size,
            max_header_size: config.max_header_size,
        }
    }
    
    pub fn check_request_size(&self, size: usize) -> Result<(), TransportError> {
        if size > self.max_request_size {
            return Err(TransportError::RequestTooLarge {
                size,
                limit: self.max_request_size,
            });
        }
        Ok(())
    }
    
    pub fn check_response_size(&self, size: usize) -> Result<(), TransportError> {
        if size > self.max_response_size {
            return Err(TransportError::ResponseTooLarge {
                size,
                limit: self.max_response_size,
            });
        }
        Ok(())
    }
}
```

### Step 3: Update Error Types

**File**: `src/error.rs`

```rust
#[derive(Error, Debug)]
pub enum TransportError {
    // ... existing variants ...
    
    #[error("Request too large: {size} bytes exceeds limit of {limit} bytes")]
    RequestTooLarge {
        size: usize,
        limit: usize,
    },
    
    #[error("Response too large: {size} bytes exceeds limit of {limit} bytes")]
    ResponseTooLarge {
        size: usize,
        limit: usize,
    },
    
    #[error("Header too large: {size} bytes exceeds limit of {limit} bytes")]
    HeaderTooLarge {
        size: usize,
        limit: usize,
    },
}
```

### Step 4: Implement in StdioTransport

**File**: `src/transport/stdio.rs`

```rust
use crate::transport::size_limiter::SizeLimiter;

pub struct StdioTransport {
    // ... existing fields ...
    size_limiter: SizeLimiter,
}

impl StdioTransport {
    pub fn new(config: TransportConfig) -> Self {
        let size_limiter = SizeLimiter::new(&config);
        // ... rest of initialization ...
    }
    
    async fn read_message(&mut self) -> TransportResult<String> {
        let mut buffer = String::new();
        let mut temp_buffer = [0u8; 8192];  // Read in chunks
        let mut total_size = 0;
        
        loop {
            let bytes_read = self.reader.read(&mut temp_buffer).await?;
            if bytes_read == 0 {
                break;
            }
            
            total_size += bytes_read;
            
            // Check size before accumulating
            self.size_limiter.check_request_size(total_size)?;
            
            buffer.push_str(&String::from_utf8_lossy(&temp_buffer[..bytes_read]));
            
            // Check for complete JSON message
            if is_complete_json(&buffer) {
                break;
            }
        }
        
        Ok(buffer)
    }
}
```

### Step 5: Implement in HTTP Transport

**File**: `src/transport/http.rs`

```rust
use axum::{
    extract::{ContentLengthLimit, State},
    http::StatusCode,
    response::IntoResponse,
};

pub struct HttpTransport {
    // ... existing fields ...
    size_limiter: SizeLimiter,
}

// In router setup
fn create_router(config: TransportConfig) -> Router {
    Router::new()
        .route("/mcp", post(handle_request))
        .layer(ContentLengthLimit::max(config.max_request_size))
        .with_state(AppState { config })
}

async fn handle_request(
    State(state): State<AppState>,
    ContentLengthLimit(body): ContentLengthLimit<Bytes, { 10_485_760 }>, // 10MB
) -> impl IntoResponse {
    // Size is already limited by ContentLengthLimit
    // Process request...
}
```

### Step 6: Add Streaming Support for Large Messages

**File**: `src/transport/streaming.rs` (new file)

```rust
/// Handle large messages in chunks to avoid loading entire message in memory
pub struct StreamingProcessor {
    chunk_size: usize,
    size_limiter: SizeLimiter,
}

impl StreamingProcessor {
    pub async fn process_stream<R>(
        &self,
        mut reader: R,
    ) -> Result<TransportMessage, TransportError>
    where
        R: AsyncRead + Unpin,
    {
        let mut parser = JsonStreamParser::new();
        let mut total_read = 0;
        let mut buffer = vec![0u8; self.chunk_size];
        
        loop {
            let n = reader.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            
            total_read += n;
            self.size_limiter.check_request_size(total_read)?;
            
            parser.feed(&buffer[..n])?;
            
            if let Some(message) = parser.try_parse()? {
                return Ok(message);
            }
        }
        
        Err(TransportError::IncompleteMessage)
    }
}
```

### Step 7: Add Metrics

```rust
// Track rejected requests
metrics::counter!("shadowcat_rejected_requests_total", "reason" => "size_limit").increment(1);

// Track request sizes
metrics::histogram!("shadowcat_request_size_bytes").record(size as f64);
```

## Configuration Example

```toml
# shadowcat.toml
[transport]
max_request_size = 10485760   # 10MB
max_response_size = 10485760  # 10MB
max_header_size = 8192        # 8KB

[transport.stdio]
max_request_size = 5242880    # 5MB for stdio specifically

[transport.http]
max_request_size = 52428800   # 50MB for HTTP
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_size_limiter_accepts_valid_size() {
        let limiter = SizeLimiter::new(&default_config());
        assert!(limiter.check_request_size(1024).is_ok());
    }
    
    #[test]
    fn test_size_limiter_rejects_oversized() {
        let limiter = SizeLimiter::new(&small_limit_config());
        let result = limiter.check_request_size(2048);
        assert!(matches!(result, Err(TransportError::RequestTooLarge { .. })));
    }
    
    #[tokio::test]
    async fn test_stdio_rejects_large_message() {
        let config = small_limit_config();
        let mut transport = StdioTransport::new(config);
        
        // Send oversized message
        let large_msg = "x".repeat(10_000);
        let result = transport.process_message(&large_msg).await;
        
        assert!(matches!(result, Err(TransportError::RequestTooLarge { .. })));
    }
}
```

### Integration Tests

```bash
# Test with large payload
echo '{"jsonrpc":"2.0","method":"test","params":"'$(head -c 20M < /dev/zero | tr '\0' 'x')'"}' | \
  ./target/debug/shadowcat forward stdio -- cat

# Should fail with "Request too large" error
```

## Validation

- [ ] Size limits are configurable
- [ ] Stdio transport enforces limits
- [ ] HTTP transport enforces limits
- [ ] Clear error messages for violations
- [ ] No memory exhaustion with large payloads
- [ ] Metrics track rejected requests
- [ ] Tests verify limit enforcement

## Performance Considerations

1. **Chunked reading**: Don't load entire message into memory
2. **Early rejection**: Check Content-Length header when available
3. **Streaming parsing**: Use incremental JSON parser for large messages
4. **Buffer pooling**: Reuse buffers to reduce allocations

## Success Criteria

- [ ] Cannot crash service with large requests
- [ ] Memory usage stays bounded under load
- [ ] Performance impact <1% for normal-sized requests
- [ ] Configuration works as documented
- [ ] All transports enforce limits consistently