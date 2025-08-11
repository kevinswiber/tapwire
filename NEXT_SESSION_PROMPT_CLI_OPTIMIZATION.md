# Next Session: Library Facade & Integration Tests (B.3 & B.6)

## ✅ B.2.1 Completed Successfully!

In the previous session, we fixed all the critical technical debt:
- Created StdioClientTransport for proper stdin/stdout handling
- Fixed forward/record commands to use real proxies with builders
- Implemented proper bidirectional message forwarding
- Added HTTP transport shutdown support
- All 681 tests passing, clippy clean

## Current State
- **Branch**: `shadowcat-cli-refactor` (git worktree at `/Users/kevin/src/tapwire/shadowcat-cli-refactor`)
- **Last Commit**: `3e50e31` - "fix(cli): properly integrate shutdown system with real proxy implementations"
- **Tests**: 681 passing with real proxy behavior
- **Tracker**: `plans/cli-refactor-optimization/cli-refactor-tracker.md`

## Next Phase: B.3 Library Facade (2-3 hours)

### What Needs to Be Done

Create a high-level library API that the CLI can use as a thin wrapper. This establishes the library-first architecture properly.

**Task File**: `plans/cli-refactor-optimization/tasks/B.3-library-facade.md`

### Key Components to Create

1. **Shadowcat struct** - Main entry point for library users
2. **ShadowcatBuilder** - Fluent configuration API
3. **High-level operations**:
   - `forward_stdio()` - Forward proxy via stdio
   - `forward_http()` - Forward proxy via HTTP
   - `record_session()` - Record MCP session
   - `reverse_proxy()` - Run reverse proxy server

### Example API Design
```rust
// Library usage (what CLI will call)
let shadowcat = Shadowcat::builder()
    .with_rate_limiting(100, 20)
    .with_session_timeout(Duration::from_secs(300))
    .build()?;

// Forward proxy with shutdown
shadowcat.forward_stdio(command, shutdown_token).await?;

// Record session
let tape = shadowcat.record_session(command, output_path).await?;
```

## Then: B.6 Integration Tests (2-3 hours)

### What Needs to Be Done

Create comprehensive integration tests that verify the entire system works end-to-end.

**Task File**: `plans/cli-refactor-optimization/tasks/B.6-integration-tests.md`

### Test Categories

1. **Forward Proxy Tests**
   - Stdio forwarding with real MCP server
   - HTTP/SSE forwarding
   - Shutdown during active proxy

2. **Recording Tests**
   - Record and verify tape contents
   - Replay recorded sessions

3. **Reverse Proxy Tests**
   - HTTP server with auth
   - Load balancing
   - Circuit breaker

4. **Resilience Tests**
   - Graceful shutdown under load
   - Recovery from transport failures
   - Rate limiting enforcement

## Commands to Start With

```bash
# Navigate to the worktree
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Review the current structure
tree -I target -L 2 src/

# Read the B.3 task file
cat plans/cli-refactor-optimization/tasks/B.3-library-facade.md

# Check what public API we currently expose
grep -n "^pub " src/lib.rs

# Start implementing the facade
echo "Creating src/facade.rs for high-level API..."
```

## Success Criteria

After B.3 & B.6:
- Clean, intuitive library API that external users could consume
- CLI becomes a thin wrapper around library calls
- Comprehensive test coverage proving the system works
- All components properly integrated and tested
- Documentation for library usage

## Why This Order

1. **B.3 First**: Create the clean API that B.6 will test
2. **B.6 Next**: Prove everything works with real integration tests
3. This validates our architecture before optimization phases

## Remaining Work After This Session

- B.4: Performance optimizations (2 hours)
- B.5: Documentation (1 hour)
- C.1-C.5: Advanced features from Phase C (if needed)

## Duration: 4-6 hours total

This session will establish the proper library-first architecture and prove it works with comprehensive tests.

Good luck!