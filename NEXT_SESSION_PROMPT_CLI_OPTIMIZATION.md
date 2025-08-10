# Next Session: CRITICAL - Fix Shutdown Integration (B.2.1)

## ⚠️ Critical Technical Debt Identified

During B.2 implementation, we took significant shortcuts that compromise the design integrity. These MUST be fixed before proceeding to B.3 (Library Facade).

## Problems to Fix

### 1. **Fake Proxy Implementation**
The CLI commands don't actually proxy anything - they just send one test message:
```rust
// This is NOT a real proxy!
let test_msg = ProtocolMessage::new_request(...);
server_transport.send(envelope).await?;
```

### 2. **Missing Client Transport**
We have no stdin/stdout reader for the client side. The forward proxy can't read from stdin.

### 3. **Bypassed Builder Patterns**
We created builders in B.1 but the CLI shutdown integration doesn't use them.

### 4. **No HTTP Shutdown Support**
Only stdio transports have shutdown - HTTP was ignored with a TODO.

### 5. **Mixed Abstraction Levels**
High-level shutdown system mixed with low-level transport manipulation in CLI.

## Current State
- **Branch**: `shadowcat-cli-refactor` (git worktree at `/Users/kevin/src/tapwire/shadowcat-cli-refactor`)
- **Tests**: 679 passing (but they don't test real proxy behavior)
- **Task File**: `plans/cli-refactor-optimization/tasks/B.2.1-proper-shutdown-integration.md`

## What Needs to Be Done (B.2.1)

1. **Create StdioClientTransport** for reading stdin/writing stdout
2. **Fix Forward Command** to use real proxy with builders
3. **Implement Proper Proxy Loop** with bidirectional forwarding and shutdown
4. **Fix Record Command** similarly  
5. **Add HTTP Transport Shutdown**
6. **Remove Unused Macro** (select_with_shutdown!)
7. **Fix Error Handling** (no more `let _ =` patterns)

## Commands to Start With

```bash
# Navigate to the worktree
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Review the current broken implementation
grep -n "test_msg" src/cli/forward.rs
grep -n "TODO" src/cli/

# Read the detailed task file
cat /Users/kevin/src/tapwire/plans/cli-refactor-optimization/tasks/B.2.1-proper-shutdown-integration.md

# Check current tests (they pass but don't test real proxy)
cargo test shutdown
```

## Expected Outcome

After B.2.1:
- Forward proxy actually forwards messages between client and server
- Stdin/stdout properly handled
- Builders used consistently
- HTTP transport has shutdown support
- No placeholder implementations
- Clean abstraction levels
- Integration tests with real proxy scenarios

## Why This Is Critical

Without fixing this:
- B.3 (Library Facade) will wrap broken implementations
- B.6 (Integration Tests) will immediately fail
- We're violating our library-first design principle
- The codebase looks functional but isn't

## Design Principles to Maintain

1. **Library-First**: CLI should be a thin wrapper
2. **Builder Pattern**: Use the builders we created
3. **Clean Abstractions**: Don't mix levels
4. **Real Implementations**: No placeholders in production code
5. **Proper Error Handling**: Log or propagate, don't ignore

## Duration: 4 hours

This is technical debt that will compound if not addressed immediately. The shutdown system architecture is good, but the integration needs to be real, not fake.

Good luck!