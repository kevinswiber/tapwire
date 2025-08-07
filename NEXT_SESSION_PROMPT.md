# Phase 1, Task 1.1: Implement SSE Event Parser

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

## Current Task: SSE Event Parser Implementation

### Objective
Implement a robust Server-Sent Events (SSE) parser for the MCP Streamable HTTP transport that can handle all standard SSE formats, edge cases, and protocol requirements.

### Task Details
**File**: `/Users/kevin/src/tapwire/plans/mcp-compliance/tasks/phase-1-task-001-sse-event-parser.md`
**Duration**: 3-4 hours
**Priority**: CRITICAL - Foundation for all SSE functionality

## Essential Context Files to Read

1. **Task Specification**: 
   - `/Users/kevin/src/tapwire/plans/mcp-compliance/tasks/phase-1-task-001-sse-event-parser.md`

2. **MCP SSE Specification**:
   - `/Users/kevin/src/tapwire/specs/mcp/docs/specification/2025-06-18/basic/transports.mdx` (Streamable HTTP section)
   - `/Users/kevin/src/tapwire/specs/mcp/docs/specification/2025-03-26/basic/transports.mdx` (for comparison)

3. **Existing Transport Infrastructure**:
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/mod.rs`
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/http.rs`
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/stdio.rs`

4. **Session Management** (for integration context):
   - `/Users/kevin/src/tapwire/shadowcat/src/session/mod.rs`

## Implementation Strategy

### Phase 1: Module Setup (30 min)
1. Create `src/transport/sse/` directory structure
2. Set up module files: `mod.rs`, `event.rs`, `parser.rs`, `buffer.rs`
3. Define core types: `SseEvent`, `SseField`, `SseError`
4. Export from transport module

### Phase 2: Core Parser (1.5 hours)
1. Implement `SseParser` struct with state machine
2. Add line parsing logic for SSE fields
3. Handle multi-line data concatenation
4. Implement event dispatch on empty lines
5. Support comments (lines starting with `:`)

### Phase 3: Edge Cases & Optimization (1 hour)
1. Handle partial message buffering
2. Add BOM handling at stream start
3. Implement zero-copy optimizations where possible
4. Add retry field parsing
5. Handle malformed input gracefully

### Phase 4: Testing (1 hour)
1. Unit tests for basic SSE parsing
2. Tests for multi-line data fields
3. Tests for custom event types
4. Tests for event IDs and retry values
5. Tests for edge cases and malformed input

## Success Criteria Checklist

- [ ] Parser correctly processes standard SSE format
- [ ] Multi-line data fields are properly concatenated with newlines
- [ ] Custom event types are preserved
- [ ] Event IDs are tracked for resumability
- [ ] Retry intervals are parsed and stored
- [ ] Comments (lines starting with `:`) are ignored
- [ ] Partial messages are buffered until complete
- [ ] Zero-copy parsing where possible for performance
- [ ] Comprehensive unit tests with 100% coverage of SSE spec
- [ ] No clippy warnings
- [ ] All tests passing

## Commands to Use

```bash
# Navigate to shadowcat
cd /Users/kevin/src/tapwire/shadowcat

# Create SSE module structure
mkdir -p src/transport/sse/tests

# Run tests for SSE module
cargo test sse

# Run all tests
cargo test

# Check compilation
cargo build

# Format code
cargo fmt

# Check for issues
cargo clippy --all-targets -- -D warnings

# When ready to commit (DO NOT commit unless explicitly asked)
git add -A
git status
```

## Implementation Details from Task File

### SSE Format to Support
- **Field Format**: `field: value\n`
- **Fields**: `data:`, `event:`, `id:`, `retry:`
- **Event dispatch**: Empty line (`\n\n`)
- **Comments**: Lines starting with `:`
- **BOM**: Ignore if present at stream start

### Core Types to Implement

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct SseEvent {
    pub id: Option<String>,
    pub event_type: String,  // Default: "message"
    pub data: String,
    pub retry: Option<u64>,
}

#[derive(Debug)]
pub enum SseField {
    Data(String),
    Event(String),
    Id(String),
    Retry(u64),
    Comment(String),
}

pub struct SseParser {
    buffer: Vec<u8>,
    current_event: EventBuilder,
    position: usize,
}
```

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
12. Commit changes with clear, descriptive messages (only when asked)

## Next Steps After This Task

Once Task 1.1 is complete:
- Update `/Users/kevin/src/tapwire/plans/mcp-compliance/compliance-tracker.md`
- Proceed to Task 1.2: SSE Connection Management
- Build on the parser to create connection management layer

## Performance Targets

Remember the project performance requirements:
- **Latency overhead**: < 5% p95 for typical tool calls
- **Memory usage**: < 100MB for 1000 concurrent sessions
- **Parse speed**: > 1GB/s for event parsing
- **Event latency**: < 1ms from receipt to emission

Good luck with the SSE parser implementation!