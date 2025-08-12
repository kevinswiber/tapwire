# Next Session: Wire Correlation to SSE Transport (M.5)

## Project Status Update

We are implementing SSE proxy integration with MCP message handling in Shadowcat, following the unified tracker at `plans/proxy-sse-message-tracker.md`.

**Current Status**: Phase 3 nearly complete!  
**Phase 0**: 100% Complete ✅ (F.1-F.5 all done)  
**Phase 1**: 100% Complete ✅ (S.1-S.4 all done)  
**Phase 2**: 100% Complete ✅ (R.1-R.4 all done)  
**Phase 3**: 87% Complete ✅ (M.1-M.4 done, only M.5 remaining)

## Accomplishments Previous Session

### Phase 3: Message Builder and Correlation Engine ✅

Successfully completed M.3 and M.4:

**M.3: Message Builder API** ✅
- Created fluent builder API in `src/mcp/builder.rs`
- Implemented RequestBuilder, ResponseBuilder, NotificationBuilder, BatchBuilder
- Added specialized helpers for common MCP methods
- 15+ comprehensive unit tests including builder/parser integration
- All tests passing, no clippy warnings

**M.4: Correlation Engine** ✅  
- Created thread-safe correlation engine in `src/mcp/correlation.rs`
- Features: request tracking, timeout management, statistics, configurable limits
- Background cleanup task with graceful shutdown
- 8+ comprehensive async tests
- All tests passing, no clippy warnings

## Remaining Task: M.5 - Wire Correlation to SSE Transport

### Objective
Integrate the correlation engine with SSE transport to automatically track request/response pairs during proxying.

### Implementation Plan

1. **Modify `src/transport/sse.rs`**
   - Add `CorrelationEngine` field to `SseTransport`
   - Start correlation engine on transport connect
   - Stop correlation engine on transport disconnect

2. **Track Outgoing Requests**
   ```rust
   // In SseTransport::send()
   if let ProtocolMessage::Request { id, .. } = &message {
       let metadata = parser.extract_metadata(&mcp_message);
       self.correlation.track_request(mcp_message, metadata, None).await?;
   }
   ```

3. **Correlate Incoming Responses**
   ```rust
   // In SseTransport::receive()
   if let ProtocolMessage::Response { id, .. } = &message {
       match self.correlation.correlate_response(mcp_message).await {
           Ok(completed) => {
               debug!("Correlated response in {}ms", completed.duration.as_millis());
           }
           Err(e) => {
               warn!("Correlation failed: {}", e);
           }
       }
   }
   ```

4. **Add Metrics Collection**
   - Expose correlation stats via transport metrics
   - Track success rates, timeouts, response times
   - Integrate with existing metrics collection

5. **Configuration**
   - Add correlation config to SSE transport options
   - Allow customization of timeout and capacity limits

### Testing Strategy

1. **Unit Tests**
   - Mock SSE transport with correlation
   - Test request tracking and response correlation
   - Verify timeout handling

2. **Integration Tests**  
   - Full SSE proxy flow with correlation
   - Multiple concurrent requests
   - Performance impact measurement

### Success Criteria

1. ✅ Correlation engine integrated with SSE transport
2. ✅ Automatic request/response tracking
3. ✅ Metrics exposed for monitoring
4. ✅ Tests demonstrating correlation in action
5. ✅ No performance regression (< 5% overhead)

## Commands for Development

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Run SSE transport tests
cargo test transport::sse

# Run correlation tests
cargo test mcp::correlation

# Check integration
cargo test --test sse_transport_test

# Performance check
cargo bench transport

# Check for warnings
cargo clippy --all-targets -- -D warnings
```

## Key Files to Modify

1. `src/transport/sse.rs` - Main integration point
2. `tests/sse_transport_test.rs` - Integration tests
3. `src/metrics/mod.rs` - Add correlation metrics (if exists)

## Next Steps After M.5

Once M.5 is complete, Phase 3 will be 100% done! Next phases include:
- **Phase 4**: Interceptor Integration (I.1-I.5)
- **Phase 5**: Recording Implementation (P.1-P.6)
- **Phase 6**: Integration and Testing

## Notes

- The correlation engine is already thread-safe and ready for integration
- Focus on clean integration without breaking existing SSE functionality
- Consider making correlation optional via configuration
- Remember to handle both request and response directions in proxy mode

---

**Primary Goal**: Complete M.5 by integrating the correlation engine with SSE transport for automatic request/response tracking.