# Task B.0: Add ResponseMode Enum

## Objective
Create the ResponseMode enum and ClientCapabilities bitflags to properly track response formats and client capabilities, replacing the dead is_sse_session boolean with a clean, type-safe solution.

## Context from Phase A Analysis

### Key Findings (Reference: analysis/response-mode-investigation.md)
- **is_sse_session is completely dead code** - The flag exists but `mark_as_sse_session()` is never called
- **Response format detection is runtime-based** - Detected via Content-Type headers, not session configuration
- **Response mode is per-response, not per-session** - A session can have different response modes over time
- **No backward compatibility needed** - Shadowcat is unreleased, we can do this right

### Design Decisions (Reference: analysis/design-decisions.md)
- **Decision #1**: ResponseMode as separate enum from TransportType (orthogonal concerns)
- **Decision #9**: Maintain SessionStore compatibility (keep serializable)
- **Use bitflags over enumflags2** (analysis/implementation-recommendations.md) for:
  - Better const support
  - Serde integration
  - Already a transitive dependency

### Architecture Context (Reference: analysis/architecture-proposal.md)
ResponseMode fits in the transport/core layer as it's fundamental to transport behavior but orthogonal to TransportType. It affects:
- How proxies route responses (JSON buffered vs SSE streamed)
- Whether interceptors can process messages
- How responses are forwarded to clients

## Prerequisites
- [x] Phase A analysis complete
- [x] Design decisions documented
- [ ] shadowcat repository checked out
- [ ] On `refactor/transport-type-architecture` branch (or create if needed)

## Detailed Implementation Steps

### Step 1: Prepare Environment (5 min)

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Create branch if not exists
git checkout -b refactor/transport-type-architecture || git checkout refactor/transport-type-architecture

# Ensure we're up to date
git pull origin main
git merge main  # If needed

# Add dependencies to Cargo.toml
```

Edit `Cargo.toml` to add (if not present):
```toml
[dependencies]
bitflags = { version = "2.9", features = ["serde"] }
# mime should already be present (verify with: grep mime Cargo.toml)
```

### Step 2: Create Core Module Structure (5 min)

```bash
# Create core module if it doesn't exist
mkdir -p src/transport/core

# Create new files
touch src/transport/core/response_mode.rs
touch src/transport/core/capabilities.rs
```

If `src/transport/core/mod.rs` doesn't exist, create it:
```rust
//! Core transport types and enums
//! 
//! This module contains fundamental types used throughout the transport layer.
//! These types are orthogonal to specific transport implementations.

pub mod response_mode;
pub mod capabilities;

pub use response_mode::ResponseMode;
pub use capabilities::ClientCapabilities;
```

### Step 3: Implement ResponseMode Enum (20 min)

Create `src/transport/core/response_mode.rs`:

```rust
//! Response format tracking for MCP messages
//! 
//! This replaces the dead is_sse_session boolean with proper type-safe response
//! format tracking. Response mode is detected at runtime from Content-Type headers,
//! not configured at session creation.
//!
//! Design rationale: analysis/response-mode-investigation.md

use mime::Mime;
use serde::{Deserialize, Serialize};

/// Represents the format of a response from the upstream server.
/// 
/// This enum tracks how responses should be processed and forwarded:
/// - Json: Buffered, parsed, can be intercepted
/// - SseStream: Streamed, event-based, requires special handling
/// - Passthrough: Unknown format, streamed without processing
/// 
/// Note: We intentionally don't include Binary, Text, or WebSocket variants.
/// Binary and Text are handled as Passthrough. WebSocket will be added when
/// the MCP spec supports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseMode {
    /// Standard JSON-RPC response (application/json)
    /// These responses are buffered, parsed, and can be intercepted
    Json,
    
    /// Server-Sent Events stream (text/event-stream)
    /// These responses are streamed and require special reconnection handling
    SseStream,
    
    /// Any other content type - passthrough without processing
    /// These are streamed directly without buffering or interception
    Passthrough,
}

