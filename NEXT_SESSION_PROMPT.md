# MCP Compliance Phase 0 - Task 0.5: Handle Dual-Channel Version Conflicts

## Context

You are working on the Shadowcat MCP proxy implementation in the Tapwire project. The project is currently at 14% completion (4 of 29 tasks) for MCP compliance, with Phase 0 at 80% complete (4 of 5 tasks). This is the FINAL task in Phase 0.

### What Has Been Completed

1. **Task 0.1**: Fix Initialize Version Extraction ✅
   - Created centralized protocol module for version management
   - Removed all non-compliant "2025-11-05" references
   - Added VersionInfo struct with negotiation tracking
   - Implemented backward compatibility between versions

2. **Task 0.2**: Fix HTTP Default Version ✅
   - Changed HTTP default from "2025-11-05" to "2025-03-26"
   - Added HTTP_DEFAULT_VERSION constant in protocol module
   - Updated all HTTP header extraction to use centralized constant

3. **Task 0.3**: Implement Version Negotiation Response ✅
   - Created `protocol/negotiation.rs` with VersionNegotiator
   - Modified forward proxy to intercept initialize responses
   - Track initialize requests by ID for response matching
   - Negotiate versions when client/server mismatch

4. **Task 0.4**: Add Version State Management ✅
   - Created comprehensive VersionState struct with state machine
   - Tracks requested/negotiated/transport versions
   - State transitions: Uninitialized → Requested → Negotiated → Validated
   - **BOTH** forward and reverse proxies track complete version lifecycle
   - Added version constants module (`protocol::versions`)
   - 17 new tests covering all scenarios

### Recent Improvements
- Fixed forward proxy bug where requested version wasn't tracked in session
- Added complete version tracking to reverse proxy (was missing)
- Created 5 comprehensive tests for reverse proxy version tracking
- Established proxy mode parity guidelines

## Current Task: Task 0.5 - Handle Dual-Channel Version Conflicts

### Objective

Implement proper enforcement of dual-channel version validation for MCP 2025-06-18, ensuring that HTTP headers match the negotiated version and providing proper error responses for conflicts. This must be implemented in BOTH forward and reverse proxy modes.

### Working Directory
```
/Users/kevin/src/tapwire/shadowcat
```

### Essential Context Files to Read

1. **Current Implementation**:
   - `src/protocol/version_state.rs` - VersionState with validation logic
   - `src/protocol/mod.rs` - Version constants and helpers
   - `src/proxy/forward.rs` - Forward proxy (check version handling)
   - `src/proxy/reverse.rs` - Reverse proxy (check version handling)
   - `src/transport/http_mcp.rs` - HTTP transport version extraction

2. **Architecture Documents**:
   - `plans/mcp-compliance/006-critical-version-bugs.md` - Bug #5: Dual-Channel Conflicts
   - `plans/mcp-compliance/compliance-tracker.md` - Task 0.5 details

### Implementation Strategy

#### Phase 1: Analyze Current State
1. Review how version validation currently works in both proxies
2. Identify where warnings are logged vs errors returned
3. Check HTTP response handling for version conflicts
4. Understand current error flow

#### Phase 2: Implement Strict Enforcement in Forward Proxy
1. Ensure version mismatches return errors (not just warnings)
2. Add proper error responses for HTTP clients
3. Implement version downgrade prevention
4. Add comprehensive tests

#### Phase 3: Implement Strict Enforcement in Reverse Proxy
1. Ensure HTTP header validation is enforced
2. Return 400 Bad Request for version conflicts
3. Prevent version downgrades
4. Add comprehensive tests

#### Phase 4: Error Response Improvements
1. Create clear error messages for version conflicts
2. Add proper HTTP status codes (400 for client errors)
3. Include diagnostic information in error responses
4. Ensure consistent error format

#### Phase 5: Testing and Validation
1. Test version mismatch scenarios in both proxies
2. Test version downgrade prevention
3. Test HTTP error responses
4. Verify both proxies behave consistently

### Detailed Objectives

1. **Enforce Dual-Channel Validation**:
   - Version 2025-06-18 MUST have matching HTTP headers and negotiated version
   - Version 2025-03-26 does NOT require HTTP header validation
   - Mismatches must be rejected with proper errors

2. **Implement Version Downgrade Prevention**:
   - Prevent clients from downgrading after negotiation
   - Detect and reject attempts to use older versions
   - Log security events for downgrade attempts

3. **Improve Error Responses**:
   - HTTP 400 Bad Request for version conflicts
   - Clear error messages explaining the conflict
   - Consistent error format across both proxies

4. **Ensure Proxy Parity**:
   - Both forward and reverse proxies must enforce equally
   - Error messages must be consistent
   - Tests must cover both modes

### Success Criteria Checklist

- [ ] Version mismatches return errors (not just warnings) in forward proxy
- [ ] Version mismatches return errors (not just warnings) in reverse proxy
- [ ] HTTP 400 Bad Request returned for version conflicts
- [ ] Version downgrade attempts are detected and rejected
- [ ] Clear error messages explain the specific conflict
- [ ] Tests for version mismatch scenarios in forward proxy
- [ ] Tests for version mismatch scenarios in reverse proxy
- [ ] Tests for version downgrade prevention
- [ ] Tests for HTTP error responses
- [ ] All existing tests still pass
- [ ] No clippy warnings
- [ ] Code properly formatted with cargo fmt
- [ ] Documentation updated
- [ ] Compliance tracker updated

