# Phase 1: Core SSE Implementation - Ready to Begin

## Context

You are working on the Shadowcat MCP proxy implementation in the Tapwire project. Phase 0 (Critical Version Bug Fixes) has been COMPLETED with all 5 tasks finished successfully. The project is now at 17% overall completion (5 of 29 tasks).

### Phase 0 Achievements
- ✅ All critical version bugs fixed
- ✅ Dual-channel validation fully enforced
- ✅ Version downgrade prevention implemented
- ✅ Both proxy modes have version state parity
- ✅ Performance optimized after thorough code review
- ✅ 419 tests passing, no clippy warnings

### Working Directory
```
/Users/kevin/src/tapwire/shadowcat
```

## Next Steps: Phase 1 - Core SSE Implementation

### Option 1: Generate Task Files (Recommended First)
Before implementing Phase 1, the detailed task files need to be generated. This should be done as a focused session to create:

1. `plans/mcp-compliance/tasks/phase-1-task-001-sse-event-parser.md`
2. `plans/mcp-compliance/tasks/phase-1-task-002-sse-connection-management.md`
3. `plans/mcp-compliance/tasks/phase-1-task-003-sse-reconnection.md`
4. `plans/mcp-compliance/tasks/phase-1-task-004-sse-session-integration.md`
5. `plans/mcp-compliance/tasks/phase-1-task-005-sse-performance.md`

Each task file should include:
- Detailed objectives and requirements
- Implementation approach
- Test scenarios
- Success criteria
- Dependencies and integration points

### Option 2: Begin Task 1.1 - SSE Event Parser
If task files already exist or you want to proceed directly:

**Objective**: Implement a robust SSE (Server-Sent Events) parser for MCP Streamable HTTP transport

**Key Requirements**:
- Parse SSE format with field types: data:, event:, id:, retry:
- Handle multi-line data fields correctly
- Support custom event types
- Handle edge cases (empty lines, comments, malformed data)
- Create comprehensive unit tests

**Files to Create/Modify**:
- Create `src/transport/sse/mod.rs` - SSE module structure
- Create `src/transport/sse/parser.rs` - Core SSE parser implementation
- Create `src/transport/sse/event.rs` - SSE event types and structures
- Update `src/transport/mod.rs` - Export SSE module

**Implementation Approach**:
1. Define SSE event structures (SseEvent, SseField, etc.)
2. Implement streaming parser using tokio
3. Handle field accumulation and event emission
4. Add error handling for malformed SSE data
5. Create unit tests for various SSE formats
6. Integration with existing transport layer

## Important Resources

### Compliance Tracker
`/Users/kevin/src/tapwire/plans/mcp-compliance/compliance-tracker.md`

### Phase 0 Review
`/Users/kevin/src/tapwire/plans/mcp-compliance/task-0.5-review-improvements.md`

### MCP Specifications
- SSE Transport: `specs/mcp/docs/specification/2025-06-18/basic/transports.mdx`
- Protocol Basics: `specs/mcp/docs/specification/2025-06-18/basic/index.mdx`

## Commands to Use

```bash
# Navigate to shadowcat
cd /Users/kevin/src/tapwire/shadowcat

# Run tests
cargo test sse  # Once SSE tests are created
cargo test

# Check compilation
cargo build

# Format and lint
cargo fmt
cargo clippy --all-targets -- -D warnings

# Commit (when ready)
git add -A
git commit -m "feat: implement SSE event parser for MCP transport"
git push

# Update parent repo
cd ..
git add shadowcat
git commit -m "feat: begin Phase 1 - SSE implementation"
git push
```

## Success Criteria for Phase 1

When Phase 1 is complete:
- [ ] SSE parser can handle all standard SSE formats
- [ ] Connection management supports persistent SSE streams
- [ ] Automatic reconnection with exponential backoff
- [ ] Sessions properly track SSE connections
- [ ] Performance benchmarks show minimal overhead
- [ ] All tests passing
- [ ] Documentation complete

## Model Usage Guidelines

- **IMPORTANT** Be mindful of model capabilities. Assess whether Claude Opus or Claude Sonnet would be best for each step. When there's a benefit to a model change, pause and recommend it. Be mindful of the context window. When the context window has less than 15% availability, suggest creating a new Claude session and output a good prompt, referencing all available plans, tasks, and completion files that are relevant. Save the prompt into NEXT_SESSION_PROMPT.md.

Good luck with Phase 1! The foundation from Phase 0 is solid and ready for SSE implementation.