# Current Architecture Analysis

## File Overview
- **File**: `src/proxy/reverse.rs`
- **Size**: 3,482 lines
- **Primary Purpose**: Reverse proxy server for MCP protocol over HTTP with SSE support

## Module Structure

### Public Interface

#### Structs
- `ReverseProxyServer` - Main server struct with bind, router, session manager, config
- `ReverseUpstreamConfig` - Configuration for upstream servers
- `ReverseUpstreamHealthCheckConfig` - Health check settings for upstreams  
- `ReverseUpstreamPoolConfig` - Connection pool configuration
- `ReverseProxyConfig` - Main proxy configuration
- `ReverseSessionConfig` - Session timeout and cleanup settings
- `ReverseProxyMetrics` - Performance and usage metrics

#### Enums
- `ReverseLoadBalancingStrategy` - Load balancing strategies (RoundRobin, WeightedRoundRobin, etc.)

#### Public Methods (on ReverseProxyServer)
- `new()` - Constructor (lines 690-904)
- `serve()` - Main serve loop with shutdown (lines 872-906)

### Internal Organization

#### Core Server Setup (lines 1-909)
- Imports and dependencies (lines 1-42)
- Struct definitions (lines 44-286)
- Default implementations (lines 106-286)
- Config validation (lines 287-354)
- Metrics implementation (lines 355-683)
- Server construction and lifecycle (lines 684-907)

#### Router and Middleware (lines 910-975)
- `create_router()` - Sets up Axum routes with middleware layers
- Authentication middleware integration
- Rate limiting middleware
- CORS and tracing layers

#### Request Handlers (lines 976-2067)
- `handle_mcp_request()` (lines 979-1545) - **567 lines!** Main POST handler
  - Batch detection and rejection
  - Session management
  - Interceptor chain processing
  - Upstream routing
  - Response handling
- `proxy_sse_response()` (lines 1549-1632) - Stream SSE without buffering
- `handle_mcp_sse_request()` (lines 1636-1780) - GET handler for SSE
- `proxy_sse_from_upstream()` (lines 1795-2067) - **272 lines!** SSE proxy logic

#### Helper Functions (lines 2069-2513)
- `select_upstream()` - Round-robin upstream selection
- `validate_content_type()` - Content-type validation
- `parse_session_id()` - Session ID parsing
- `get_or_create_session()` - Session management (81 lines)
- `process_message()` - Message routing to upstream
- `process_via_stdio_pooled()` - Stdio transport handling
- `process_via_http()` - **143 lines!** HTTP transport with SSE detection
- `validate_mcp_response_headers()` - Response validation
- `echo_response()` - Test echo handler

#### Admin Endpoints (lines 2514-3482)
- `handle_health()` - Health check endpoint
- `handle_metrics()` - Metrics endpoint (82 lines)
- `handle_admin_request()` - **876 lines!** Admin interface

## Critical Issues Identified

### 1. SSE Buffering Bug (THE MAIN ISSUE)
**Location**: `process_via_http()` lines 2312-2454

The bug occurs because:
1. Line 2423: Detects SSE content-type
2. Line 2431: Returns `SseStreamingRequired` error
3. Lines 1289-1311: Catches error and makes DUPLICATE request
4. Problem: The original response stream is lost, creating inefficiency

**Root Cause**: Trying to determine response type AFTER consuming headers but BEFORE reading body. The function signature forces returning a `ProtocolMessage`, but SSE streams can't be converted to that.

### 2. Function Size Problems
- `handle_mcp_request()`: 567 lines (needs splitting)
- `handle_admin_request()`: 876 lines (massive, needs major refactor)
- `proxy_sse_from_upstream()`: 272 lines
- `process_via_http()`: 143 lines

### 3. Duplicate Request Anti-Pattern
Lines 1293-1311 show the problematic workaround:
- Makes initial request that detects SSE
- Throws away response
- Makes SECOND identical request for streaming
- Wastes resources and adds latency

## Execution Flows

### JSON Request Path
1. Client → `handle_mcp_request()`
2. Parse and validate headers
3. Session management
4. Interceptor chain (request)
5. Upstream selection
6. `process_via_http()` or `process_via_stdio_pooled()`
7. Interceptor chain (response)
8. Return JSON response

### SSE Request Path (GET)
1. Client → `handle_mcp_sse_request()`
2. Validate Accept header
3. Session management  
4. Create SSE stream channel
5. Spawn `proxy_sse_from_upstream()` task
6. Return SSE response immediately

### SSE Response Path (POST returning SSE)
1. Client → `handle_mcp_request()`
2. Normal processing until `process_via_http()`
3. Detects SSE content-type
4. Returns `SseStreamingRequired` error
5. Catches error, makes duplicate request
6. Calls `proxy_sse_response()`
7. Streams SSE events

## Module Boundaries for Refactoring

Suggested module split (~500 lines each):

1. **config.rs** - All config structs and validation (lines 44-354)
2. **metrics.rs** - Metrics implementation (lines 355-683)
3. **server.rs** - Core server setup and routing (lines 684-975)
4. **handlers/json.rs** - JSON request handling logic
5. **handlers/sse.rs** - SSE handlers and streaming
6. **handlers/admin.rs** - Admin endpoints (needs major refactor)
7. **upstream.rs** - Upstream selection and connection
8. **transport.rs** - Transport-specific processing (stdio, http)
9. **interceptors.rs** - Interceptor integration logic

## Key Observations

1. **Monolithic Design**: Everything in one file makes it hard to maintain
2. **Mixed Concerns**: Business logic, transport handling, admin UI all together
3. **Duplicate Logic**: Similar patterns repeated for request/response interception
4. **SSE Afterthought**: SSE support added on top rather than designed in
5. **No Streaming Abstraction**: Functions assume buffered responses, incompatible with SSE
6. **Session Management**: Tightly coupled throughout, could be abstracted
7. **Error Handling**: Inconsistent patterns, some errors used for control flow

## Refactoring Opportunities

1. **Extract trait for upstream communication** with JSON and SSE variants
2. **Separate transport concerns** from business logic
3. **Create streaming-first abstractions** that work for both JSON and SSE
4. **Extract interceptor processing** into reusable functions
5. **Modularize admin interface** (or move to separate crate)
6. **Centralize session management** interactions
7. **Create proper SSE event processing pipeline**