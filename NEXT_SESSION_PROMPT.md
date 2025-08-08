# Next Session: Phase 0 - Task F.1: Create Protocol Version Manager

## Context

We are implementing SSE proxy integration with MCP message handling capabilities in Shadowcat. The unified tracker (`plans/proxy-sse-message-tracker.md`) coordinates this work across 7 phases with 120-140 hours of effort.

### Current Status
- **Phase**: Phase 0 - Foundation Components (Week 1)
- **Task**: F.1 - Create Protocol Version Manager
- **Duration**: 2 hours
- **Dependencies**: None (this is the first task)

### What Has Been Completed
- Comprehensive planning and architecture design
- Created unified tracker interleaving SSE and MCP work
- Generated detailed task specifications
- Created foundation and glue task implementation guides
- Established integration coordination between initiatives

## Objective

Implement the Protocol Version Manager as the first foundation component. This will be the single source of truth for MCP protocol version handling throughout the codebase, supporting both 2025-03-26 (with batching) and 2025-06-18 versions.

## Essential Context Files to Read

1. **Primary Tracker**: `plans/proxy-sse-message-tracker.md`
2. **Task Details**: `plans/integration-tasks/foundation-tasks.md` (Task F.1 section)
3. **Integration Guide**: `plans/integration-coordination.md`
4. **Existing Protocol Module**: `shadowcat/src/protocol/` (to understand current implementation)

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Task Details

### Deliverables
1. Create `src/mcp/protocol.rs` with:
   - `ProtocolVersion` enum for 2025-03-26 and 2025-06-18
   - Version detection from headers
   - Capability detection (e.g., batching support)
   - Version negotiation logic
   - Default version handling (2025-03-26 for backwards compatibility)

2. Create comprehensive tests demonstrating:
   - Version parsing from strings
   - Header extraction
   - Default behavior
   - Capability detection
   - Version negotiation scenarios

### Implementation Strategy

#### Phase 1: Module Setup (15 min)
1. Create `src/mcp/` directory if it doesn't exist
2. Create `src/mcp/mod.rs` with module exports
3. Create `src/mcp/protocol.rs` file
4. Update `src/lib.rs` to include the mcp module

#### Phase 2: Core Implementation (60 min)
1. Implement `ProtocolVersion` enum with variants for each version
2. Add `FromStr` implementation for parsing version strings
3. Create version capability detection methods (supports_batching, etc.)
4. Implement `VersionNegotiator` for handling version negotiation
5. Add header extraction utilities

#### Phase 3: Testing (30 min)
1. Write unit tests for version parsing
2. Test default version behavior
3. Test version negotiation scenarios
4. Test capability detection

#### Phase 4: Integration (15 min)
1. Ensure the module compiles with the rest of the codebase
2. Run `cargo fmt` to format the code
3. Run `cargo clippy --all-targets -- -D warnings` to check for issues
4. Update the tracker with completion status

## Commands to Use

```bash
# Create the MCP module directory
mkdir -p src/mcp

# Run tests as you implement
cargo test mcp::protocol

# Format your code
cargo fmt

# Check for issues
cargo clippy --all-targets -- -D warnings

# Run all tests to ensure nothing broke
cargo test
```

## Success Criteria Checklist

- [ ] `src/mcp/protocol.rs` created with full implementation
- [ ] `ProtocolVersion` enum supports both 2025-03-26 and 2025-06-18
- [ ] Version parsing from strings works correctly
- [ ] Default version is 2025-03-26 for backwards compatibility
- [ ] `supports_batching()` returns true only for 2025-03-26
- [ ] `VersionNegotiator` can negotiate between client and server versions
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code is properly formatted
- [ ] Tracker updated with completion status

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

## Example Implementation Reference

The task details in `plans/integration-tasks/foundation-tasks.md` provide a complete implementation example. Use this as a guide, but adapt based on the existing codebase patterns and any specific requirements discovered during implementation.

## Next Steps After This Task

Once F.1 is complete, the next task will be:
- **F.2**: Build Minimal MCP Parser (4 hours, no dependencies)

This will build on the Protocol Version Manager to create a lightweight parser for extracting basic MCP message information.

---

**Session Goal**: Complete the Protocol Version Manager implementation with comprehensive tests and documentation, setting a solid foundation for all subsequent SSE and MCP work.