# External Dependencies Matrix

## Component Dependency Analysis

| Component | External Crate Deps | Internal Module Deps | Can Extract? | Extraction Priority | Notes |
|-----------|-------------------|---------------------|--------------|-------------------|--------|
| **Config Types** | serde, serde_json | TransportType from mcp | ✅ Yes - Leaf | P0 (First) | No behavior, pure data |
| **Error Types** | std, axum::http | None | ✅ Yes - Leaf | P0 (First) | Self-contained |
| **Metrics** | std::sync::atomic | None | ✅ Yes - Leaf | P0 (First) | Simple atomics |
| **Load Balancing Enum** | serde | None | ✅ Yes - Leaf | P0 (First) | Pure enum |
| **Builders** | None | Config types | ✅ Yes | P1 (Second) | After configs |
| **Validators** | None | Config types, Error types | ✅ Yes | P1 (Second) | After configs/errors |
| **Admin Handlers** | axum, serde_json | Metrics, AppState | ⚠️ Maybe | P2 (Third) | Needs metrics interface |
| **Session Helpers** | tokio, uuid | SessionManager, Session | ⚠️ Maybe | P2 (Third) | Needs trait abstraction |
| **Transport Router** | None | Transport types, pools | ⚠️ Maybe | P2 (Third) | Complex dependencies |
| **Process Functions** | tokio, hyper | Transport, pools | ❌ Difficult | P3 (Fourth) | Heavy integration |
| **SSE Handlers** | axum::sse, tokio_stream | Multiple | ❌ Difficult | P3 (Fourth) | Complex streaming |
| **Request Handlers** | axum, all | Everything | ❌ Last | P4 (Final) | Core orchestration |
| **Server Setup** | axum, tower, tokio | Everything | ❌ Last | P4 (Final) | Initialization logic |
| **AppState** | Arc, multiple | All internal | ❌ Last | P4 (Final) | Central coordination |

## Detailed Component Analysis

### Configs (Lines 57-280)

**External Dependencies:**
```rust
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
```

**Internal Dependencies:**
```rust
use crate::mcp::TransportType;
```

**Extraction Feasibility:** ✅ Easy
- Move to `config.rs`
- Keep TransportType import
- No behavioral dependencies

### Metrics (Lines 356-401)

**External Dependencies:**
```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
```

**Internal Dependencies:** None

**Extraction Feasibility:** ✅ Easy
- Move to `metrics.rs`
- Self-contained implementation
- Optional feature flag possible

### Admin Endpoints (Lines 2706-2826)

**External Dependencies:**
```rust
use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;
```

**Internal Dependencies:**
```rust
use AppState;  // For metrics access
use AuthContext;  // From request extensions
```

**Extraction Feasibility:** ⚠️ Medium
- Move to `admin.rs`
- Need metrics trait
- Could mock AppState

### Session Management (Lines 2202-2282)

**External Dependencies:**
```rust
use tokio;
use uuid::Uuid;
```

**Internal Dependencies:**
```rust
use crate::session::{Session, SessionManager};
use crate::mcp::SessionId;
```

**Extraction Feasibility:** ⚠️ Medium
- Move to `session_handler.rs`
- Needs SessionManager trait
- Async operations throughout

### Transport Processing (Lines 2318-2645)

**External Dependencies:**
```rust
use tokio;
use hyper::{Client, Request, Body};
```

**Internal Dependencies:**
```rust
use crate::transport::{OutgoingTransport, SubprocessOutgoing};
use crate::proxy::pool::ConnectionPool;
```

**Extraction Feasibility:** ❌ Difficult
- Split into `transport_handler.rs`
- Heavy transport dependencies
- Pool management complexity

### SSE Handlers (Lines 1585-2147)

**External Dependencies:**
```rust
use axum::response::sse::{Event, Sse};
use tokio_stream::StreamExt;
use futures::stream;
```

**Internal Dependencies:**
```rust
use crate::transport::sse::SseEvent;
use SessionManager, OutgoingTransport;
```

**Extraction Feasibility:** ❌ Very Difficult
- Move to `sse.rs`
- Complex async streaming
- Multiple integration points

### Main Request Handler (Lines 1030-1580)

**External Dependencies:**
```rust
use axum::{extract, http, Json};
use All framework types;
```

**Internal Dependencies:**
```rust
use Everything;  // All internal modules
```

**Extraction Feasibility:** ❌ Last to Extract
- Core orchestration logic
- Depends on all components
- Must be refactored, not just moved

## Extraction Order Strategy

### Wave 1: Pure Data (No Dependencies)
```
1. errors.rs       - Error types
2. config.rs       - Configuration structs
3. metrics.rs      - Metrics collection
4. constants.rs    - Shared constants
```

### Wave 2: Simple Logic (Minimal Dependencies)
```
5. builders.rs     - Config builders
6. validators.rs   - Validation logic
7. helpers.rs      - Utility functions
```

### Wave 3: Feature Modules (Some Dependencies)
```
8. admin.rs        - Admin endpoints
9. session_handler.rs - Session helpers
10. response.rs    - Response formatting
```

### Wave 4: Complex Integration (Heavy Dependencies)
```
11. transport_handler.rs - Transport routing
12. sse.rs         - SSE streaming
13. processing.rs  - Message processing
```

### Wave 5: Core Orchestration (All Dependencies)
```
14. handlers.rs    - Request handlers
15. server.rs      - Server setup
16. state.rs       - AppState management
```

## Dependency Injection Requirements

### Traits Needed
```rust
// For testing and modularity
trait SessionStore {
    async fn get_or_create(&self, id: SessionId) -> Result<Session>;
}

trait MetricsCollector {
    fn record_request(&self, duration: Duration, success: bool);
}

trait MessageInterceptor {
    async fn intercept(&self, msg: Message) -> InterceptAction;
}

trait TransportPool {
    async fn acquire(&self) -> Result<Transport>;
}
```

### Interface Segregation
- Split AppState into focused interfaces
- Pass only needed capabilities to functions
- Enable unit testing with mocks

## Risk Assessment

### Low Risk Extractions
- Config types (data only)
- Error types (no dependencies)
- Metrics (atomic operations)
- Constants (values only)

### Medium Risk Extractions  
- Admin endpoints (needs interfaces)
- Session helpers (async complexity)
- Response builders (error handling)

### High Risk Extractions
- Transport processing (pool management)
- SSE handlers (streaming complexity)
- Request handlers (orchestration)
- Server setup (initialization)

## Success Metrics

### Phase 1 Success
- [ ] All config types extracted
- [ ] Error types in separate module
- [ ] Tests still passing
- [ ] No functionality change

### Phase 2 Success
- [ ] Helper functions extracted
- [ ] Builders in separate files
- [ ] Reduced file size by 30%

### Phase 3 Success
- [ ] Feature modules created
- [ ] Admin UI separated
- [ ] File under 2000 lines

### Phase 4 Success
- [ ] Core handlers refactored
- [ ] All modules under 500 lines
- [ ] Clean module boundaries
- [ ] Improved testability