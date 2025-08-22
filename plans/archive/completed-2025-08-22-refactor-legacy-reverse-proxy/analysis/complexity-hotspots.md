# Complexity Hotspots in legacy.rs

## Functions Exceeding 100 Lines

### 1. `handle_mcp_request` (550 lines, 1030-1580)
**Complexity Issues:**
- Massive function doing too many things
- Multiple nested conditionals
- Session management mixed with request processing
- Error handling scattered throughout
- Protocol negotiation logic embedded
- Authentication checks
- Recording/interception logic
- Transport selection and routing

**Responsibilities:**
- Extract MCP headers
- Session lookup/creation
- Version negotiation
- Authentication validation
- Message interception
- Transport routing (stdio vs HTTP)
- Response formatting
- Error response generation

### 2. `proxy_sse_from_upstream` (298 lines, 1849-2147)
**Complexity Issues:**
- Complex async stream handling
- Nested error handling
- Session state management
- Multiple exit conditions
- Retry logic embedded

**Responsibilities:**
- Upstream connection management
- SSE event streaming
- Session lifecycle tracking
- Error recovery
- Keep-alive handling
- Message transformation

### 3. `handle_mcp_sse_request` (162 lines, 1672-1834)
**Complexity Issues:**
- Dual transport handling (stdio/HTTP)
- Complex initialization flow
- Session management
- Error handling at multiple levels

**Responsibilities:**
- SSE connection setup
- Transport selection
- Session initialization
- Response streaming setup

### 4. `process_via_http` (147 lines, 2498-2645)
**Complexity Issues:**
- Complex HTTP client logic
- Multiple error conditions
- Header manipulation
- Response transformation

**Responsibilities:**
- HTTP request construction
- Header forwarding
- Response processing
- Error mapping

## Functions 50-100 Lines

### 5. `proxy_sse_response` (83 lines, 1585-1670)
- SSE response streaming
- Event transformation
- Error handling

### 6. `handle_metrics` (81 lines, 2715-2796)
- Metrics collection
- JSON formatting
- State aggregation

### 7. `get_or_create_session` (80 lines, 2202-2282)
- Session lookup
- Session creation
- Initialization handling

### 8. `process_via_stdio_pooled` (67 lines, 2318-2385)
- Connection pool management
- Subprocess communication
- Error recovery

### 9. `create_router` (65 lines, 961-1027)
- Route configuration
- Middleware setup
- Handler registration

### 10. `process_via_http_hyper` (61 lines, 2392-2453)
- HTTP client operations
- Response handling

## Deeply Nested Code (>3 levels)

### Session Initialization Logic
Location: Lines 1200-1400 in `handle_mcp_request`
```
if session.is_new() {
    if let Some(init_request) = ... {
        match transport_type {
            TransportType::Stdio => {
                if let Ok(pool) = ... {
                    // 4+ levels deep
                }
            }
        }
    }
}
```

### Error Handling Cascades
Multiple locations with:
```
match result {
    Ok(value) => {
        match process(value) {
            Ok(processed) => {
                if condition {
                    // Deep nesting
                }
            }
        }
    }
}
```

## High Cyclomatic Complexity Areas

### 1. Version Negotiation (Lines 1100-1200)
- Multiple version checks
- Fallback logic
- Compatibility matrix
- ~8 different code paths

### 2. Transport Selection (Lines 1400-1500)
- Transport type matching
- Fallback mechanisms
- Pool availability checks
- ~6 different paths

### 3. Response Mode Selection (Lines 1500-1580)
- Content type checking
- Mode determination
- Response formatting
- ~5 different paths

## Code Duplication

### 1. Session Header Extraction
Duplicated in 3 places:
- `handle_mcp_request`
- `handle_mcp_sse_request`
- `get_or_create_session`

### 2. Error Response Generation
Similar error response patterns in:
- `handle_mcp_request` (4 locations)
- `process_via_http` (2 locations)
- `handle_admin_request` (1 location)

### 3. Transport Processing
Similar patterns for:
- `process_via_stdio_pooled`
- `process_via_http`
- `process_via_http_hyper`

## Mixed Responsibilities

### 1. `handle_mcp_request`
Mixes:
- HTTP handling
- Protocol processing
- Session management
- Authentication
- Transport routing
- Response formatting

### 2. `ReverseProxyServer` struct methods
Mixes:
- Server lifecycle
- Configuration
- Router setup
- Shutdown handling

### 3. Configuration structs
Mixes:
- Data storage
- Validation logic
- Default generation
- Builder patterns

## Refactoring Priorities

### Critical (P0)
1. **Break down `handle_mcp_request`** into:
   - Request parsing
   - Session handling
   - Protocol negotiation
   - Transport routing
   - Response generation

2. **Simplify `proxy_sse_from_upstream`** into:
   - Connection management
   - Stream processing
   - Error handling

### High (P1)
3. **Extract duplicate code** into utilities:
   - Session header parsing
   - Error response generation
   - Transport processing patterns

4. **Reduce nesting** through:
   - Early returns
   - Extract method refactoring
   - Error propagation improvements

### Medium (P2)
5. **Separate concerns** in configuration:
   - Pure data structures
   - Validation functions
   - Builder implementations

6. **Improve testability** by:
   - Extracting pure functions
   - Reducing I/O dependencies
   - Creating mockable interfaces

## Metrics Summary

- **Total Lines**: 3,682
- **Functions > 100 lines**: 4
- **Functions 50-100 lines**: 6
- **Maximum nesting depth**: 5+ levels
- **Duplicated patterns**: ~8 instances
- **Mixed responsibility areas**: 5 major sections

## Recommended Module Structure

To address these hotspots:

1. **request_handler.rs** (~200 lines)
   - Request parsing
   - Header extraction
   - Basic validation

2. **session_handler.rs** (~150 lines)
   - Session lookup
   - Session creation
   - State management

3. **protocol_handler.rs** (~200 lines)
   - Version negotiation
   - Protocol validation
   - Message processing

4. **transport_router.rs** (~150 lines)
   - Transport selection
   - Routing logic
   - Pool management

5. **sse_handler.rs** (~400 lines)
   - SSE connection setup
   - Stream processing
   - Event handling

6. **response_handler.rs** (~100 lines)
   - Response formatting
   - Error responses
   - Header construction