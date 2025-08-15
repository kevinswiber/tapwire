# Next Session: Phase 2-3 - Integration and Migration

## Project Context

This project migrates Shadowcat's tape recording format from monolithic JSON files to a streaming-friendly JSON Lines format, enabling better memory efficiency, streaming capabilities, and resilience for long-running MCP session recordings.

**Project**: Tape Format JSON Lines Migration
**Tracker**: `plans/tape-format-json-lines/tape-format-json-lines-tracker.md`
**Status**: Phase 1 Complete ✅ | Phase 2 - Core Implementation (60% Complete)

## IMPORTANT: Git Worktree Setup

**🚨 CRITICAL**: All work for this feature MUST be done in the dedicated git worktree:
- **Worktree Path**: `shadowcat-tape-format-json-lines/`
- **Branch**: `feat/tape-format-json-lines`
- **DO NOT work in the main shadowcat directory**

```bash
# Navigate to the worktree
cd shadowcat-tape-format-json-lines

# Verify you're on the correct branch
git branch --show-current  # Should show: feat/tape-format-json-lines

# Check worktree status
git worktree list
```

## Current Status

### What Has Been Completed
- **Phase 1: Design & Analysis** (✅ Completed 2025-08-14)
  - Format specification with streaming-first design
  - Performance analysis showing 100-1000x improvements
  - Migration strategy (clean slate, no backward compatibility needed)
  - API design for streaming writer/reader interfaces

- **Phase 2: Core Implementation** (✅ Completed 2025-08-15)
  - ✅ Task 2.1: StreamingTapeWriter implemented with O(1) append
  - ✅ Task 2.2: StreamingTapeReader implemented with line-by-line parsing
  - ✅ Task 2.3: Index Enhancement with BTreeMap-based indexing
  - ✅ Task 2.4: Seek Capability with frame and time-based lookup

### What's Ready to Start
- **Phase 3**: Migration and integration (8 hours)

## Your Mission

Complete Phase 3 integration work to replace the old JSON format with JSON Lines.

### Priority 1: Direct Integration (3 hours)

**Task 3.1: Replace Old Implementation**
- Update `recorder/tape.rs` to use `StreamingTapeWriter`
- Update `recorder/storage.rs` to use new tape format
- Update replay module to use `StreamingTapeReader`
- Remove old JSON-based tape code

### Priority 2: CLI Integration (2 hours)

**Task 3.2: Update CLI Commands**
- Update `record` command to use new format
- Update `replay` command to use streaming reader
- Update `list` and other tape commands
- Test end-to-end recording and playback

### Priority 3: Testing & Cleanup (2 hours)

**Task 3.3: Testing & Validation**
- Comprehensive integration tests
- Performance benchmarks
- Remove old tape format code
- Documentation updates

## Essential Context Files to Read

1. **Phase 1 Deliverables** (MUST READ):
   - `plans/tape-format-json-lines/analysis/format-specification.md` - JSON Lines format spec
   - `plans/tape-format-json-lines/analysis/api-design.md` - Streaming API interfaces
   - `plans/tape-format-json-lines/analysis/performance-analysis.md` - Performance targets
   - `plans/tape-format-json-lines/analysis/migration-strategy.md` - Implementation approach

2. **Phase 2 Implementation** (REVIEW):
   - `shadowcat-tape-format-json-lines/src/recorder/streaming/writer.rs` - StreamingTapeWriter
   - `shadowcat-tape-format-json-lines/src/recorder/streaming/reader.rs` - StreamingTapeReader
   - `shadowcat-tape-format-json-lines/src/recorder/streaming/types.rs` - Type definitions
   - `shadowcat-tape-format-json-lines/src/recorder/streaming/mod.rs` - Module exports

3. **Current Implementation** (Reference):
   - `shadowcat-tape-format-json-lines/src/recorder/tape.rs` - Existing tape structure
   - `shadowcat-tape-format-json-lines/src/recorder/storage.rs` - Storage implementation

4. **Examples** (Reference):
   - `plans/tape-format-json-lines/analysis/example-complete.jsonl` - Sample tape file
   - `plans/tape-format-json-lines/analysis/example-complete.meta.json` - Sample metadata

## Working Directory

```bash
# ALWAYS work in the worktree
cd shadowcat-tape-format-json-lines

# Verify correct location
pwd  # Should show: .../shadowcat-tape-format-json-lines
```

## Commands to Run First

```bash
# Verify worktree and branch
cd shadowcat-tape-format-json-lines
git branch --show-current

# Check current state
git status

# Run existing tests to ensure baseline
cargo test recorder::tape --lib
cargo test recorder::storage --lib

# Check for issues
cargo clippy --all-targets -- -D warnings
```