impl ResponseMode {
    /// Detect response mode from Content-Type header using proper MIME parsing.
    /// 
    /// This replaces the old hyper_response.is_sse() method with a more
    /// comprehensive approach that handles all content types.
    /// 
    /// # Examples
    /// ```
    /// assert_eq!(ResponseMode::from_content_type("application/json"), ResponseMode::Json);
    /// assert_eq!(ResponseMode::from_content_type("text/event-stream"), ResponseMode::SseStream);
    /// assert_eq!(ResponseMode::from_content_type("text/plain"), ResponseMode::Passthrough);
    /// ```
    pub fn from_content_type(content_type: &str) -> Self {
        match content_type.parse::<Mime>() {
            Ok(mime) => {
                match (mime.type_(), mime.subtype()) {
                    (mime::APPLICATION, mime::JSON) => Self::Json,
                    (mime::TEXT, subtype) if subtype == "event-stream" => Self::SseStream,
                    _ => Self::Passthrough,
                }
            }
            Err(_) => Self::Passthrough, // Invalid MIME types are passed through
        }
    }
    
    /// Check if this mode requires streaming.
    /// 
    /// Currently only SSE requires streaming. When WebSocket support is added
    /// to the MCP spec, this will return true for WebSocket as well.
    pub fn is_streaming(&self) -> bool {
        matches!(self, Self::SseStream)
        // Future: matches!(self, Self::SseStream | Self::WebSocket)
    }
    
    /// Check if this mode supports message interception.
    /// 
    /// Only JSON and SSE responses can be meaningfully intercepted since
    /// we understand their format. Passthrough responses are opaque.
    pub fn supports_interception(&self) -> bool {
        matches!(self, Self::Json | Self::SseStream)
    }
    
    /// Check if this mode requires buffering the complete response.
    /// 
    /// JSON responses need to be buffered for parsing. SSE and Passthrough
    /// are streamed without buffering.
    pub fn requires_buffering(&self) -> bool {
        matches!(self, Self::Json)
    }
    
    /// Get the expected MIME type for this response mode.
    /// 
    /// Returns None for Passthrough since it represents multiple types.
    pub fn mime_type(&self) -> Option<&'static str> {
        match self {
            Self::Json => Some("application/json"),
            Self::SseStream => Some("text/event-stream"),
            Self::Passthrough => None,
        }
    }
}

impl Default for ResponseMode {
    /// Default to Passthrough for unknown/unset response modes
    fn default() -> Self {
        Self::Passthrough
    }
}

