# Next Claude Session Prompt - Shadowcat Refactor Task 005

## Context

You are continuing the systematic refactoring of the Shadowcat Rust proxy codebase. **Phase 1 (Critical Safety) has been successfully completed**:
- Task 001: All 35 production unwrap calls eliminated ✅
- Task 002: Duplicate error types consolidated ✅  
- Task 003: Request size limits implemented ✅
- Task 004: Blocking I/O operations made async ✅

**🎉 Phase 1 Complete!** The codebase is now crash-resistant with a production readiness score of 95/100.

## Your Current Objective

**Start Phase 2 with Task 005: Implement Record Command**

Implement the `shadowcat record` command to capture MCP traffic and store it as "tapes" for later replay.

## Essential Context Files

Please read these files to understand your current task:

1. **Task Definition**: `/Users/kevin/src/tapwire/plans/refactors/task-005-implement-record.md` (create if it doesn't exist)
2. **Overall Refactor Plan**: `/Users/kevin/src/tapwire/plans/refactors/shadowcat-refactor-tracker.md`
3. **Original Review**: `/Users/kevin/src/tapwire/reviews/shadowcat-comprehensive-review-2025-08-06.md`

## Working Directory

`/Users/kevin/src/tapwire/shadowcat`

## Critical Development Practices

**IMPORTANT: Code Quality Standards**
- Run `cargo fmt` after every significant code change
- Run `cargo clippy -- -D warnings` before EVERY commit
- Both commands MUST pass with zero errors/warnings before committing

## What Has Been Accomplished

### ✅ Phase 1 Complete (All 4 Tasks)
- **Task 001**: 35 production unwraps eliminated, proper error handling throughout
- **Task 002**: Duplicate error types consolidated, clean error hierarchy  
- **Task 003**: Request size limits with DoS protection, comprehensive testing
- **Task 004**: Blocking I/O made async, tokio runtime optimized

**Result**: Crash-resistant codebase ready for feature implementation

### 📊 Current Status
- **349 tests passing**
- **Clean cargo fmt and clippy output**
- **No performance degradation**
- **Production readiness: 95/100**

## Your Task 005 Objectives

The `shadowcat record` command should work like this:
```bash
# Record stdio MCP session
shadowcat record --output session.tape -- echo '{"jsonrpc":"2.0","method":"ping","id":1}'

# Record HTTP MCP session
shadowcat record --output session.tape --transport http --port 8080

# Record with metadata
shadowcat record --name "Test Session" --description "Testing ping" --output session.tape -- echo '{"jsonrpc":"2.0","method":"ping","id":1}'
```

### Core Requirements

1. **CLI Integration**: Add record subcommand to main CLI with proper argument parsing
2. **Tape Storage**: Implement tape creation, writing, and metadata management
3. **Traffic Capture**: Intercept and record all MCP messages with timing information
4. **Multiple Transports**: Support both stdio and HTTP transport recording
5. **Metadata**: Store session metadata, timestamps, and replay information
6. **Error Handling**: Robust error handling with proper cleanup on interruption

### Success Criteria for Task 005

- [ ] `shadowcat record --help` shows comprehensive usage information
- [ ] `shadowcat record stdio -- command` records stdio MCP sessions
- [ ] `shadowcat record http --port 8080` records HTTP MCP sessions  
- [ ] Recorded tapes contain all request/response pairs with timing
- [ ] Tape metadata includes session info, timestamps, transport type
- [ ] Tapes are stored in SQLite database with proper schema
- [ ] Integration tests demonstrate end-to-end recording functionality
- [ ] Error handling covers interrupted sessions and storage failures
- [ ] All existing tests still pass
- [ ] `cargo fmt` and `cargo clippy -- -D warnings` pass

## Implementation Strategy

### 1. Examine Existing Code Structure
```bash
# Check current CLI structure
rg "enum.*Command" --type rust src/

# Review existing recorder module
find . -name "*.rs" | xargs grep -l "record\|tape\|TapeRecorder"

# Check transport implementations
ls src/transport/

# Review database schema
rg "CREATE TABLE" --type rust src/
```

### 2. Key Components to Implement/Extend

Based on existing codebase structure:

- **CLI Module**: Extend `src/main.rs` and CLI modules with record subcommand
- **Recorder Module**: Implement `TapeRecorder` trait and concrete implementations
- **Storage Layer**: SQLite-based tape storage with metadata
- **Transport Integration**: Hook recording into existing transport layer
- **Session Management**: Track and associate recorded messages

### 3. Implementation Phases

**Phase A: Core Infrastructure**
1. Add CLI subcommand structure for `record`
2. Implement basic `TapeRecorder` trait
3. Create SQLite schema for tape storage
4. Add integration with existing transport layer

**Phase B: Transport Integration** 
1. Implement stdio transport recording
2. Implement HTTP transport recording
3. Add timing information capture
4. Handle session lifecycle properly

**Phase C: Polish & Testing**
1. Add comprehensive error handling
2. Implement graceful shutdown on Ctrl-C
3. Add metadata management
4. Create integration tests
5. Update documentation

## Current Code Architecture Understanding

From previous tasks, we know:

- **Transport Layer**: Unified `Transport` trait with stdio/HTTP implementations
- **Session Management**: Session tracking with proper lifecycle
- **Error Handling**: Comprehensive `ShadowcatError` hierarchy
- **Storage**: SQLite integration via `sqlx`
- **CLI**: `clap`-based command structure

## Common Patterns to Follow

Based on completed Phase 1 tasks:

1. **Error Handling**: Use `Result<T, ShadowcatError>` consistently
2. **Async**: All I/O operations are async with proper error context
3. **Testing**: Comprehensive unit and integration tests
4. **Documentation**: Clear docstrings for public APIs
5. **Configuration**: Structured config with defaults and validation

## Commands to Use

```bash
# Check current CLI structure
cargo run -- --help
cargo run -- record --help  # (should fail initially)

# Examine existing recorder code
rg "TapeRecorder\|RecorderError" --type rust
find . -name "*.rs" -exec grep -l "record\|tape" {} \;

# Check transport implementations 
rg "impl.*Transport" --type rust src/

# Database exploration
rg "sqlx\|CREATE\|migration" --type rust

# Build and test
cargo check
cargo test
cargo clippy -- -D warnings
cargo fmt
```

## Current Phase Status

**Phase 1: Critical Safety ✅ (COMPLETED)**
- ✅ Task 001: Remove All Unwrap Calls  
- ✅ Task 002: Fix Duplicate Error Types
- ✅ Task 003: Add Request Size Limits
- ✅ Task 004: Fix Blocking IO in Async

**Phase 2: Core Features (0/5 tasks complete)**
- 🔄 Task 005: Implement Record Command (CURRENT)
- ⏳ Task 006: Implement Replay Command
- ⏳ Task 007: Implement Rate Limiting
- ⏳ Task 008: Complete Session Matching
- ⏳ Task 009: Implement Session Cleanup

## Important Notes

- **Always use TodoWrite tool** to track your progress through the task
- **Start with examining existing code** to understand current architecture
- **Follow established patterns** from Phase 1 implementation
- **Test incrementally** as you build each component
- **Run `cargo fmt`** after implementing new functionality
- **Run `cargo clippy -- -D warnings`** before any commit  
- **Update the refactor tracker** when Task 005 is complete
- **This starts Phase 2**: Focus shifts from safety to feature completeness

## Model Usage Guidelines

- **IMPORTANT** Be mindful of model capabilities. Assess whether Claude Opus or Claude Sonnet would be best for each step. When there's a benefit to a model change, pause and recommend it. Be mindful of the context window. When the context window has less than 15% availability, suggest creating a new Claude session and output a good prompt, referencing all available plans, tasks, and completion files that are relevant. Save the prompt into NEXT_SESSION_PROMPT.md.

## Development Workflow

1. Create todo list with TodoWrite tool
2. Examine existing codebase architecture and patterns
3. Study current CLI, recorder, and transport implementations
4. Design the record command interface and data flow
5. Implement core recording infrastructure incrementally
6. Add transport-specific recording capabilities
7. Implement tape storage and metadata management
8. Add comprehensive error handling and cleanup
9. Create integration tests demonstrating end-to-end functionality
10. Run tests after each significant change
11. Run `cargo fmt` and `cargo clippy -- -D warnings`
12. Update refactor tracker documentation
13. Commit with clear messages

## Expected Deliverables

By the end of Task 005:

1. **Working CLI Command**: `shadowcat record` with full argument parsing
2. **Stdio Recording**: Capture MCP traffic from stdio-based commands
3. **HTTP Recording**: Capture MCP traffic from HTTP endpoints
4. **Tape Storage**: SQLite-based storage with proper schema and metadata
5. **Integration Tests**: Demonstrate recording functionality works end-to-end
6. **Documentation**: Updated help text and usage examples
7. **Error Handling**: Robust handling of recording failures and interruptions

Begin by using the TodoWrite tool to create a comprehensive task list, then systematically examine the existing codebase to understand current patterns before implementing the record command functionality.