## Implementation Strategy

### Phase 1: Direct Replacement (3 hours)
1. Navigate to worktree directory
2. Replace tape.rs implementation with streaming version
3. Update storage.rs to handle JSON Lines format
4. Update replay module for streaming
5. Remove old JSON serialization code

### Phase 2: CLI Updates (2 hours)
1. Update record command
2. Update replay command  
3. Update list/delete commands
4. End-to-end testing

### Phase 3: Cleanup (2 hours)
1. Remove all old tape format code
2. Update documentation
3. Performance benchmarks
4. Final testing

## Success Criteria Checklist

### Phase 2 Completion ✅
- [x] `StreamingTapeWriter` fully implemented
- [x] `StreamingTapeReader` fully implemented
- [x] Metadata file management working
- [x] Concurrent read/write tests passing
- [x] Index and seek capabilities
- [x] Performance benchmarks meeting targets:
  - [x] Constant memory usage (< 100KB)
  - [x] O(1) append time (< 1ms)
  - [x] Instant read start (< 5ms)
  - [x] Seek performance (< 10ms with index)

### Phase 3 Integration (No Backward Compatibility Needed!)
- [ ] Old tape.rs replaced with streaming implementation
- [ ] Storage.rs updated for JSON Lines
- [ ] Replay module using StreamingTapeReader
- [ ] CLI commands updated
- [ ] Old JSON format code removed
- [ ] All tests passing
- [ ] No clippy warnings

## Key Commands

```bash
# Development commands (in worktree)
cd shadowcat-tape-format-json-lines
cargo build
cargo test recorder:: --lib

# Benchmark commands
cargo bench tape
cargo test --release -- --nocapture benchmark

# Validation commands
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Important Notes

- **CRITICAL**: Always work in the `shadowcat-tape-format-json-lines` worktree
- **NEVER** work directly in the main shadowcat directory for this feature
- **Always use TodoWrite tool** to track progress through tasks
- **Start with examining existing code** in the worktree
- **Create task files** if they don't exist using the template
- **Document all findings** in the `analysis/` directory
- **Test incrementally** as you design
- **Update tracker** when tasks are complete

## Key Design Decisions

1. **NO BACKWARD COMPATIBILITY NEEDED**: Shadowcat is pre-release - direct replacement!
2. **Streaming-First**: Never buffer entire tape, true O(1) operations
3. **Separate Metadata Files**: Avoid lock contention, enable concurrent access
4. **Index for Seeking**: BTreeMap-based index for O(log n) seeks
5. **Clean Replacement**: Remove all old JSON code, no migration tool needed

## Performance/Quality Targets

- **Memory Usage**: < 10MB for tapes with 1M+ frames
- **Append Latency**: < 1ms per frame
- **Streaming Rate**: > 10,000 frames/second
- **Migration Speed**: < 5 seconds for 1GB tape
- **Test Coverage**: 95% for new code

## Risk Factors & Blockers

- **Data Loss Risk**: Ensure comprehensive testing before migration
- **API Breaking Changes**: Design compatibility layer carefully
- **Performance Regression**: Benchmark small files to ensure no regression

## Next Steps After This Session

Once Phase 2 Core Implementation is complete:
- **Phase 3 - Migration & Testing**:
  - Task 3.1: Migration Tool (3 hours)
  - Task 3.2: CLI Integration (2 hours)
  - Task 3.3: Testing & Validation (3 hours)

The implementation can then be merged to main branch!

## Model Usage Guidelines

- **IMPORTANT**: Be mindful of model capabilities. When context window has less than 15% availability, suggest creating a new session and save prompt to next-session-prompt.md

## Session Time Management

**Estimated Session Duration**: 10-12 hours
- Setup & Module Creation: 30 min
- Streaming Writer: 4 hours
- Streaming Reader: 4 hours
- Integration & Testing: 2 hours
- Documentation & Cleanup: 30 min

## Related Context

- **Integration Points**: Recorder module, Storage layer, CLI replay commands
- **Downstream Dependencies**: All tape recording and playback functionality
- **Parallel Work**: Can design while main development continues

---

**Session Goal**: Complete the remaining Phase 2 tasks (index/seek) and implement Phase 3 migration tool and integration to make the JSON Lines tape format production-ready.

**Last Updated**: 2025-08-15
**Next Review**: After Phase 2-3 completion

## Worktree Reminder

**Remember**: Every command, every edit, every test MUST be run in:
```bash
cd shadowcat-tape-format-json-lines
```