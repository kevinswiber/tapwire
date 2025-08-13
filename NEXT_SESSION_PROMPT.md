# Next Session Prompt

## Project Context

You are working on **Shadowcat**, a high-performance MCP (Model Context Protocol) proxy written in Rust. This is part of the larger Tapwire platform vision for MCP inspection, recording, and observability.

**Current Status**: Phase 0-6 ✅ COMPLETE, Ready for Phase 7 (Testing & Integration)

## Recent Achievements (2025-01-13)

### Phase 6: MCP-Aware Replay System (Completed)
Successfully implemented a sophisticated replay system with four integrated components:

1. **Replay Engine Core** (`src/replay/engine.rs`)
   - Tape loading and frame processing
   - Variable speed playback (0.1x to 10x)
   - Event-driven architecture
   - Configurable timing control

2. **Replay Controller** (`src/replay/controller.rs`)
   - High-level playback controls
   - Breakpoint system for debugging
   - Frame-by-frame stepping
   - State tracking and event handlers

3. **Message Transformer** (`src/replay/transformer.rs`)
   - Timestamp updates to current time
   - Session ID regeneration/override
   - Field replacements in JSON
   - Authentication token stripping

4. **SSE Replay Support** (`src/replay/sse_support.rs`)
   - SSE stream reconstruction
   - Keep-alive events and retry delays
   - Metadata comments for debugging
   - Connection simulation features

## Completed Phases Summary

According to @plans/proxy-sse-message-tracker.md:
- ✅ Phase 0: Foundation Components (11 hours)
- ✅ Phase 1: SSE Transport with MCP Awareness (12 hours)
- ✅ Phase 2: Reverse Proxy Streamable HTTP (12 hours)
- ✅ Phase 3: Full MCP Parser and Correlation (16 hours)
- ✅ Phase 4: MCP-Aware Interceptor (17 hours)
- ✅ Phase 5: MCP-Aware Recorder (16 hours)
- ✅ Phase 5.5: Recorder Consolidation (16 hours, completed in ~3 hours)
- ✅ Phase 6: MCP-Aware Replay (15 hours, completed in ~4 hours)

**Total Completed**: 115 hours of implementation (107 hours actual)

## Next Tasks: Phase 7 - Testing and Integration (22 hours total)

All components are ready for comprehensive testing:

### T.1-T.3: Integration Tests (8h)
- T.1: Forward Proxy SSE Tests (2h)
- T.2: Reverse Proxy Streamable HTTP Tests (3h)
- T.3: End-to-End MCP Flow Tests (3h)

### T.4-T.7: Component Tests (10h)
- T.4: MCP Parser Conformance Tests (2h)
- T.5: Correlation Engine Tests (2h)
- T.6: Interceptor Integration Tests (3h)
- T.7: Recorder/Replay Tests (3h)

### T.8: Performance Benchmarks (4h)
- Measure < 5% overhead target
- Memory usage validation
- Throughput testing

## Current Architecture

### Replay System Integration
```rust
// The replay system connects with existing components:
TapeStorage → ReplayEngine → MessageTransformer → Output
                ↓                                     ↓
         ReplayController                    SseReplayAdapter
```

### Key Features Implemented
- **Recording**: Both forward and reverse proxies record sessions
- **Replay**: Full replay system with transformation capabilities
- **Interception**: Rule-based message interception with pause/resume
- **Correlation**: Request-response matching with timeouts
- **SSE Support**: Full SSE transport in forward and reverse proxies

## Testing Strategy for Phase 7

### Integration Test Areas
1. **Proxy Flow Tests**
   - Forward proxy with all transports (stdio, HTTP, SSE)
   - Reverse proxy with MCP endpoint
   - Session management across proxies

2. **Recording/Replay Cycle**
   - Record session → Save tape → Load tape → Replay
   - Verify message fidelity
   - Test transformations

3. **Interceptor Chain**
   - Multiple interceptors in sequence
   - Pause/resume with external control
   - Rule processing performance

4. **SSE Streaming**
   - Long-running SSE connections
   - Reconnection handling
   - Event ID correlation

## Key Technical Context

### Performance Requirements
- Latency overhead: < 5% p95
- Memory per session: < 100KB
- Throughput: > 10,000 req/sec
- Startup time: < 100ms

### Testing Commands
```bash
# Run all tests
cargo test

# Run specific phase tests
cargo test replay::           # Replay system tests
cargo test recorder::         # Recording tests
cargo test interceptor::      # Interceptor tests
cargo test transport::sse::   # SSE transport tests

# Run integration tests
cargo test --test integration_*

# Run benchmarks
cargo bench

# Check test coverage
cargo tarpaulin --out Html
```

## Development Guidelines

### Code Quality Standards
```bash
# Before ANY commit, run:
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

### Key Files to Review
- `src/replay/README.md` - Comprehensive replay documentation
- `src/replay/mod.rs` - Module with rustdoc examples
- `plans/proxy-sse-message-tracker.md` - Project tracker
- `tests/integration/` - Integration test structure

## Session Focus

For the next session, focus on Phase 7 - Testing and Integration:

1. **Start with T.1-T.3**: Integration tests for the complete flow
2. **Then T.4-T.7**: Component-specific conformance tests
3. **Finally T.8**: Performance benchmarking

The testing phase should:
- Validate all proxy modes work correctly
- Ensure recording/replay cycle is complete
- Verify MCP protocol compliance
- Measure performance against targets
- Test error handling and edge cases

## Quick Start Commands

```bash
# Navigate to shadowcat
cd shadowcat

# Run replay tests to verify Phase 6
cargo test replay::

# Start integration testing
cargo test --test integration_forward_proxy
cargo test --test integration_reverse_proxy

# Check for any remaining clippy issues
cargo clippy --all-targets -- -D warnings
```

## Important Notes

- All 6 implementation phases are complete
- Replay system has comprehensive documentation
- 38 replay tests passing, zero clippy warnings
- Focus should be on integration testing and validation
- Performance benchmarking is critical for production readiness
- Consider creating example applications demonstrating features

## Additional Context

### Replay System Capabilities
The newly implemented replay system can:
- Load tapes from storage or directly
- Play at variable speeds with timing control
- Transform messages during replay (timestamps, IDs, fields)
- Generate SSE streams from recorded sessions
- Support debugging with breakpoints
- Handle both stdio and SSE transports

### Integration Points
The replay system integrates with:
- `TapeStorage` for loading recordings
- `MessageEnvelope` for frame structure
- `TransportContext` for transport-specific metadata
- `ProtocolMessage` for MCP message handling

Refer to @plans/proxy-sse-message-tracker.md for the complete task breakdown and dependencies.