# Next Session Prompt - Session 12

## Quick Start
Continue the refactor-legacy-reverse-proxy plan by implementing critical fixes H.6 through H.10.

## Current Status (Session 11 Complete)
- ✅ H.0: Fixed connection pool (inner Arc pattern)
- ✅ H.1: Fixed subprocess health detection  
- ✅ H.2: Added server Drop implementation
- ✅ H.3: P95 latency Phase 1 optimizations applied (30-40% improvement expected)
- ✅ H.4: Deduplicated AppState creation
- ✅ H.5: Implemented SSE reconnection with exponential backoff (with architectural research)

## Priority Tasks for Session 12

### 🟡 HIGH - H.6: Add Request Timeouts (3 hours)
**Add proper timeout handling for all upstream connections.**

Implement separate timeouts for:
- Connection establishment
- Request sending  
- Response receiving

Files: All upstream implementations in `shadowcat/src/proxy/reverse/upstream/`

### 🟡 HIGH - H.7: Restore Buffer Pooling (2 hours)
Re-enable buffer pooling for SSE to reduce memory usage.
Check what was removed by comparing with legacy.rs backup.

### 🟡 HIGH - H.9: Performance Benchmarks (3 hours)
Validate our P95 latency improvements:
- Run benchmarks to confirm 30-40% improvement from Phase 1
- Identify if Phase 2 optimizations are needed
- Compare against legacy baseline

## Context and Documentation
- **Tracker**: `@plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`
- **Reviews**: `@plans/refactor-legacy-reverse-proxy/reviews/` - Comprehensive analysis
- **Task Details**: `@plans/refactor-legacy-reverse-proxy/tasks/`
- **P95 Analysis**: `@shadowcat/docs/p95-latency-analysis.md`

## What Was Completed in Session 11
1. **H.5**: Implemented SSE reconnection with exponential backoff
   - Created `reconnect_simple.rs` module with reconnection logic
   - Added exponential backoff with jitter for retry delays
   - Implemented session state preservation via Last-Event-Id header
   - Added event deduplication to handle duplicate events after reconnection
   - Integrated reconnection into reverse proxy handlers
   - Created integration tests (test compilation issues remain but lib tests pass)

2. **Architectural Research**: Deep analysis of interceptor architecture
   - Discovered fundamental issues with Sync requirement for streams
   - Identified performance bottleneck: processing SSE streams as individual events (50-100μs per event)
   - Clarified MCP transport model: 2 transports (stdio and Streamable HTTP), not 3
   - Documented that interceptors should remain transport-agnostic
   - Created comprehensive research documentation in `docs/research/interceptor-streams/`
   - Key insight: Performance issue is not transport awareness but event-by-event processing
   - Solution: Optimize stream processing at transport layer while keeping interceptors protocol-focused

## What Was Completed in Session 10
1. **H.1**: Fixed subprocess health detection - wrapped Child in Arc<Mutex> for thread-safe status checking
2. **H.2**: Added Drop implementation to ReverseProxyServer for proper resource cleanup
3. **H.3**: Analyzed P95 latency issues and applied Phase 1 optimizations:
   - Pre-computed command strings to avoid per-request allocation
   - Added conditional compilation for debug-only logging
   - Direct byte serialization instead of JSON Value intermediate
4. **H.4**: Deduplicated AppState creation into centralized functions

## Development Workflow
1. Check tracker for latest status
2. Pick next task (H.5 is critical)
3. Create task file in `tasks/` directory
4. Implement solution
5. Run tests: `cargo test --lib` and relevant integration tests
6. Update tracker and commit changes

## Testing Commands
```bash
# Quick checks during development
cargo check
cargo test --lib

# Before committing
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test

# Specific tests
cargo test --test test_reverse_proxy_sse
cargo test --test e2e_resilience_test
cargo test --test test_subprocess_health

# Performance validation
cargo bench reverse_proxy
```

## Git Workflow
Remember: shadowcat is a submodule - commit there first!
1. Make changes in shadowcat/
2. Commit and push shadowcat (branch: refactor/legacy-reverse-proxy)
3. Update tapwire tracker and docs
4. Commit and push tapwire (branch: main)

## Success Criteria for Session 12
- [ ] H.6 Request timeouts implemented
- [ ] H.7 Buffer pooling restored (if time permits)
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Tracker updated with progress
- [ ] Performance benchmarks started (H.9)

## Files to Reference
```bash
cd /Users/kevin/src/tapwire/shadowcat
git checkout refactor/legacy-reverse-proxy

# Recently modified files
src/proxy/reverse/upstream/stdio.rs     # P95 optimizations applied
src/proxy/reverse/server.rs             # Drop impl and AppState dedup
src/transport/outgoing/subprocess.rs    # Health detection logic

# Next files to work on
src/proxy/reverse/upstream/http/streaming/initiator.rs  # SSE reconnection
src/proxy/reverse/handlers/mcp.rs                       # SSE handler
```