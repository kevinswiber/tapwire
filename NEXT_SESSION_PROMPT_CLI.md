# Next Session: CLI Refactor - Continue Phase 3 Command Migration

## Context
We're refactoring Shadowcat's main.rs into a modular CLI structure. Phase 1 (Analysis), Phase 2 (Core Infrastructure), and Phase 3 Task C.1 (Forward Proxy) are complete. We're continuing with the remaining command migrations.

## Current Status
- ✅ Phase 1 Complete: Analysis & Design (Tasks A.1-A.3)
- ✅ Phase 2 Complete: Core Infrastructure (Tasks B.1-B.3)
- ✅ Phase 3 C.1 Complete: Forward proxy migration
  - Moved 319 lines from main.rs to cli/forward.rs
  - main.rs reduced from 1294 to 975 lines
- 🎯 Next: C.2-C.4 (Reverse, Record, Replay migrations)
- ⏳ Target: Reduce main.rs to < 200 lines

## Session Accomplishments So Far
- Created comprehensive common utilities module (605 lines)
- Migrated forward proxy commands successfully (328 lines)
- All tests passing, no clippy warnings
- Established command migration pattern with ForwardCommand::execute()

## Your Tasks for This Session

### Task C.2: Migrate reverse proxy command (1.5 hours)
1. Move `run_reverse_proxy()` from main.rs (lines ~221-330) to cli/reverse.rs
2. Update ReverseCommand with ProxyConfig fields
3. Implement ReverseCommand::execute()
4. Update main.rs to delegate to ReverseCommand
5. Add unit tests for ReverseCommand

### Task C.3: Migrate record commands (1.5 hours)
1. Move stdio recording logic (lines ~361-492) to cli/record.rs
2. Move HTTP recording logic (lines ~494-618) to cli/record.rs
3. Update RecordCommand with appropriate configuration
4. Implement RecordCommand::execute() with transport variants
5. Update main.rs to delegate to RecordCommand
6. Add unit tests

### Task C.4: Migrate replay command (1 hour)
1. Move `run_replay()` from main.rs (lines ~620-779) to cli/replay.rs
2. Update ReplayCommand with configuration
3. Implement ReplayCommand::execute()
4. Update main.rs to delegate to ReplayCommand
5. Add unit tests

## Key Files
- **Main file**: `shadowcat-cli-refactor/src/main.rs` (975 lines, target < 200)
- **Common utilities**: `shadowcat-cli-refactor/src/cli/common.rs` (605 lines, complete)
- **Forward module**: `shadowcat-cli-refactor/src/cli/forward.rs` (328 lines, complete)
- **Reverse stub**: `shadowcat-cli-refactor/src/cli/reverse.rs` (50 lines, needs implementation)
- **Record stub**: `shadowcat-cli-refactor/src/cli/record.rs` (134 lines, needs implementation)
- **Replay stub**: `shadowcat-cli-refactor/src/cli/replay.rs` (58 lines, needs implementation)

## Migration Pattern (Established in C.1)
1. Add config fields to command struct (rate limiting, session config)
2. Move function implementation to module
3. Update imports from `shadowcat::` to `crate::`
4. Create execute() method that builds ProxyConfig
5. Update main.rs Commands enum to use new command
6. Remove old function and enum from main.rs
7. Fix imports and run clippy
8. Add unit tests

## Testing Commands
```bash
# Test reverse proxy
cargo run -- reverse --bind 127.0.0.1:8080 --upstream http://localhost:3000

# Test recording
cargo run -- record stdio --output test.tape -- echo
cargo run -- record http --port 8080 --target http://localhost:3000

# Test replay
cargo run -- replay test.tape --port 8080

# Run tests
cargo test --lib cli::reverse
cargo test --lib cli::record
cargo test --lib cli::replay

# Check for warnings
cargo clippy --all-targets -- -Dwarnings
```

## Success Criteria
- [ ] All commands migrated to their modules
- [ ] main.rs reduced to < 200 lines (currently 975)
- [ ] All existing CLI commands still work
- [ ] No clippy warnings
- [ ] Unit tests for each command
- [ ] Tracker updated with progress

## Important Notes
- Follow the pattern established in ForwardCommand migration
- Preserve all existing CLI argument names and defaults
- Each command should include ProxyConfig fields where applicable
- Use `crate::` imports instead of `shadowcat::`
- Update cli/mod.rs exports as needed

## Git Strategy
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor
# Already on branch shadowcat-cli-refactor

# Commit after each task
git add -A
git commit -m "feat(cli): migrate reverse proxy command to module"
# Repeat for record and replay
```

## Next Steps After Phase 3
Once all commands are migrated:
- Phase 4 D.1: Add comprehensive integration tests
- Phase 4 D.2: Final cleanup of main.rs
- Phase 4 D.3: Documentation and performance validation

## References
- [Full tracker](plans/cli-refactor/cli-refactor-tracker.md)
- [Phase 3 details](plans/cli-refactor/tasks/C.2-reverse-proxy.md)
- [Forward migration example](shadowcat-cli-refactor/src/cli/forward.rs)

Start with Task C.2 (reverse proxy) and continue systematically through C.3 and C.4. The goal is to reduce main.rs by another ~550 lines through these migrations. Good luck!