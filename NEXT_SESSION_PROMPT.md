# MCP Compliance Task 0.1: Fix Initialize Version Extraction

## Context

You are working on the Shadowcat MCP proxy implementation which has critical compliance issues with the MCP specification. The project is currently non-compliant with MCP versions 2025-03-26 (minimum supported) and 2025-06-18 (current target).

**Working Directory**: `/Users/kevin/src/tapwire`

## Current Status

The MCP Compliance project is in **Phase 0: Critical Version Bug Fixes**. This phase addresses fundamental version negotiation bugs that prevent basic MCP compliance. No tasks have been completed yet (0/29 tasks done).

### Critical Issue Being Addressed

The Shadowcat proxy completely ignores the `protocolVersion` field in MCP initialize requests. This violates the MCP specification and prevents proper version negotiation, affecting every MCP session.

**Current Bug Location**: `shadowcat/src/session/manager.rs:783-786`
```rust
TransportMessage::Request { method, .. } if method == "initialize" => {
    session.transition(SessionEvent::InitializeRequest)?;
    // CRITICAL BUG: Ignores params field containing protocolVersion!
}
```

## Task Objectives

Implement proper version extraction from initialize requests:

1. **Extract `protocolVersion`** from initialize request params
2. **Store requested version** in session state  
3. **Add version validation** logic to check if version is supported
4. **Create unit tests** for version extraction functionality

## Essential Context Files to Read

Start by reading these files in order:

1. **Task Specification**: `plans/mcp-compliance/tasks/phase-0-task-001-initialize-version-extraction.md`
   - Contains detailed implementation plan and code examples
   
2. **Critical Bug Report**: `plans/mcp-compliance/006-critical-version-bugs.md`
   - Explains the severity and impact of this bug

3. **Current Implementation**: 
   - `shadowcat/src/session/manager.rs` (lines 783-800) - Where bug exists
   - `shadowcat/src/session/store.rs` - Session structure that needs updating
   - `shadowcat/src/transport/mod.rs` - Current version constants

4. **MCP Specifications** (for reference):
   - `specs/mcp/docs/specification/2025-06-18/basic/lifecycle.mdx` - Initialize handshake
   - `specs/mcp/docs/specification/2025-03-26/basic/lifecycle.mdx` - Older version for comparison

## Implementation Strategy

### Phase 1: Create Protocol Module (30 min)
1. Create new file `shadowcat/src/protocol/mod.rs`
2. Define version constants and supported versions array
3. Implement `extract_protocol_version()` helper function
4. Add `is_version_supported()` validation function

### Phase 2: Update Session Structure (45 min)
1. Modify `shadowcat/src/session/store.rs`
2. Add `VersionInfo` struct with requested/negotiated fields
3. Add `set_requested_version()` method to Session
4. Implement negotiation_required flag logic

### Phase 3: Fix Initialize Handler (45 min)
1. Update `shadowcat/src/session/manager.rs:783-800`
2. Extract params and call version extraction helper
3. Store version in session state
4. Add debug logging for version tracking

### Phase 4: Add Tests (60 min)
1. Create unit tests in protocol module
2. Test version extraction with valid/invalid params
3. Test version support checking
4. Create integration test for full flow

### Phase 5: Validate and Clean Up (30 min)
1. Run all tests: `cargo test`
2. Format code: `cargo fmt`
3. Check for issues: `cargo clippy --all-targets -- -D warnings`
4. Update compliance tracker

## Success Criteria Checklist

- [ ] Protocol version successfully extracted from initialize params
- [ ] Version stored correctly in session state
- [ ] Unsupported versions flagged for negotiation
- [ ] Missing version handled with appropriate default ("2025-03-26")
- [ ] All unit tests passing
- [ ] Integration test demonstrates end-to-end flow
- [ ] No clippy warnings
- [ ] Debug logging added for version tracking
- [ ] Code formatted with `cargo fmt`

## Commands to Use

```bash
# Navigate to shadowcat directory
cd shadowcat

# Create new protocol module
mkdir -p src/protocol

# Run tests frequently
cargo test protocol::
cargo test session::manager::tests::
cargo test version_extraction

# Check your work
cargo fmt
cargo clippy --all-targets -- -D warnings

# Run specific test with output
cargo test test_extract_version_from_valid_params -- --nocapture
```

## Expected Deliverables

By the end of this session, you should have:

1. **New Files Created**:
   - `shadowcat/src/protocol/mod.rs` - Protocol version handling module
   - `shadowcat/tests/version_extraction_test.rs` - Integration tests

2. **Files Modified**:
   - `shadowcat/src/session/store.rs` - Added VersionInfo to Session
   - `shadowcat/src/session/manager.rs` - Fixed initialize handler
   - `shadowcat/src/lib.rs` - Added protocol module export

3. **Tests Passing**:
   - Unit tests for version extraction
   - Unit tests for version validation
   - Integration test for initialize flow

4. **Documentation**:
   - Updated `plans/mcp-compliance/compliance-tracker.md` with task completion

## Important Notes

- **Always use TodoWrite tool** to track your progress through the task
- **Start with examining existing code** to understand current architecture
- **Follow established patterns** from previous implementations
- **Test incrementally** as you build each component
- **Run `cargo fmt`** after implementing new functionality
- **Run `cargo clippy --all-targets -- -D warnings`** before any commit
- **Update the refactor tracker** when the task is complete
- **Focus on the current phase objectives**

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

## Using the rust-code-reviewer

If you encounter complex Rust patterns or need to ensure memory safety, use the `rust-code-reviewer` subagent to:
- Validate ownership and borrowing patterns
- Check async/await usage with tokio
- Verify error handling with Result types
- Ensure no unwrap()/expect() in production code

## Next Steps After Completion

Once Task 0.1 is complete:
- Task 0.2 (Fix HTTP Default Version) can run in parallel
- Task 0.3 (Version Negotiation Response) depends on 0.1 completion
- Update tracker to mark Task 0.1 as complete
- Create new session for next task if context window > 70%

## References

- Compliance Tracker: `plans/mcp-compliance/compliance-tracker.md`
- Task File: `plans/mcp-compliance/tasks/phase-0-task-001-initialize-version-extraction.md`
- Bug Report: `plans/mcp-compliance/006-critical-version-bugs.md`
- MCP 2025-06-18 Spec: `specs/mcp/docs/specification/2025-06-18/`
- MCP 2025-03-26 Spec: `specs/mcp/docs/specification/2025-03-26/`

Good luck! This fix is critical for MCP compliance and will unblock many other tasks.