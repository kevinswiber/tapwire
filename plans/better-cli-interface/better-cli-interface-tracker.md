# Better CLI Interface Tracker

## Overview

This tracker coordinates the improvement of Shadowcat's CLI interface to provide a more intuitive developer experience. The plan implements a hybrid approach with smart auto-detection for common cases while maintaining explicit control when needed.

**Last Updated**: 2025-01-14  
**Total Estimated Duration**: 16-24 hours  
**Status**: Planning

## Goals

1. **Intuitive Default Experience** - Make `shadowcat my-server` just work without flags
2. **Clear Mental Model** - Use familiar "forward" and "gateway" terminology instead of forward/reverse proxy
3. **Discoverability** - Improve help text and error messages to guide users

## Architecture Vision

```
Current State:
shadowcat forward [options]  → Works but paired with confusing "reverse"
shadowcat reverse [options]  → Technical term most developers don't know

Future State:
shadowcat my-server          → Auto-detects forward proxy mode
shadowcat :8080             → Auto-detects gateway mode
shadowcat session.tape      → Auto-detects replay mode

With explicit control:
shadowcat forward [options]  → Forward proxy (client → shadowcat → server)
shadowcat gateway [options]  → API gateway (client → shadowcat → backends)
shadowcat record/replay      → Session management
```

## Work Phases

### Phase A: Analysis & Design (Week 1)
Understand current CLI structure and design the new interface

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | **Current CLI Analysis** | 2h | None | ⬜ Not Started | | [Details](tasks/A.0-current-cli-analysis.md) |
| A.1 | **User Experience Research** | 2h | None | ⬜ Not Started | | [Details](tasks/A.1-user-experience-research.md) |
| A.2 | **Design Proposal** | 3h | A.0, A.1 | ⬜ Not Started | | [Details](tasks/A.2-design-proposal.md) |
| A.3 | **Implementation Plan** | 2h | A.2 | ⬜ Not Started | | [Details](tasks/A.3-implementation-plan.md) |

**Phase A Total**: 9 hours

### Phase B: Core Implementation (Week 1-2)
Implement the smart detection and new command structure

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.1 | **Refactor CLI Module Structure** | 3h | A.3 | ⬜ Not Started | | [Details](tasks/B.1-refactor-cli-structure.md) |
| B.2 | **Implement Smart Detection** | 4h | B.1 | ⬜ Not Started | | [Details](tasks/B.2-implement-smart-detection.md) |
| B.3 | **Add Forward/Gateway Commands** | 3h | B.1 | ⬜ Not Started | | [Details](tasks/B.3-add-forward-gateway.md) |
| B.4 | **Update Help System** | 2h | B.2, B.3 | ⬜ Not Started | | [Details](tasks/B.4-update-help-system.md) |

**Phase B Total**: 12 hours

### Phase C: Testing & Polish (Week 2)
Ensure quality and user experience

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | **Integration Tests** | 2h | B.2, B.3 | ⬜ Not Started | | [Details](tasks/C.1-integration-tests.md) |
| C.2 | **Error Messages & UX** | 2h | B.2, B.3 | ⬜ Not Started | | [Details](tasks/C.2-error-messages.md) |
| C.3 | **Documentation Update** | 1h | C.1, C.2 | ⬜ Not Started | | [Details](tasks/C.3-documentation.md) |

**Phase C Total**: 5 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (Jan 14-21)
- [ ] A.0: Current CLI Analysis
- [ ] A.1: User Experience Research
- [ ] A.2: Design Proposal
- [ ] A.3: Implementation Plan
- [ ] B.1: Refactor CLI Module Structure

### Week 2 (Jan 21-28)
- [ ] B.2: Implement Smart Detection
- [ ] B.3: Add Forward/Gateway Commands
- [ ] B.4: Update Help System
- [ ] C.1: Integration Tests
- [ ] C.2: Error Messages & UX
- [ ] C.3: Documentation Update

### Completed Tasks
(None yet)

## Success Criteria

### Functional Requirements
- ⬜ Smart detection works for common cases (executable, port, file)
- ⬜ Explicit commands (forward/gateway) work as expected
- ⬜ Help text clearly explains usage patterns
- ⬜ Error messages guide users to correct usage
- ⬜ "reverse" command shows helpful warning and continues as "gateway"

### User Experience Requirements
- ⬜ `shadowcat my-server` works without flags (forward proxy)
- ⬜ `shadowcat :8080` starts gateway mode
- ⬜ `shadowcat session.tape` replays recording
- ⬜ `shadowcat --help` shows primary workflows clearly
- ⬜ `shadowcat reverse` shows educational warning but still works