impl std::fmt::Display for ResponseMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
            Self::SseStream => write!(f, "SSE"),
            Self::Passthrough => write!(f, "Passthrough"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_response_mode_from_content_type() {
        // JSON detection
        assert_eq!(
            ResponseMode::from_content_type("application/json"),
            ResponseMode::Json
        );
        assert_eq!(
            ResponseMode::from_content_type("application/json; charset=utf-8"),
            ResponseMode::Json
        );
        
        // SSE detection
        assert_eq!(
            ResponseMode::from_content_type("text/event-stream"),
            ResponseMode::SseStream
        );
        assert_eq!(
            ResponseMode::from_content_type("text/event-stream; charset=utf-8"),
            ResponseMode::SseStream
        );
        
        // Passthrough for everything else
        assert_eq!(
            ResponseMode::from_content_type("text/plain"),
            ResponseMode::Passthrough
        );
        assert_eq!(
            ResponseMode::from_content_type("application/octet-stream"),
            ResponseMode::Passthrough
        );
        assert_eq!(
            ResponseMode::from_content_type("text/html"),
            ResponseMode::Passthrough
        );
        
        // Invalid MIME types default to Passthrough
        assert_eq!(
            ResponseMode::from_content_type("not-a-valid-mime"),
            ResponseMode::Passthrough
        );
        assert_eq!(
            ResponseMode::from_content_type(""),
            ResponseMode::Passthrough
        );
    }
    
    #[test]
    fn test_response_mode_properties() {
        // JSON properties
        assert!(!ResponseMode::Json.is_streaming());
        assert!(ResponseMode::Json.supports_interception());
        assert!(ResponseMode::Json.requires_buffering());
        assert_eq!(ResponseMode::Json.mime_type(), Some("application/json"));
        
        // SSE properties
        assert!(ResponseMode::SseStream.is_streaming());
        assert!(ResponseMode::SseStream.supports_interception());
        assert!(!ResponseMode::SseStream.requires_buffering());
        assert_eq!(ResponseMode::SseStream.mime_type(), Some("text/event-stream"));
        
        // Passthrough properties
        assert!(!ResponseMode::Passthrough.is_streaming());
        assert!(!ResponseMode::Passthrough.supports_interception());
        assert!(!ResponseMode::Passthrough.requires_buffering());
        assert_eq!(ResponseMode::Passthrough.mime_type(), None);
    }
    
    #[test]
    fn test_response_mode_serialization() {
        // Verify serialization for distributed storage compatibility
        for mode in [ResponseMode::Json, ResponseMode::SseStream, ResponseMode::Passthrough] {
            let serialized = serde_json::to_string(&mode).unwrap();
            let deserialized: ResponseMode = serde_json::from_str(&serialized).unwrap();
            assert_eq!(mode, deserialized);
        }
    }
    
    #[test]
    fn test_response_mode_display() {
        assert_eq!(ResponseMode::Json.to_string(), "JSON");
        assert_eq!(ResponseMode::SseStream.to_string(), "SSE");
        assert_eq!(ResponseMode::Passthrough.to_string(), "Passthrough");
    }
}
```

### Step 4: Implement ClientCapabilities Bitflags (15 min)

Create `src/transport/core/capabilities.rs`:

```rust
//! Client capability tracking using bitflags
//! 
//! This replaces the proposed ClientCapabilities struct with boolean fields
//! with an efficient bitflags implementation. This provides type-safe capability
//! tracking with minimal memory overhead and good const support.
//!
//! Design rationale: analysis/implementation-recommendations.md

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use super::ResponseMode;

bitflags! {
    /// Client capabilities for content negotiation and feature support.
    /// 
    /// These flags track what a client can accept and what features it supports.
    /// They're derived from transport type at session creation but can be updated
    /// based on client headers or protocol negotiation.
    /// 
    /// Note: We use bitflags instead of enumflags2 for better const support
    /// and serde integration (see analysis/implementation-recommendations.md)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ClientCapabilities: u32 {
        /// Client accepts JSON responses (application/json)
        const ACCEPTS_JSON = 0b00000001;
        
        /// Client accepts Server-Sent Events (text/event-stream)
        const ACCEPTS_SSE = 0b00000010;
        
        /// Client accepts binary/passthrough content (application/octet-stream, */*)
        const ACCEPTS_BINARY = 0b00000100;
        
        /// Client supports response compression (gzip, deflate)
        const SUPPORTS_COMPRESSION = 0b00001000;
        
        /// Client supports batch requests (MCP batch messages)
        const SUPPORTS_BATCH = 0b00010000;
        
        /// Client supports WebSocket upgrade (future - when MCP spec adds support)
        const SUPPORTS_WEBSOCKET = 0b00100000;
        
        /// Client supports request cancellation
        const SUPPORTS_CANCELLATION = 0b01000000;
        
        /// Client supports progress notifications
        const SUPPORTS_PROGRESS = 0b10000000;
        
        // Convenience combinations
        
        /// Standard capabilities for JSON-only clients (typical stdio transport)
        const STANDARD = Self::ACCEPTS_JSON.bits();
        
        /// Streaming capabilities for SSE-capable clients (typical HTTP transport)
        const STREAMING = Self::ACCEPTS_JSON.bits() | Self::ACCEPTS_SSE.bits();
        
        /// Full capabilities for all current content types
        const FULL = Self::ACCEPTS_JSON.bits() 
                   | Self::ACCEPTS_SSE.bits() 
                   | Self::ACCEPTS_BINARY.bits();
        
        /// All features enabled (for testing)
        const ALL = Self::FULL.bits()
                  | Self::SUPPORTS_COMPRESSION.bits()
                  | Self::SUPPORTS_BATCH.bits()
                  | Self::SUPPORTS_CANCELLATION.bits()
                  | Self::SUPPORTS_PROGRESS.bits();
    }
}

