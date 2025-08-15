# Next Session: Implement Phase C - Fix SSE Bug

## Context
We've completed Phase B (SessionStore Abstraction) of the reverse proxy refactor. A critical architectural discovery was made: frames and MessageEnvelopes were conflated. We've separated these concerns - frames belong in the recording domain, not session management.

## Current Status
- **Phase A**: ✅ COMPLETE (5 hours) - All analysis done
- **Phase B**: ✅ COMPLETE (4 hours) - SessionStore abstraction implemented
  - Created SessionStore trait (WITHOUT frame storage)
  - Moved InMemorySessionStore to memory.rs
  - Updated SessionManager to use trait
  - Fixed all compilation issues
  - All 57 tests passing
- **Phase C**: ⬜ Ready to start - Fix SSE bug (5-6 hours)
- **Phase D**: ⬜ Blocked on C - Modularization (10 hours)
- **Phase E**: ⬜ Final - Integration & testing (4 hours)

## Key Architectural Decisions from Phase B
1. **Frames vs MessageEnvelopes**: Separated recording concerns from session management
2. **SessionStore has NO frame storage**: Frames belong in tape/recording domain
3. **MessageEventReceiver abstraction needed**: Proxy shouldn't know about tapes/frames (TODO in Phase D)

## Your Task: Implement Phase C - Fix SSE Bug Properly

### Key Documents to Read First
1. **Main Tracker**: `plans/reverse-proxy-refactor/tracker.md`
2. **Implementation Guide**: `plans/reverse-proxy-refactor/analysis/unified-plan.md`
3. **SSE Solution**: `plans/reverse-proxy-refactor/analysis/corrected-sse-solution.md`
4. **Frame Decision**: `plans/reverse-proxy-refactor/analysis/frame-vs-envelope-decision.md`

### The SSE Bug
- Proxy makes **duplicate HTTP requests** for SSE streams
- Root cause: Function signature returns ProtocolMessage, can't stream Response
- Current workaround uses error as control flow (SseStreamingRequired)
- This causes Response to be dropped, requiring duplicate request

### Phase C Implementation Steps

#### C.1: Create UpstreamResponse Wrapper (1.5 hours)
**File**: `src/proxy/reverse/upstream_response.rs`

```rust
pub struct UpstreamResponse {
    pub response: Response,
    pub content_type: Option<Mime>,
    pub is_sse: bool,
    pub is_json: bool,
    pub session_id: SessionId,
}
```

Modify `process_via_http()` to return `Result<UpstreamResponse>` instead of `Result<ProtocolMessage>`.

#### C.2: Early Content-Type Detection (1 hour)
- Parse Content-Type header immediately after getting Response
- Determine routing path (JSON vs SSE vs other) early
- Remove the duplicate request pattern

#### C.3: Implement SSE Streaming Path (2 hours)
- Use existing SseParser from transport layer
- Stream events without buffering entire response
- Process each SSE event through interceptors incrementally
- Use backpressure to control upstream reading pace

#### C.4: Handle Last-Event-Id (1 hour)
- Use SessionStore's SSE methods (added in Phase B)
- Store Last-Event-Id for reconnection support
- Include Last-Event-Id header in reconnection requests

#### C.5: Test and Validate (0.5 hours)
- Test with MCP Inspector
- Verify no duplicate requests
- Check SSE streaming works without timeouts
- Confirm Last-Event-Id reconnection

### Success Criteria for Phase C
- [ ] No duplicate HTTP requests for SSE streams
- [ ] SSE streams without buffering/timeouts
- [ ] UpstreamResponse wrapper implemented
- [ ] Content-Type routing works correctly
- [ ] Last-Event-Id stored via SessionStore
- [ ] Tests pass with MCP Inspector

### Testing Commands
```bash
# Build and check
cargo build --release
cargo clippy --all-targets -- -D warnings

# Test SSE specifically
cargo test --test integration_sse

# Run with Inspector (manual test)
./target/release/shadowcat reverse \
  --bind 127.0.0.1:8080 \
  --upstream http://localhost:3000
```

### What NOT to Do in Phase C
- Don't modularize the code yet (Phase D)
- Don't implement recording abstraction yet (Phase D.4)
- Don't add Redis support yet (future)
- Focus ONLY on fixing the SSE bug

### Important Notes
1. The `SseStreamingRequired` error should be completely removed
2. Use `bytes_stream()` on Response for streaming
3. The SessionStore now has Last-Event-Id methods (use them!)
4. Recording is handled by TapeRecorder, not SessionManager

### Next Phase Preview (Phase D)
After Phase C is complete, Phase D will:
- Break up the 3,482-line file into modules
- Extract admin interface (876 lines)
- Implement MessageEventReceiver abstraction
- Create clean module boundaries

### Time Estimate
- Phase C: 5-6 hours
- Remaining work: 14 hours
- Total project: ~23 hours (9 complete, 14 remaining)

## Questions?
All design decisions have been made and documented. The SSE fix approach is clear in `analysis/corrected-sse-solution.md`. Begin with C.1: Create UpstreamResponse wrapper.