# Tracker Template

<!-- INSTRUCTIONS (DO NOT COPY THESE TO YOUR TRACKER):
This template provides a standard structure for development trackers in the Shadowcat project.
When creating a new tracker:
1. Copy everything BELOW the "START OF TEMPLATE" marker
2. Replace all placeholders marked with {PLACEHOLDER_NAME}
3. Customize sections as needed for your specific project
4. Delete any sections that don't apply
5. Add project-specific sections as needed

Key principles:
- Each task should be completable in one Claude session (2-4 hours)
- Dependencies should be clearly marked
- Status tracking should be consistent
- Include both functional and quality requirements
- Always consider both forward and reverse proxy modes
END OF INSTRUCTIONS -->

<!-- ==================== START OF TEMPLATE ==================== -->

# {PROJECT_NAME} Tracker

## Overview

{Brief description of what this tracker coordinates and why it exists}

**Last Updated**: {DATE}  
**Total Estimated Duration**: {X-Y} hours  
**Status**: {Planning | In Progress | Blocked | Complete}

## Goals

1. **{Primary Goal}** - {Brief description}
2. **{Secondary Goal}** - {Brief description}
3. **{Additional goals as needed}**

## Architecture Vision

```
{ASCII or simple diagram showing the architecture}
```

## Work Phases

### Phase {N}: {Phase Name} (Week {X})
{Brief description of what this phase accomplishes}

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| {ID} | **{Task Name}** | {X}h | {Deps or None} | ⬜ Not Started | | [{Details}](path/to/task.md) |
| {ID} | **{Task Name}** | {X}h | {Deps} | 🔄 In Progress | | [{Details}](path/to/task.md) |
| {ID} | **{Task Name}** | {X}h | {Deps} | ✅ Complete | | [{Details}](path/to/task.md) |
| {ID} | **{Task Name}** | {X}h | {Deps} | ❌ Blocked | | {Blocker description} |

**Phase {N} Total**: {X} hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week {N} ({Date Range})
- [ ] {Task ID}: {Task Name}
- [ ] {Task ID}: {Task Name}
- [ ] {Task ID}: {Task Name}

### Completed Tasks
- [x] {Task ID}: {Task Name} - Completed {Date}
- [x] {Task ID}: {Task Name} - Completed {Date}

## Success Criteria

### Functional Requirements
- ✅ {Requirement 1}
- ✅ {Requirement 2}
- ✅ {Requirement 3}

### Performance Requirements
- ✅ < {X}% latency overhead
- ✅ < {X}MB memory for {Y} operations
- ✅ Support {X} operations/second

### Quality Requirements
- ✅ {X}% test coverage
- ✅ No clippy warnings
- ✅ Full documentation
- ✅ Integration tests passing

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| {Risk description} | {HIGH/MEDIUM/LOW} | {Mitigation strategy} | {Active/Planned/Resolved} |

## Session Planning Guidelines

### Next Session Prompt
Each plan should have a corresponding `next-session-prompt.md` file in the same directory as this tracker, based on the template in `plans/template/next-session-prompt.md`. This file should be updated at the end of each session to set up the next session with proper context.

### Optimal Session Structure
1. **Review** (5 min): Check this tracker and relevant task files
2. **Implementation** (2-3 hours): Complete the task deliverables
3. **Testing** (30 min): Run tests, fix issues
4. **Documentation** (15 min): Update tracker, create PR if needed
5. **Handoff** (10 min): Update next-session-prompt.md in this plan directory

### Using the rust-code-reviewer
For complex Rust implementation tasks, consider using the `rust-code-reviewer` subagent to:
- Review memory safety and ownership patterns
- Validate async/await correctness with tokio
- Check for performance optimizations
- Ensure proper error handling with Result types
- Verify test coverage for critical paths

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

## Critical Implementation Guidelines

### Proxy Mode Parity
**ALWAYS implement changes in BOTH proxy modes:**
- **Forward Proxy** (`src/proxy/forward.rs`): Client → Shadowcat → Server
- **Reverse Proxy** (`src/proxy/reverse.rs`): Client → Shadowcat (HTTP) → Server

When implementing any MCP compliance feature:
1. ✅ Implement in forward proxy
2. ✅ Implement in reverse proxy  
3. ✅ Add tests for both modes
4. ✅ Verify behavior consistency

**Common oversights:**
- Version tracking (must track in both modes)
- Error handling (must be consistent)
- Session state management (must be synchronized)
- Protocol validation (must enforce equally)

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
- [{Reference Document 1}](path/to/doc1.md)
- [{Reference Document 2}](path/to/doc2.md)

### Task Files
- [{Task Category}](tasks/)
- Task files should follow the structure defined in `plans/template/task.md`

### Specifications
- [{Spec Document}](path/to/spec.md)

## Next Actions

1. **{Immediate next step}**
2. **{Following step}**
3. **{Additional steps as needed}**

## Notes

- {Important notes about the project}
- {Any special considerations}
- {Dependencies or constraints}

---

**Document Version**: {X.Y}  
**Created**: {DATE}  
**Last Modified**: {DATE}  
**Author**: {Author/Team}

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| {DATE} | {X.Y} | {Description of changes} | {Author} |