impl ClientCapabilities {
    /// Check if client accepts a specific response mode.
    /// 
    /// This is used during response routing to ensure we only send
    /// response formats the client can handle.
    pub fn accepts_response_mode(&self, mode: ResponseMode) -> bool {
        match mode {
            ResponseMode::Json => self.contains(Self::ACCEPTS_JSON),
            ResponseMode::SseStream => self.contains(Self::ACCEPTS_SSE),
            ResponseMode::Passthrough => {
                // Passthrough is accepted if client accepts binary OR has no specific requirements
                self.contains(Self::ACCEPTS_BINARY) || self.is_empty()
            }
        }
    }
    
    /// Create capabilities from an HTTP Accept header.
    /// 
    /// Parses the Accept header to determine what content types the client accepts.
    /// This is used by the reverse proxy to understand client capabilities.
    /// 
    /// # Examples
    /// ```
    /// let caps = ClientCapabilities::from_accept_header("application/json, text/event-stream");
    /// assert!(caps.contains(ClientCapabilities::ACCEPTS_JSON));
    /// assert!(caps.contains(ClientCapabilities::ACCEPTS_SSE));
    /// ```
    pub fn from_accept_header(accept: &str) -> Self {
        let mut caps = ClientCapabilities::empty();
        
        // Check for specific MIME types
        if accept.contains("application/json") {
            caps |= Self::ACCEPTS_JSON;
        }
        if accept.contains("text/event-stream") {
            caps |= Self::ACCEPTS_SSE;
        }
        if accept.contains("application/octet-stream") || accept.contains("*/*") {
            caps |= Self::ACCEPTS_BINARY;
        }
        
        // If no specific types found but header exists, assume standard
        if caps.is_empty() && !accept.is_empty() {
            caps = Self::STANDARD;
        }
        
        caps
    }
    
    /// Generate an HTTP Accept header from capabilities.
    /// 
    /// This is used when making upstream requests to communicate what
    /// response formats we can handle.
    pub fn to_accept_header(&self) -> String {
        let mut types = Vec::new();
        
        if self.contains(Self::ACCEPTS_JSON) {
            types.push("application/json");
        }
        if self.contains(Self::ACCEPTS_SSE) {
            types.push("text/event-stream");
        }
        if self.contains(Self::ACCEPTS_BINARY) {
            types.push("application/octet-stream");
        }
        
        if types.is_empty() {
            "*/*".to_string()
        } else {
            types.join(", ")
        }
    }
    
    /// Create capabilities appropriate for a transport type.
    /// 
    /// This provides sensible defaults based on the transport mechanism.
    pub fn from_transport_type(transport_type: crate::transport::TransportType) -> Self {
        use crate::transport::TransportType;
        
        match transport_type {
            TransportType::Stdio => Self::STANDARD,
            TransportType::Sse | TransportType::Http => Self::STREAMING,
            // When we rename Sse to StreamableHttp:
            // TransportType::StreamableHttp => Self::STREAMING,
        }
    }
    
    /// Check if client has any streaming capabilities
    pub fn supports_streaming(&self) -> bool {
        self.contains(Self::ACCEPTS_SSE) || self.contains(Self::SUPPORTS_WEBSOCKET)
    }
}

impl Default for ClientCapabilities {
    /// Default to standard JSON-only capabilities
    fn default() -> Self {
        Self::STANDARD
    }
}

