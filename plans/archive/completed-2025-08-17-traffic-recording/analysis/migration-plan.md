# Migration Plan: Traffic Recording Refactor

## Overview

This document provides a detailed, step-by-step plan for migrating from `TransportContext::Sse` to `TransportContext::Http` with `ResponseMode`, while adding raw wire data support to the recording layer.

**Key Principle**: Since shadowcat hasn't been released yet, we can make breaking changes without backward compatibility concerns. This simplifies our migration significantly.

## Migration Phases

### Phase 1: Consolidate SseEvent Types (2 hours)
**Goal**: Single canonical SseEvent type  
**Risk**: Low - Pure refactoring  
**Breaking**: No

#### Step 1.1: Identify All SseEvent Definitions
```bash
# Files to update:
src/transport/sse/event.rs          # Keep this one (canonical)
src/transport/outgoing/http.rs:52   # Remove duplicate struct
src/recorder/tape.rs:186            # Keep SseMetadata (different purpose)
```

#### Step 1.2: Update outgoing::http
```rust
// src/transport/outgoing/http.rs
// Remove lines 52-59 (duplicate SseEvent struct)
// Add import:
use crate::transport::sse::event::SseEvent;
```

#### Step 1.3: Run Tests
```bash
cargo test transport::outgoing
cargo test --lib
```

### Phase 2: Add ResponseMode Support (3 hours)
**Goal**: Add ResponseMode field to TransportContext::Http  
**Risk**: Medium - Requires careful updates  
**Breaking**: Yes - Changes TransportContext structure

#### Step 2.1: Update TransportContext
```rust
// src/mcp/types.rs
use crate::transport::core::response_mode::ResponseMode;

pub enum TransportContext {
    Stdio { 
        process_id: Option<u32>,
        command: Option<String>,
    },
    Http {
        method: String,
        path: String,
        headers: HashMap<String, String>,
        status_code: Option<u16>,
        remote_addr: Option<String>,
        response_mode: Option<ResponseMode>, // NEW FIELD
    },
    Sse {  // Keep temporarily for migration
        event_type: Option<String>,
        event_id: Option<String>,
        retry_ms: Option<u64>,
        headers: HashMap<String, String>,
    },
}
```

#### Step 2.2: Update All Http Context Creation Sites
```bash
# Find all TransportContext::Http creations
rg "TransportContext::Http \{" --type rust

# Update each to include:
response_mode: None,  // Or detect from headers
```

Key files to update:
- `src/transport/outgoing/http.rs`
- `src/transport/http/mod.rs`
- `src/proxy/reverse/legacy.rs`
- Tests in `src/mcp/types.rs`

#### Step 2.3: Update Http Transports to Set ResponseMode
```rust
// src/transport/outgoing/http.rs
let response_mode = ResponseMode::from_content_type(
    headers.get("content-type").map(|s| s.as_str()).unwrap_or("")
);

TransportContext::Http {
    // ... existing fields ...
    response_mode: Some(response_mode),
}
```

#### Step 2.4: Run Tests
```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

### Phase 3: Implement Raw Wire Data Support (4 hours)
**Goal**: Add infrastructure for passing raw data to recorder  
**Risk**: Low - Additive changes only  
**Breaking**: No

#### Step 3.1: Create RawWireData Types
```rust
// src/recorder/wire_data.rs (new file)
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RawWireData {
    pub bytes: Arc<Vec<u8>>,
    pub format: WireFormat,
    pub direction: DataDirection,
}

