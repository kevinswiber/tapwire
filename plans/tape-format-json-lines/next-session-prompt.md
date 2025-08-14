# Next Session: Phase 1 - Design & Analysis

## Project Context

This project migrates Shadowcat's tape recording format from monolithic JSON files to a streaming-friendly JSON Lines format, enabling better memory efficiency, streaming capabilities, and resilience for long-running MCP session recordings.

**Project**: Tape Format JSON Lines Migration
**Tracker**: `plans/tape-format-json-lines/tape-format-json-lines-tracker.md`
**Status**: Phase 1 - Design & Analysis (0% Complete)

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
- **Initial Assessment** (✅ Completed 2025-08-13)
  - Identified memory efficiency issues with current JSON format
  - Analyzed JSON Lines benefits for streaming
  - Created tracker and task structure

### What's In Progress
- **Phase 1: Design & Analysis** (Not Started)
  - Duration: 9 hours total
  - Dependencies: None

## Your Mission

Complete the initial design and analysis phase for the JSON Lines tape format migration. This session focuses on defining the exact format specification, analyzing performance implications, and designing the migration strategy.

### Priority 1: Format Specification & Performance Analysis (5 hours)

1. **Task 1.1: Format Specification** (2h)
   - Define exact JSON Lines schema for tape recordings
   - Document header, frame, correlation, and footer formats
   - Create example files showing the new format
   
2. **Task 1.2: Performance Analysis** (3h)
   - Benchmark current JSON implementation
   - Estimate memory savings with JSON Lines
   - Measure append performance improvements

### Priority 2: Migration Strategy & API Design (4 hours)

1. **Task 1.3: Migration Strategy** (2h)
   - Design backward compatibility approach
   - Plan incremental migration path
   - Define rollback procedures
   
2. **Task 1.4: API Design** (2h)
   - Design new TapeWriter/TapeReader interfaces
   - Plan streaming API methods
   - Document breaking changes

## Essential Context Files to Read

1. **Primary Tracker**: `plans/tape-format-json-lines/tape-format-json-lines-tracker.md` - Full project context
2. **Task Details**: 
   - `plans/tape-format-json-lines/tasks/1.1-format-specification.md`
   - `plans/tape-format-json-lines/tasks/1.2-performance-analysis.md`
3. **Current Implementation**: 
   - `shadowcat-tape-format-json-lines/src/recorder/tape.rs` - Existing tape structure
   - `shadowcat-tape-format-json-lines/src/recorder/storage.rs` - Storage implementation
4. **Assessment**: `plans/tape-format-json-lines/analysis/assessment.md` - Initial findings

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

### Phase 1: Setup & Review (30 min)
1. Navigate to worktree directory
2. Review current tape.rs and storage.rs implementations
3. Understand existing tape format structure
4. Set up benchmark framework

### Phase 2: Format Specification (2 hours)
1. Design JSON Lines schema for all record types
2. Create example tape files in new format
3. Document format in `analysis/format-specification.md`
4. Define validation rules

### Phase 3: Performance Analysis (3 hours)
1. Create benchmarks for current implementation
2. Simulate JSON Lines performance
3. Measure memory usage patterns
4. Document findings in `analysis/performance-analysis.md`

### Phase 4: Strategy & Design (3 hours)
1. Design migration tool approach
2. Define backward compatibility layer
3. Create API design document
4. Update tracker with findings

### Phase 5: Cleanup & Documentation (30 min)
1. Run formatters and linters
2. Update tracker with completion status
3. Update this NEXT_SESSION_PROMPT.md for next phase

## Success Criteria Checklist

- [ ] Format specification document created with complete schema
- [ ] Performance benchmarks completed and documented
- [ ] Migration strategy defined with clear steps
- [ ] API design document with interface definitions
- [ ] All analysis documents in `analysis/` directory
- [ ] Tracker updated with Phase 1 completion
- [ ] No clippy warnings in any new code

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

## Key Design Considerations

1. **Streaming Efficiency**: Must support reading/writing without loading entire file
2. **Backward Compatibility**: Must provide migration path from existing JSON tapes
3. **Corruption Resilience**: Partial corruption should only affect damaged lines
4. **Performance**: Target < 10MB memory for 1M+ frame tapes
5. **Atomicity**: Each line must be written atomically

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

## Next Steps After This Task

Once Phase 1 is complete:
- **Task 2.1**: Streaming Writer Implementation (4 hours, depends on 1.1, 1.4)
- **Task 2.2**: Streaming Reader Implementation (4 hours, depends on 1.1, 1.4)

After completing Phase 2:
- Move to Phase 3 - Migration & Compatibility

## Model Usage Guidelines

- **IMPORTANT**: Be mindful of model capabilities. When context window has less than 15% availability, suggest creating a new session and save prompt to NEXT_SESSION_PROMPT.md

## Session Time Management

**Estimated Session Duration**: 8-9 hours
- Setup & Context: 30 min
- Format Specification: 2 hours
- Performance Analysis: 3 hours
- Strategy & Design: 3 hours
- Documentation: 30 min

## Related Context

- **Integration Points**: Recorder module, Storage layer, CLI replay commands
- **Downstream Dependencies**: All tape recording and playback functionality
- **Parallel Work**: Can design while main development continues

---

**Session Goal**: Complete Phase 1 design and analysis with comprehensive documentation of the JSON Lines format specification and migration strategy.

**Last Updated**: 2025-08-14
**Next Review**: After Phase 1 completion

## Worktree Reminder

**Remember**: Every command, every edit, every test MUST be run in:
```bash
cd shadowcat-tape-format-json-lines
```