### Commands to Use

```bash
# Navigate to shadowcat directory
cd /Users/kevin/src/tapwire/shadowcat

# Run tests frequently
cargo test version
cargo test proxy

# Run specific test modules
cargo test proxy::forward::tests
cargo test proxy::reverse::tests

# Check for compilation errors
cargo build

# Format code
cargo fmt

# Run clippy
cargo clippy --all-targets -- -D warnings

# Run all tests
cargo test

# Test with logging to see warnings/errors
RUST_LOG=shadowcat=debug cargo test

# Commit changes (in shadowcat repo)
git add -A
git commit -m "feat: enforce dual-channel version validation with proper error handling"
git push

# Update parent repo
cd ..
git add shadowcat
git commit -m "feat: complete Phase 0 with dual-channel conflict handling"
git push
```

### Expected Deliverables

1. **Modified files**:
   - `src/proxy/forward.rs` - Enforce version validation, return errors
   - `src/proxy/reverse.rs` - Enforce HTTP header validation, return 400
   - `src/error.rs` - Add specific error types if needed
   - `src/protocol/version_state.rs` - Enhance validation if needed

2. **Test files**:
   - Add tests in forward.rs for version conflict scenarios
   - Add tests in reverse.rs for HTTP header validation
   - Test version downgrade prevention in both

3. **Documentation**:
   - Update comments explaining dual-channel enforcement
   - Document error response format
   - Update compliance tracker

## Important Notes

- **Always use TodoWrite tool** to track your progress through the task
- **Start with examining existing code** to understand current architecture
- **Follow established patterns** from previous implementations
- **Test incrementally** as you build each component
- **Run `cargo fmt`** after implementing new functionality
- **Run `cargo clippy --all-targets -- -D warnings`** before any commit
- **Update the compliance tracker** when the task is complete
- **Focus on the current phase objectives**

## ⚠️ CRITICAL: Proxy Mode Parity

**MUST implement in BOTH proxy modes:**
- Forward Proxy (`src/proxy/forward.rs`)
- Reverse Proxy (`src/proxy/reverse.rs`)

Remember to:
1. ✅ Implement in forward proxy
2. ✅ Implement in reverse proxy
3. ✅ Add tests for both modes
4. ✅ Verify behavior consistency

## Model Usage Guidelines

- **IMPORTANT** Be mindful of model capabilities. Assess whether Claude Opus or Claude Sonnet would be best for each step. When there's a benefit to a model change, pause and recommend it. Be mindful of the context window. When the context window has less than 15% availability, suggest creating a new Claude session and output a good prompt, referencing all available plans, tasks, and completion files that are relevant. Save the prompt into NEXT_SESSION_PROMPT.md.

## Development Workflow

1. Create todo list with TodoWrite tool to track progress
2. Examine existing codebase architecture and established patterns
3. Study current implementations related to the task
4. Design the solution approach and identify key components
5. Implement functionality incrementally with frequent testing
6. Add comprehensive error handling following project patterns
7. Create tests demonstrating functionality works correctly
8. Run tests after each significant change to catch issues early
9. Run `cargo fmt` to ensure consistent code formatting
10. Run `cargo clippy -- -D warnings` to catch potential issues
11. Update project documentation and tracker as needed
12. Commit changes with clear, descriptive messages

## Current Phase Status

**Phase 0: Critical Version Bug Fixes**
- Task 0.1: Fix Initialize Version Extraction ✅
- Task 0.2: Fix HTTP Default Version ✅
- Task 0.3: Implement Version Negotiation Response ✅
- Task 0.4: Add Version State Management ✅
- **Task 0.5: Handle Dual-Channel Version Conflicts** 🎯 CURRENT (FINAL TASK)

After completing Task 0.5, Phase 0 will be COMPLETE (100%), and Phase 1 (Core SSE Implementation) will be ready to begin.

## Key Technical Context

### Dual-Channel Negotiation (2025-06-18)
- Version is negotiated via initialize request/response
- HTTP headers MUST match the negotiated version
- Mismatches indicate potential security issues or bugs
- Must be strictly enforced

### Initialize-Only Negotiation (2025-03-26)
- Version is negotiated ONLY via initialize
- HTTP headers are not validated
- This is the backward-compatible mode

### Current Enforcement Status
- VersionState validates and tracks mismatches
- Reverse proxy returns ProtocolError for mismatches
- Forward proxy may only warn (needs verification)
- Need to ensure consistent strict enforcement

## Notes from Previous Session

- VersionState already has validation logic in place
- Reverse proxy has some enforcement but may need strengthening
- Forward proxy version tracking was fixed but enforcement needs review
- Tests exist for VersionState but need proxy-level conflict tests
- Both proxies now track complete version lifecycle

Start by examining how version conflicts are currently handled in both proxies, then implement strict enforcement with proper error responses and comprehensive tests.

## Completion Criteria

When this task is complete:
- [ ] Phase 0 will be 100% complete (5 of 5 tasks)
- [ ] All critical version bugs will be fixed
- [ ] Dual-channel validation will be properly enforced
- [ ] Both proxy modes will have consistent behavior
- [ ] The codebase will be ready for Phase 1 (SSE Implementation)

Good luck! This is the final step to complete Phase 0 of MCP compliance!