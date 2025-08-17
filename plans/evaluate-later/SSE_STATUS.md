# SSE Streaming Status and Next Steps

## Current Situation
- **Problem**: MCP Inspector client hangs when receiving SSE responses from reverse proxy
- **Root Cause**: We're trying to consume the entire SSE stream with `response.text().await`, which blocks forever since SSE streams don't end
- **Failed Fix Attempt**: Making duplicate requests to upstream (inefficient and incorrect)

## What We've Learned
1. SSE responses must be streamed, not buffered
2. We have existing SSE infrastructure in `src/transport/sse/` that we should use
3. Interceptors need to process complete SSE events, not raw bytes
4. Session mapping is required for proper reverse proxy operation
5. The `reverse.rs` file is too large (3400+ lines) and needs refactoring

## Proper Architecture

### Option 1: Early Detection (Recommended)
- Check Accept header before making upstream request
- If SSE is accepted, use a different code path that streams the response
- Never try to parse SSE as JSON-RPC ProtocolMessage
- Process each SSE event through interceptors as it arrives

### Option 2: Response Type Enum
- Change `process_via_http` to return an enum:
  ```rust
  enum UpstreamResponse {
      Json(ProtocolMessage),
      SseStream(Response), // Keep the response for streaming
  }
  ```
- Handle each type appropriately in the caller

### Option 3: Separate Handlers
- Have completely separate handlers for JSON and SSE requests
- `/mcp` POST with JSON -> JSON handler
- `/mcp` GET/POST with SSE Accept -> SSE handler

## Immediate Fix Needed
For now, we need a working solution that:
1. Doesn't make duplicate requests
2. Properly streams SSE without buffering
3. Doesn't cause client timeouts

## Recommended Approach
1. **Short-term**: Fix the immediate hanging issue by not consuming SSE streams
2. **Medium-term**: Refactor into separate modules (Phase 1 of refactor plan)
3. **Long-term**: Integrate existing SSE infrastructure with interceptors and session mapping

## Technical Constraints
- Can't return both ProtocolMessage and raw Response from same function
- Need to maintain session state for both JSON and SSE responses
- Must support interceptors for both response types
- Should reuse existing SSE infrastructure

## Next Action
Instead of the current hacky approach with duplicate requests, we should:
1. Detect SSE early (from Accept header or first response)
2. Branch to different handling paths
3. For SSE: Stream directly without trying to parse as JSON
4. For JSON: Continue with current approach

This avoids the need for `SseStreamingRequired` error and duplicate requests.