impl std::fmt::Display for ClientCapabilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "None")
        } else if *self == Self::STANDARD {
            write!(f, "Standard")
        } else if *self == Self::STREAMING {
            write!(f, "Streaming")
        } else if *self == Self::FULL {
            write!(f, "Full")
        } else {
            // Show individual flags
            write!(f, "{:?}", self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_capabilities_combinations() {
        // Test predefined combinations
        assert!(ClientCapabilities::STANDARD.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(!ClientCapabilities::STANDARD.contains(ClientCapabilities::ACCEPTS_SSE));
        
        assert!(ClientCapabilities::STREAMING.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(ClientCapabilities::STREAMING.contains(ClientCapabilities::ACCEPTS_SSE));
        
        assert!(ClientCapabilities::FULL.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(ClientCapabilities::FULL.contains(ClientCapabilities::ACCEPTS_SSE));
        assert!(ClientCapabilities::FULL.contains(ClientCapabilities::ACCEPTS_BINARY));
    }
    
    #[test]
    fn test_accepts_response_mode() {
        let standard = ClientCapabilities::STANDARD;
        assert!(standard.accepts_response_mode(ResponseMode::Json));
        assert!(!standard.accepts_response_mode(ResponseMode::SseStream));
        
        let streaming = ClientCapabilities::STREAMING;
        assert!(streaming.accepts_response_mode(ResponseMode::Json));
        assert!(streaming.accepts_response_mode(ResponseMode::SseStream));
        
        let empty = ClientCapabilities::empty();
        assert!(!empty.accepts_response_mode(ResponseMode::Json));
        assert!(empty.accepts_response_mode(ResponseMode::Passthrough)); // Passthrough allowed for empty
    }
    
    #[test]
    fn test_from_accept_header() {
        // Multiple types
        let caps = ClientCapabilities::from_accept_header("application/json, text/event-stream");
        assert!(caps.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(caps.contains(ClientCapabilities::ACCEPTS_SSE));
        assert!(!caps.contains(ClientCapabilities::ACCEPTS_BINARY));
        
        // Wildcard
        let caps = ClientCapabilities::from_accept_header("*/*");
        assert!(caps.contains(ClientCapabilities::ACCEPTS_BINARY));
        
        // With charset and q-values (should still work)
        let caps = ClientCapabilities::from_accept_header(
            "application/json; charset=utf-8, text/event-stream; q=0.8"
        );
        assert!(caps.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(caps.contains(ClientCapabilities::ACCEPTS_SSE));
    }
    
    #[test]
    fn test_to_accept_header() {
        assert_eq!(
            ClientCapabilities::STANDARD.to_accept_header(),
            "application/json"
        );
        
        assert_eq!(
            ClientCapabilities::STREAMING.to_accept_header(),
            "application/json, text/event-stream"
        );
        
        assert_eq!(
            ClientCapabilities::empty().to_accept_header(),
            "*/*"
        );
    }
    
    #[test]
    fn test_bitwise_operations() {
        let mut caps = ClientCapabilities::empty();
        
        // Add capabilities
        caps |= ClientCapabilities::ACCEPTS_JSON;
        assert!(caps.contains(ClientCapabilities::ACCEPTS_JSON));
        
        caps |= ClientCapabilities::ACCEPTS_SSE;
        assert!(caps.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(caps.contains(ClientCapabilities::ACCEPTS_SSE));
        
        // Remove capability
        caps &= !ClientCapabilities::ACCEPTS_JSON;
        assert!(!caps.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(caps.contains(ClientCapabilities::ACCEPTS_SSE));
        
        // Toggle capability
        caps ^= ClientCapabilities::ACCEPTS_JSON;
        assert!(caps.contains(ClientCapabilities::ACCEPTS_JSON));
    }
    
    #[test]
    fn test_serialization() {
        // Test that capabilities can be serialized for distributed storage
        for caps in [
            ClientCapabilities::empty(),
            ClientCapabilities::STANDARD,
            ClientCapabilities::STREAMING,
            ClientCapabilities::FULL,
            ClientCapabilities::ALL,
        ] {
            let serialized = serde_json::to_string(&caps).unwrap();
            let deserialized: ClientCapabilities = serde_json::from_str(&serialized).unwrap();
            assert_eq!(caps, deserialized);
        }
    }
}
```

### Step 5: Update Transport Module Exports (5 min)

Update `src/transport/mod.rs` to include core module:

```rust
// Add near the top with other module declarations
pub mod core;

// Re-export commonly used types (add to existing re-exports)
pub use core::{ResponseMode, ClientCapabilities};
```

### Step 6: Build and Test (10 min)

```bash
# Full build to catch any issues
cargo build

# Run tests for new modules specifically
cargo test transport::core::response_mode -- --nocapture
cargo test transport::core::capabilities -- --nocapture

# Check for clippy warnings (MUST pass before commit)
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt

# Run quick test suite to ensure nothing broke
cargo test --lib
```

### Step 7: Verify Integration Points (5 min)

Check that the new types are accessible where needed:

```bash
# Verify imports work
echo "use shadowcat::transport::{ResponseMode, ClientCapabilities};" | \
  cargo check --lib 2>&1 | grep -q error && echo "Import failed" || echo "Import successful"

# Check serialization in a quick test
cat > /tmp/test_serialize.rs << 'EOF'
use shadowcat::transport::{ResponseMode, ClientCapabilities};

fn main() {
    let mode = ResponseMode::SseStream;
    println!("Mode: {}", serde_json::to_string(&mode).unwrap());
    
    let caps = ClientCapabilities::STREAMING;
    println!("Caps: {}", serde_json::to_string(&caps).unwrap());
}
EOF

rustc --edition 2021 --extern shadowcat=target/debug/libshadowcat.rlib \
  --extern serde_json /tmp/test_serialize.rs -o /tmp/test_serialize && \
  /tmp/test_serialize
```

## Success Criteria Checklist

- [ ] ResponseMode enum created with exactly 3 variants (Json, SseStream, Passthrough)
- [ ] No Unknown, Binary, Text, or WebSocket variants (per design decisions)
- [ ] MIME parsing uses mime crate, not string contains
- [ ] ClientCapabilities uses bitflags, not boolean struct
- [ ] Both types derive Serialize/Deserialize for distributed storage
- [ ] Comprehensive unit tests covering all cases
- [ ] Display implementations for better debugging
- [ ] Default implementations provided
- [ ] Core module properly structured and exported
- [ ] Code compiles without any warnings
- [ ] All clippy checks pass
- [ ] All new tests pass
- [ ] No backward compatibility code (no is_sse() method)

## Common Issues and Solutions

1. **Module structure issues**
   - If `transport/core` doesn't exist, create it
   - Ensure mod.rs properly declares and exports submodules
   - Check that transport/mod.rs includes `pub mod core`

2. **Import errors**
   - Verify `use crate::transport::TransportType` in capabilities.rs
   - May need to adjust based on actual module structure

3. **Clippy warnings**
   - Run `cargo clippy --fix` for auto-fixable issues
   - Common: unnecessary return statements, redundant closures
   - Must fix all before committing

4. **Test failures**
   - MIME parsing is strict - ensure test cases use valid MIME types
   - Bitflags operations are bitwise - use |, &, ^ operators

## References to Design Documents

Throughout implementation, refer back to:
- **analysis/response-mode-investigation.md** - Why we need ResponseMode
- **analysis/implementation-recommendations.md** - Why bitflags over alternatives
- **analysis/design-decisions.md** - Architectural decisions and rationale
- **analysis/architecture-proposal.md** - Overall system design
- **analysis/distributed-storage-considerations.md** - Why Serialize/Deserialize

## Duration Estimate
**Total: 60 minutes**
- Environment setup: 5 min
- Module structure: 5 min
- ResponseMode implementation: 20 min
- ClientCapabilities implementation: 15 min
- Module exports: 5 min
- Build and test: 10 min

## Next Steps
After completing this task:
1. Commit changes: `git commit -m "feat(transport): add ResponseMode enum and ClientCapabilities bitflags"`
2. Proceed to B.1 to update Session structure
3. B.1 will use these types to replace is_sse_session

## Notes
- This is a pure addition - no existing code is modified yet
- These types are foundational for the rest of Phase B
- The real impact comes in B.1 and B.2 when we integrate these types
- Remember: No backward compatibility needed - do it right!

---

**Task Status**: Ready for implementation
**Prerequisites**: Phase A complete, shadowcat repo access
**Blocks**: B.1 (Session structure update)
**Reviewer**: Check against success criteria before moving to B.1