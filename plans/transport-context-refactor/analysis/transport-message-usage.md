# TransportMessage Usage Analysis

## Summary Statistics
- Total files: 34
- Total occurrences: 330
- Import statements: 33
- Actual usage (TransportMessage::): 194
- Pattern matching: 21 occurrences
- Active usage files: 34 (100%)
- Critical path files: 8

## Usage Categories

### Category 1: Import-Only (0 files)
All files that import TransportMessage actively use it - no dead imports found.

### Category 2: Message Creators (15 files)
Files that construct TransportMessage instances.

**High Priority** (transport implementations):
- `transport/stdio.rs` (16 occurrences) - Creates messages from stdio streams
- `transport/http.rs` (24 occurrences) - Creates messages from HTTP requests
- `transport/http_mcp.rs` (29 occurrences) - MCP-specific HTTP handling
- `transport/replay.rs` (12 occurrences) - Creates messages from recorded tapes
- `transport/sse/manager.rs` (5 occurrences) - SSE stream handling

**Medium Priority** (proxy/session):
- `proxy/forward.rs` (22 occurrences) - Forward proxy message handling
- `proxy/reverse.rs` (22 occurrences) - Reverse proxy with auth
- `session/manager.rs` (44 occurrences) - **CRITICAL**: Session lifecycle and routing
- `main.rs` (24 occurrences) - CLI entry points and testing

### Category 3: Message Consumers (20 files)
Files that match on or destructure TransportMessage.

**Critical Consumers**:
- `session/manager.rs` - Extracts IDs, methods, tracks sessions
- `interceptor/engine.rs` (8 occurrences) - Pattern matches for interception
- `interceptor/actions.rs` (22 occurrences) - Modifies messages
- `recorder/tape.rs` (7 occurrences) - Records messages to storage
- `recorder/format.rs` (9 occurrences) - Formats for display

**Analysis Tools**:
- `mcp/early_parser.rs` (10 occurrences) - Parses and categorizes
- `cli/tape.rs` (6 occurrences) - Tape management and replay

### Category 4: Message Transformers (8 files)
Files that modify or convert TransportMessage.

**Critical Transformers**:
- `interceptor/actions.rs` - Modifies message content
- `mcp/early_parser.rs` - Converts between formats
- `transport/http_mcp.rs` - Converts to/from HTTP format
- `proxy/forward.rs` - Forwards with potential modification
- `proxy/reverse.rs` - Adds auth headers and transforms

### Category 5: Transport Implementations (5 files)
The actual transport layer implementations.

- `transport/stdio.rs` - Process-based transport
- `transport/http.rs` - HTTP request/response
- `transport/http_mcp.rs` - MCP-specific HTTP
- `transport/sse/manager.rs` - Server-sent events
- `transport/replay.rs` - Replay from recordings

## Critical Paths

### Path 1: Request Forwarding
Flow: `Client → Transport → Session Manager → Interceptor → Proxy → Transport → Server`
- Files: `transport/*.rs` → `session/manager.rs` → `interceptor/engine.rs` → `proxy/forward.rs`
- Impact: Every proxied request
- Risk: **HIGH** - Core functionality

### Path 2: Session Management
Flow: `Message → Session Manager → Frame Recording → Session Store`
- Files: `session/manager.rs` → `session/store.rs` → `recorder/tape.rs`
- Impact: All messages require session tracking
- Risk: **HIGH** - Stateful operations

### Path 3: Notification Routing
Flow: `Notification → Session Manager → Direction Detection → Routing`
- Current Issue: **No explicit direction tracking in TransportMessage**
- Files: `session/manager.rs`, all transport implementations
- Risk: **CRITICAL** - Notifications can't be properly routed

## Existing Metadata Patterns

### Direction Handling
- **Pattern**: `Direction` enum exists (`ClientToServer`, `ServerToClient`)
- **Location**: Tracked in `Frame` struct, not in `TransportMessage`
- **Issue**: Direction is determined externally, not carried with message
- **Files**: `session/manager.rs`, `session/store.rs`

### Session Tracking
- **Pattern**: `SessionId` tracked separately from messages
- **Location**: `session/manager.rs` maintains session state
- **Method**: Extracts session from initialize messages or headers
- **Issue**: Session context not attached to messages

### Headers Handling
- **HTTP Transport**: Headers in `http_mcp.rs` via `McpHeaders` struct
- **Pattern**: Headers extracted at transport layer, not propagated
- **Files**: `transport/http_mcp.rs`, `transport/http.rs`

### Notification Direction
- **Current approach**: Direction inferred from transport edge
- **Issues**: 
  - No way to track bidirectional notifications
  - Can't determine if notification is client→server or server→client
  - Proxy can't route notifications correctly

## Migration Recommendations

### Phase 1: Core Infrastructure (10 hours)
1. `transport/mod.rs` - Define `MessageEnvelope` and `TransportContext`
2. `transport/envelope.rs` - New types and conversion traits
3. Add compatibility layer for existing `TransportMessage`

### Phase 2: Transport Implementations (9 hours)
1. `transport/stdio.rs` - Add context creation
2. `transport/http_mcp.rs` - Attach HTTP headers to context
3. `transport/sse/manager.rs` - Add SSE event context
4. `transport/http.rs` - Basic HTTP context
5. `transport/replay.rs` - Preserve recorded context

### Phase 3: Session Layer (8 hours)
1. `session/manager.rs` - Update to use `MessageEnvelope`
2. `session/store.rs` - Store context with frames
3. Add direction detection logic

### Phase 4: Proxy Layer (7 hours)
1. `proxy/forward.rs` - Use envelope for routing
2. `proxy/reverse.rs` - Preserve context through auth
3. `interceptor/engine.rs` - Process envelopes
4. `interceptor/actions.rs` - Modify with context preservation

### Phase 5: Remaining Components (6 hours)
1. `recorder/tape.rs` - Record full context
2. `recorder/format.rs` - Display context info
3. `cli/tape.rs` - Handle context in replay
4. Update tests

## Key Findings

1. **No Dead Imports**: All 34 files actively use TransportMessage
2. **Session Manager is Central**: 44 occurrences, manages all routing
3. **Direction is External**: Currently tracked in Frame, not Message
4. **Headers are Lost**: HTTP headers extracted but not propagated
5. **Notification Routing Broken**: Can't determine direction for proper routing
6. **Pattern Matching Heavy**: 21 locations use pattern matching - will need updates

## Risk Assessment

| Component | Risk | Mitigation |
|-----------|------|------------|
| Session Manager | **CRITICAL** | Gradual migration with compatibility layer |
| Transport Layer | **HIGH** | Update one transport at a time |
| Proxy Forwarding | **HIGH** | Extensive testing required |
| Interceptors | **MEDIUM** | Can adapt to both old/new formats |
| CLI/Tools | **LOW** | Internal only, can update freely |

## Compatibility Requirements

### Must Maintain
- Wire protocol format (JSON-RPC)
- CLI command interface
- Tape format (with versioning)
- Public API contracts

### Can Break (Internal)
- Internal type signatures
- Module interfaces
- Test helpers
- Debug representations