#[derive(Debug, Clone, Copy)]
pub enum WireFormat {
    Json,
    ServerSentEvent,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum DataDirection {
    ClientToServer,
    ServerToClient,
}
```

#### Step 3.2: Add to recorder module
```rust
// src/recorder/mod.rs
pub mod wire_data;
pub use wire_data::{RawWireData, WireFormat, DataDirection};
```

#### Step 3.3: Add record_frame_with_raw Method
```rust
// src/recorder/tape.rs
impl TapeRecorder {
    pub async fn record_frame_with_raw(
        &self,
        envelope: MessageEnvelope,
        raw_data: Option<RawWireData>,
    ) -> RecorderResult<()> {
        // Implementation from design doc
    }
}
```

#### Step 3.4: Add SSE Parsing Utilities
```rust
// src/recorder/sse_parser.rs (new file)
use crate::transport::sse::event::SseEvent;

pub fn parse_sse_event(bytes: &[u8]) -> Result<SseEvent> {
    // Parse SSE wire format
    // Can reuse logic from transport::sse
}
```

### Phase 4: Migrate Sse to Http Contexts (3 hours)
**Goal**: Replace all TransportContext::Sse usage  
**Risk**: High - Changes core message flow  
**Breaking**: Yes

#### Step 4.1: Find All Sse Context Usage
```bash
rg "TransportContext::Sse" --type rust

# Key files:
src/mcp/types.rs                     # Definition
src/recorder/session_recorder.rs:393 # Extraction
src/transport/sse/mod.rs            # Creation (probably)
```

#### Step 4.2: Update SSE Transport
```rust
// src/transport/sse/mod.rs (or wherever SSE contexts are created)
// Change from:
TransportContext::Sse {
    event_type: Some(event.event_type.clone()),
    event_id: event.id.clone(),
    retry_ms: event.retry,
    headers: headers.clone(),
}

// To:
TransportContext::Http {
    method: "GET".to_string(),
    path: self.path.clone(),
    headers: headers.clone(),
    status_code: Some(200),
    remote_addr: self.remote_addr.clone(),
    response_mode: Some(ResponseMode::SseStream),
}
```

#### Step 4.3: Update Recording Layer
```rust
// src/recorder/session_recorder.rs:393
// Remove the Sse match arm, handle via Http + ResponseMode
match &envelope.context.transport {
    TransportContext::Http { response_mode, .. } => {
        if matches!(response_mode, Some(ResponseMode::SseStream)) {
            // Extract SSE metadata from raw_data if available
            if let Some(raw) = raw_data {
                // Parse and extract SSE metadata
            }
        }
        // Regular HTTP metadata extraction
    }
    TransportContext::Stdio { .. } => {
        // Unchanged
    }
    // Remove Sse arm
}
```

#### Step 4.4: Fix Compilation Errors
```bash
cargo check
# Fix any remaining references to TransportContext::Sse
```

### Phase 5: Remove TransportContext::Sse (1 hour)
**Goal**: Complete removal of deprecated variant  
**Risk**: Low - If Phase 4 complete  
**Breaking**: Yes

#### Step 5.1: Remove Variant
```rust
// src/mcp/types.rs
pub enum TransportContext {
    Stdio { /* unchanged */ },
    Http { /* with response_mode */ },
    // DELETE Sse variant completely
}
```

#### Step 5.2: Fix Any Remaining Compilation Errors
```bash
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```

### Phase 6: Update Proxies to Use Raw Data (2 hours)
**Goal**: Pass raw wire data through proxy to recorder  
**Risk**: Medium - Changes proxy flow  
**Breaking**: No (backward compatible)

#### Step 6.1: Update Forward Proxy
```rust
// src/proxy/forward.rs
// Update to use record_frame_with_raw when available
if let Some(recorder) = &tape_recorder_c2s {
    // Try to get raw data from transport
    let raw_data = transport.last_raw_data(); // New method
    recorder.record_frame_with_raw(envelope.clone(), raw_data).await?;
}
```

#### Step 6.2: Update Reverse Proxy
```rust
// src/proxy/reverse/legacy.rs
// Similar updates for reverse proxy recording
```

### Phase 7: Testing and Validation (2 hours)
**Goal**: Ensure everything works correctly  
**Risk**: Low - Testing phase  
**Breaking**: No

#### Step 7.1: Unit Tests
```bash
# Run all unit tests
cargo test --lib

# Run specific transport tests
cargo test transport::
cargo test recorder::
```

#### Step 7.2: Integration Tests
```bash
# Run integration tests
cargo test --test integration_mcp
cargo test --test e2e_basic_integration_test
```

#### Step 7.3: Manual Testing
```bash
# Test with real MCP server
cd shadowcat
cargo run -- forward stdio -- npx @modelcontextprotocol/server-everything

# Test recording
cargo run -- record --output test.tape forward stdio -- npx @modelcontextprotocol/server-everything

# Test replay
cargo run -- replay test.tape
```

#### Step 7.4: Performance Testing
```bash
# Run benchmarks if available
cargo bench

# Manual performance test with large payloads
```

## File Change Summary

### Files to Create
- `src/recorder/wire_data.rs` - New RawWireData types
- `src/recorder/sse_parser.rs` - SSE parsing utilities

### Files to Modify Significantly
- `src/mcp/types.rs` - Remove Sse variant, add response_mode to Http
- `src/recorder/session_recorder.rs` - Update metadata extraction
- `src/recorder/tape.rs` - Add record_frame_with_raw
- `src/transport/sse/mod.rs` - Update context creation

### Files to Update (Minor)
- `src/transport/outgoing/http.rs` - Remove duplicate SseEvent, set response_mode
- `src/transport/http/mod.rs` - Set response_mode
- `src/proxy/forward.rs` - Use new recording method
- `src/proxy/reverse/legacy.rs` - Use new recording method

### Files to Delete
- None (all changes are modifications)

## Rollback Plan

Since we're not released yet, rollback is simple:

```bash
# If issues found during migration
git stash  # Save work in progress
git checkout main  # Return to stable state

# Or if committed
git revert HEAD~n  # Revert last n commits
```

## Risk Assessment

| Phase | Risk Level | Impact | Mitigation |
|-------|------------|--------|------------|
| 1. Consolidate SseEvent | Low | Minimal | Simple refactor, easy to revert |
| 2. Add ResponseMode | Medium | Compilation errors | Update sites incrementally |
| 3. Raw Wire Data | Low | None (additive) | New code, doesn't affect existing |
| 4. Migrate Sse→Http | High | Core functionality | Thorough testing at each step |
| 5. Remove Sse | Low | Compilation errors | Phase 4 handles migration |
| 6. Update Proxies | Medium | Recording might fail | Keep fallback to old method |
| 7. Testing | Low | None | Just validation |

## Success Metrics

- [ ] All tests passing (1300+ tests)
- [ ] No clippy warnings
- [ ] SSE recording/replay works correctly
- [ ] Performance unchanged (< 5% overhead)
- [ ] Clean architecture boundaries

## Timeline Estimate

- **Phase 1**: 2 hours (consolidate types)
- **Phase 2**: 3 hours (add ResponseMode)
- **Phase 3**: 4 hours (raw wire data)
- **Phase 4**: 3 hours (migrate contexts)
- **Phase 5**: 1 hour (remove Sse)
- **Phase 6**: 2 hours (update proxies)
- **Phase 7**: 2 hours (testing)

**Total**: 17 hours (2-3 days of work)

## Next Steps

1. Start with Phase 1 (consolidate SseEvent) - lowest risk
2. Proceed through phases sequentially
3. Run tests after each phase
4. Commit after each successful phase
5. Document any issues or deviations

## Notes

- No backward compatibility needed since shadowcat is unreleased
- Can make breaking changes freely
- Focus on clean architecture over compatibility
- This is the perfect time to fix these issues properly

---

**Document Version**: 1.0  
**Created**: 2025-08-17  
**Status**: Ready for implementation