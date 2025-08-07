# Next Claude Session Prompt - Shadowcat Refactor Task 007

## Context

You are continuing the systematic refactoring of the Shadowcat Rust proxy codebase. **Phase 1 (Critical Safety) has been successfully completed**:
- Task 001: All 35 production unwrap calls eliminated ✅
- Task 002: Duplicate error types consolidated ✅  
- Task 003: Request size limits implemented ✅
- Task 004: Blocking I/O operations made async ✅

**Phase 2 Progress**: 2/5 tasks complete:
- Task 005: Record Command fully implemented ✅
- Task 006: Replay Command fully implemented ✅

**🎉 Task 006 Complete!** Replay command is fully functional with tape ID/file path loading, HTTP server replay, and request matching.

## Your Current Objective

**Continue Phase 2 with Task 007: Implement Rate Limiting**

Implement rate limiting functionality to prevent abuse and ensure fair resource allocation.

## Essential Context Files

Please read these files to understand your current task:

1. **Task Definition**: `/Users/kevin/src/tapwire/plans/refactors/task-007-implement-rate-limiting.md`
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

### ✅ Task 005 Complete (Record Command)
- **CLI Interface**: Full stdio and HTTP recording with comprehensive args
- **Tape Storage**: Complete tape data with request/response pairs and timing
- **Rich Metadata**: Session info, timestamps, transport type, frame counts
- **Integration**: Works seamlessly with existing tape management system
- **Error Handling**: Comprehensive error handling and cleanup
- **Testing**: 4 new integration tests + all 349 tests passing

### ✅ Task 006 Complete (Replay Command)
- **CLI Interface**: Replay by tape ID or file path with port configuration
- **Tape Loading**: Supports both UUID tape IDs and file paths
- **HTTP Server**: Axum-based server serving replayed responses
- **Request Matching**: Matches incoming requests to tape frames by method
- **Error Handling**: Robust handling of missing/invalid tapes
- **Testing**: 4 integration tests demonstrating full record->replay flow

**Result**: Complete record/replay functionality working end-to-end

### 📊 Current Status
- **353 tests passing** (4 new replay tests added)
- **Clean cargo fmt and clippy output**
- **Production readiness: 97/100** ⬆️ (+1 point)
- **Complete record/replay functionality** working

### 🎯 Working Record/Replay Examples
```bash
# Record commands (all working)
shadowcat record stdio --output demo.tape --name "Demo" --description "Test" -- echo '{"jsonrpc":"2.0","method":"ping","id":1}'
shadowcat record http --output http.tape --port 8081

# Replay commands (all working)
shadowcat replay ef510f7f-1de3-426e-b3b6-66f0b16141d6 --port 8080  # By tape ID
shadowcat replay ./tapes/demo.json --port 8081                       # By file path

# Test replay
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"ping","id":1}' http://localhost:8080/

shadowcat tape list  # Shows all recorded tapes
```

## Your Task 007 Objectives

Rate limiting is partially stubbed but needs to be connected and made functional.

### Core Requirements for Task 007

1. **Connect Rate Limiter**: Wire up existing `MultiTierRateLimiter` to proxy paths
2. **Configuration**: Add rate limiting configuration to CLI and config files
3. **Middleware Integration**: Add rate limiting middleware to HTTP paths
4. **Per-Client Tracking**: Track rates per IP/session/user as appropriate
5. **Metrics**: Expose rate limiting metrics and status

### Success Criteria for Task 007

- [ ] Rate limiting enforces configured limits on forward proxy
- [ ] Rate limiting enforces configured limits on reverse proxy
- [ ] Rate limiting enforces configured limits on replay server
- [ ] Per-IP rate limiting works correctly
- [ ] Per-session rate limiting works correctly
- [ ] Rate limit exceeded returns appropriate HTTP 429 responses
- [ ] Metrics show rate limiting statistics
- [ ] Integration tests verify rate limiting behavior
- [ ] All existing tests still pass
- [ ] `cargo fmt` and `cargo clippy -- -D warnings` pass

## Implementation Strategy

### 1. Examine Existing Infrastructure
```bash
# Check what's available for replay
rg "TapePlayer\|ReplayTransport" --type rust src/
rg "load_tape" --type rust src/

# Review existing HTTP server patterns from record command
rg "axum\|Router" --type rust src/main.rs

# Check available test tapes
ls -la tapes/
shadowcat tape list
```

### 2. Key Components to Use/Extend

Based on Task 005 completion and existing infrastructure:

- **CLI Module**: Enhance existing `Commands::Replay` in `src/main.rs`
- **TapeRecorder**: Use `load_tape()` method for tape loading
- **TapePlayer**: Use existing `src/recorder/replay.rs` for playback control
- **HTTP Server**: Follow existing axum patterns from record command
- **Transport Layer**: Leverage existing transport abstractions

### 3. Implementation Strategy

**Phase A: CLI Enhancement**
1. Update `Commands::Replay` args to match requirements
2. Implement `run_replay_server()` function using existing patterns from Task 005

**Phase B: Core Replay Logic**
1. Use existing `TapeRecorder::load_tape()` to load tapes
2. Create HTTP server using axum (same as record command)
3. Use `TapePlayer` for playback control and timing
4. Handle tape ID vs file path resolution

**Phase C: Integration & Testing**
1. Test with tapes created by record command
2. Add integration tests demonstrating record -> replay flow
3. Add error handling tests

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