### Quality Requirements
- ⬜ All existing tests still pass
- ⬜ New tests for smart detection logic
- ⬜ No clippy warnings
- ⬜ Documentation updated in README and help text

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Breaking existing scripts/workflows | HIGH | Clear error messages pointing to new commands | Planned |
| Smart detection ambiguity | MEDIUM | Provide explicit connect/serve commands | Planned |
| Complex regex/parsing for detection | LOW | Use simple heuristics first, enhance later | Planned |
| User retraining needed | MEDIUM | Excellent help text and examples | Planned |

## Session Planning Guidelines

### Next Session Prompt
Each plan should have a corresponding `next-session-prompt.md` file in the same directory as this tracker, based on the template in `plans/template/next-session-prompt.md`. This file should be updated at the end of each session to set up the next session with proper context.

### Optimal Session Structure
1. **Review** (5 min): Check this tracker and relevant task files
2. **Implementation** (2-3 hours): Complete the task deliverables
3. **Testing** (30 min): Run tests, fix issues
4. **Documentation** (15 min): Update tracker, create PR if needed
5. **Handoff** (10 min): Update next-session-prompt.md in this plan directory

### Context Window Management
- Each task is designed to require < 50% context window
- If approaching 70% usage, create NEXT_SESSION_PROMPT.md
- Keep focus on single task to avoid context bloat
- Reference documentation only when needed

### Task Completion Criteria
- [ ] All deliverables checked off
- [ ] Tests passing
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Tracker status updated

## Handling Common Proxy Terminology

### Supporting "reverse" as a Helpful Redirect
Since many developers are familiar with "reverse proxy" terminology, we'll catch when users type `shadowcat reverse` and helpfully redirect them to the clearer `gateway` command:

1. Show an educational note
2. Continue execution as `gateway` 
3. Help users learn the preferred terminology

**Message Format:**
```
Note: 'reverse' has been renamed to 'gateway' for clarity.
  Example: shadowcat gateway --port 8080
[Continuing as 'gateway'...]
```

### Implementation Example
```rust
match command {
    "reverse" => {
        eprintln!("Note: 'reverse' has been renamed to 'gateway' for clarity.");
        eprintln!("  Example: shadowcat gateway {}", args.join(" "));
        eprintln!("[Continuing as 'gateway'...]\n");
        
        // Log for telemetry
        log::info!("User used 'reverse' command, redirecting to 'gateway'");
        
        Command::Gateway(args)
    }
    "gateway" => Command::Gateway(args),
    // ...
}
```

## Critical Implementation Guidelines

### CLI Design Principles
**ALWAYS maintain these principles:**
- **Progressive Disclosure**: Simple things simple, complex things possible
- **Fail Gracefully**: Clear error messages with suggestions
- **Consistency**: Similar patterns across all commands
- **Discoverability**: Users should be able to explore via --help

### Implementation Checklist
When implementing CLI changes:
1. ✅ Update clap command definitions
2. ✅ Update help text and examples
3. ✅ Add/update integration tests
4. ✅ Update documentation (README, help)
5. ✅ Test all command variations
6. ✅ Verify backward compatibility

## Communication Protocol

### Status Updates
After completing each task, update:
1. Task status in this tracker
2. Completion date and notes
3. Any new issues discovered
4. Next recommended task

### Handoff Notes
If context window becomes limited:
1. Save progress to NEXT_SESSION_PROMPT.md
2. Include:
   - Current task status
   - Completed deliverables
   - Remaining work
   - Any blockers or decisions needed

## Related Documents

### Primary References
- [Shadowcat Architecture](../002-shadowcat-architecture-plan.md)
- [Developer Guide](../003-shadowcat-developer-guide.md)
- [CLI Reference](../../shadowcat/README.md#cli-usage)

### Task Files
- [Analysis Tasks](tasks/)
- Task files follow the structure defined in `plans/template/task.md`

### Specifications
- [MCP Protocol Spec](https://spec.modelcontextprotocol.io)
- [Clap Documentation](https://docs.rs/clap/latest/clap/)

## Next Actions

1. **Begin Phase A analysis** - Understand current implementation
2. **Research CLI patterns** - Look at successful tools
3. **Create detailed design** - Document the new interface

## Notes

- The goal is to make the common case magical while keeping explicit control available
- "forward" stays the same, "reverse" becomes "gateway" for clarity
- "Gateway" is much more intuitive than "reverse proxy" for most developers
- Smart detection should be simple and predictable, not overly clever
- Consider how this will work with future transport types (WebSocket, etc.)

---

**Document Version**: 1.0  
**Created**: 2025-01-14  
**Last Modified**: 2025-01-14  
**Author**: Kevin

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-01-14 | 1.0 | Initial plan creation | Kevin |