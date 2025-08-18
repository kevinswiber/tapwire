# Current Structure Analysis of legacy.rs

## File Overview
- **Total Lines**: 3,682
- **Implementation Lines**: ~2,857 (lines 1-2857)
- **Test Lines**: ~824 (lines 2858-3682)
- **Import Statements**: 31 use statements

## Major Sections and Line Counts

### 1. Imports and Dependencies (Lines 1-46)
- 31 use statements
- Heavy dependency on:
  - Axum/Tower for HTTP framework
  - Tokio for async runtime
  - Internal modules: transport (4), mcp (2), auth (2)

### 2. Core Data Structures (Lines 47-355)
- **ReverseProxyServer** (48-53): Main server struct
- **ReverseUpstreamConfig** (57-74): Upstream configuration
- **ReverseLoadBalancingStrategy** (78-86): Load balancing enum
- **ReverseUpstreamHealthCheckConfig** (90-99): Health check config
- **ReverseUpstreamPoolConfig** (102-107): Connection pool config
- **ReverseProxyConfig** (151-178): Main proxy configuration
- **ReverseSessionConfig** (180-184): Session configuration
- **ReverseProxyMetrics** (356-360): Metrics tracking

### 3. Implementation Blocks (Lines 109-401)
- **ReverseUpstreamConfig impl** (109-123, 282-354): Default and validation
- **ReverseUpstreamHealthCheckConfig impl** (124-137): Defaults
- **ReverseUpstreamPoolConfig impl** (138-149): Defaults
- **ReverseProxyConfig impl** (186-222, 223-280): Defaults and validation
- **ReverseProxyMetrics impl** (362-401): Metrics methods

### 4. Builder Pattern (Lines 402-451)
- **ReverseProxyServerBuilder** (402-451): Builder for server configuration

### 5. Server Implementation (Lines 452-1029)
- **Main server methods** (452-960):
  - `new()`: Constructor
  - `build()`: Build from config
  - `start()`: Start server
  - `start_with_address()`: Start with specific address
  - `start_with_shutdown()`: Start with shutdown token
  - `run_with_shutdown()`: Main server loop
- **Router creation** (961-1027): 65 lines for router setup

### 6. Request Handlers (Lines 1030-2705)

#### Major Handlers:
- **handle_mcp_request** (1030-1580): **550 lines** - Main MCP request handler
- **proxy_sse_response** (1585-1670): 83 lines - SSE response proxying
- **handle_mcp_sse_request** (1672-1834): 162 lines - SSE request handler
- **proxy_sse_from_upstream** (1849-2147): **298 lines** - Upstream SSE proxy

#### Helper Functions:
- **get_or_create_session** (2202-2282): 80 lines - Session management
- **process_message** (2286-2316): Message processing
- **process_via_stdio_pooled** (2318-2385): 67 lines - Stdio processing
- **process_via_http_hyper** (2392-2453): 61 lines - HTTP processing
- **forward_raw_response** (2456-2495): Response forwarding
- **process_via_http** (2498-2645): **147 lines** - HTTP processing

### 7. Admin & Monitoring Endpoints (Lines 2706-2828)
- **handle_health** (2706-2713): Health check endpoint
- **handle_metrics** (2715-2796): 81 lines - Metrics endpoint
- **handle_admin_request** (2799-2826): Admin endpoint

### 8. Error Handling (Lines 2829-2857)
- **IntoResponse for ReverseProxyError** (2829-2857): Error response conversion

### 9. Tests Module (Lines 2858-3682)
- 20 test functions
- ~824 lines of test code

## Key Observations

### Complexity Hotspots
1. **handle_mcp_request** (550 lines) - Extremely complex, needs major refactoring
2. **proxy_sse_from_upstream** (298 lines) - Complex SSE handling logic
3. **handle_mcp_sse_request** (162 lines) - SSE request processing
4. **process_via_http** (147 lines) - HTTP processing logic

### Mixed Responsibilities
The file handles:
- Server lifecycle management
- Configuration management
- Request routing
- MCP protocol handling
- SSE streaming
- Session management
- Connection pooling
- Health checks
- Metrics collection
- Admin endpoints
- Error handling

### Natural Module Boundaries Identified

1. **Configuration Module** (~300 lines)
   - All config structs and their implementations
   - Validation logic

2. **Server Module** (~500 lines)
   - ReverseProxyServer struct
   - Builder pattern
   - Server lifecycle methods

3. **Handlers Module** (~800 lines)
   - Request handlers (could be further split)
   - Response processing

4. **SSE Module** (~550 lines)
   - SSE-specific handlers
   - Streaming logic

5. **Processing Module** (~400 lines)
   - Message processing
   - Transport-specific processing (stdio, HTTP)

6. **Session Module** (~100 lines)
   - Session management helpers

7. **Admin Module** (~150 lines)
   - Health, metrics, admin endpoints

8. **Tests Module** (~824 lines)
   - All test code

## Recommendations for Refactoring

### Priority 1: Extract Large Functions
- Break down `handle_mcp_request` into smaller, focused functions
- Split `proxy_sse_from_upstream` into manageable chunks
- Refactor `process_via_http` for clarity

### Priority 2: Module Separation
- Create separate modules for distinct concerns
- Move configuration to `config.rs`
- Extract SSE handling to `sse.rs`
- Move admin endpoints to `admin.rs`

### Priority 3: Improve Testability
- Extract business logic from request handlers
- Create smaller, unit-testable functions
- Separate I/O from logic where possible