# Main.rs Component Inventory

## File Statistics
- **Total Lines**: 1294
- **Last Updated**: 2025-01-09

## Command Structure

### Top-Level Commands (7 total)
1. **Forward** - Forward proxy with stdio/HTTP transports
2. **Reverse** - Reverse proxy server  
3. **Record** - Session recording with stdio/HTTP transports
4. **Replay** - Replay recorded sessions
5. **Tape** - Tape management (delegated to cli module)
6. **Intercept** - Interception rules (delegated to cli module)
7. **Session** - Session management (delegated to cli module)

### Transport Enums
- **ForwardTransport** (2 variants)
  - Stdio - Forward proxy over stdio
  - Http - Forward proxy over HTTP
- **RecordTransport** (2 variants)
  - Stdio - Record stdio sessions
  - Http - Record HTTP sessions

## Handler Functions

### Core Proxy Handlers
- `run_stdio_forward()` - Lines 281-391 (~110 lines)
  - Creates session manager
  - Sets up rate limiter
  - Spawns stdio transport
  - Currently just sends test initialize request
  
- `run_http_forward_proxy()` - Lines 393-452 (~60 lines)
  - Creates HTTP server with axum
  - Sets up session manager and rate limiter
  - Handles HTTP-to-stdio proxy

- `run_reverse_proxy()` - Lines 536-639 (~103 lines)
  - Creates ReverseProxyServer with full config
  - Integrates rate limiting if enabled
  - Starts axum server

### Recording Handlers  
- `run_stdio_recording()` - Lines 641-731 (~90 lines)
  - Creates tape recorder
  - Spawns stdio transport
  - Records all message exchanges
  
- `run_http_recording()` - Lines 733-793 (~60 lines)
  - Creates HTTP recording server
  - Sets up tape recorder
  - Proxies and records HTTP traffic

- `handle_recording_request()` - Lines 795-891 (~96 lines)
  - HTTP request handler for recording
  - Manages recorder lifecycle

### Replay Handlers
- `run_replay_server()` - Lines 893-1051 (~158 lines)
  - Loads tape file
  - Creates HTTP server for replay
  - Optionally sets up rate limiting
  
- `handle_replay_request()` - Lines 1053-1149 (~96 lines)
  - HTTP request handler for replay
  - Matches requests against tape
  - Returns recorded responses

## Helper Functions

### JSON Conversion Utilities
- `json_to_transport_message()` - Lines 454-499 (~45 lines)
  - Converts JSON to ProtocolMessage
  - Handles request/response/notification types

- `transport_message_to_json()` - Lines 501-534 (~33 lines)
  - Converts ProtocolMessage to JSON
  - Preserves all message fields

### Message Matching
- `messages_match()` - Lines 1151-1160 (~9 lines)
  - Compares tape and request messages
  - Used for replay matching

### Logging
- `init_logging()` - Lines 253-279 (~26 lines)
  - Sets up tracing subscriber
  - Configures log levels

## Configuration Structures

### ProxyConfig
- **Definition**: Lines 208-251 (~43 lines)
- **Fields**: 
  - Rate limiting settings (enable, rpm, burst)
  - Session settings (timeout, max_sessions, cleanup_interval)
- **Methods**:
  - `from_cli_args()` - Creates from CLI parameters
  - `to_session_config()` - Converts to SessionConfig
- **Usage**: Used by forward (stdio/HTTP), reverse, and replay commands

## Main Function
- **Lines**: 1162-1293 (~131 lines)
- **Structure**: 
  - Parse CLI args
  - Initialize logging
  - Match command and dispatch to handler
  - Error handling and exit

## Already Modularized Components
- **Tape management** - Delegates to `shadowcat::cli::tape::TapeCli`
- **Intercept management** - Delegates to `shadowcat::cli::intercept::InterceptManager`
- **Session management** - Delegates to `shadowcat::cli::session::SessionCli`

## Code Patterns

### Rate Limiter Creation
- **Pattern**: MultiTierRateLimiter::new() with config
- **Occurrences**: 3 times (lines 319, 430, 923)
- **Configuration**: Disables per-user/ip/endpoint/session, only uses global

### Session Manager Creation
- **Pattern**: Arc::new(SessionManager::with_config())
- **Used in**: All proxy and recording handlers

### Error Handling
- **Pattern**: Result<()> returns with ShadowcatError
- **Exit on error**: main() calls exit(1) on error