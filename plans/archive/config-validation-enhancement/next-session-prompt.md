# Next Session: {PHASE/TASK NAME}

## Project Context

{Brief description of the overall project and its goals}

**Project**: {Project name from tracker}
**Tracker**: `plans/{project-directory}/{project}-tracker.md`
**Status**: Phase {X} - {Current Phase Name} ({X}% Complete)

## Current Status

### What Has Been Completed
- **{Previous Task ID}: {Task Name}** (✅ Completed {Date})
  - {Key achievement 1}
  - {Key achievement 2}
  - {Metrics if applicable: X tests passing, zero warnings}

### What's In Progress
- **{Current Task ID}: {Task Name}** ({Status: Not Started | In Progress | Blocked})
  - Duration: {X} hours
  - Dependencies: {List dependencies or "None"}

## Your Mission

{Clear, action-oriented description of what needs to be accomplished in this session}

### Priority 1: {Main Task} ({X} hours)

1. **{Subtask 1}** ({X}h)
   - {Specific deliverable}
   - {Success criteria}
   
2. **{Subtask 2}** ({X}h)
   - {Specific deliverable}
   - {Success criteria}

### Priority 2: {Secondary Task if time permits}
{Only include if realistic for session}

## Essential Context Files to Read

1. **Primary Tracker**: `plans/{path}/tracker.md` - Full project context
2. **Task Details**: `plans/{path}/tasks/{task-file}.md` - Current task specifications
3. **Implementation**: `{src/module/file.rs}` - Existing code to understand
4. **Dependencies**: `{src/related/module.rs}` - Related components

## Working Directory

```bash
cd {/path/to/working/directory}
```

## Commands to Run First

```bash
# Verify current state
{command to check status}

# Run existing tests
{command to run tests}

# Check for issues
cargo clippy --all-targets -- -D warnings  # For Rust projects
```

## Implementation Strategy

### Phase 1: {Setup/Analysis} ({X} min)
1. {Step 1}
2. {Step 2}
3. {Step 3}

### Phase 2: {Core Implementation} ({X} hours)
1. {Step 1}
2. {Step 2}
3. {Step 3}

### Phase 3: {Testing/Validation} ({X} min)
1. {Step 1}
2. {Step 2}
3. {Step 3}

### Phase 4: {Cleanup/Documentation} ({X} min)
1. Run formatters and linters
2. Update tracker with completion status
3. Create next-session-prompt.md if needed

## Success Criteria Checklist

- [ ] {Primary deliverable completed}
- [ ] {Tests written and passing}
- [ ] {No linter warnings}
- [ ] {Documentation updated}
- [ ] {Tracker updated with status}
- [ ] {Performance targets met (if applicable)}

## Key Commands

```bash
# Development commands
{frequently used command 1}
{frequently used command 2}

# Testing commands
{test command 1}
{test command 2}

# Validation commands
{validation command 1}
{validation command 2}
```

## Important Notes

- **Always use TodoWrite tool** to track progress through tasks
- **Start with examining existing code** to understand architecture
- **Follow established patterns** from previous implementations
- **Test incrementally** as you build
- **Run linters before considering complete**
- **Update tracker** when task is complete

## Key Design Considerations

1. **{Consideration 1}**: {Explanation}
2. **{Consideration 2}**: {Explanation}
3. **{Consideration 3}**: {Explanation}

## Performance/Quality Targets (if applicable)

- **{Metric 1}**: {Target value}
- **{Metric 2}**: {Target value}
- **{Metric 3}**: {Target value}

## Risk Factors & Blockers

- **{Risk 1}**: {Mitigation strategy}
- **{Blocker if any}**: {Resolution approach}

## Next Steps After This Task

Once {current task} is complete:
- **{Next Task ID}**: {Next Task Name} ({X} hours, {dependencies})
- **{Following Task}**: {Task Name} ({X} hours)

After completing {current phase}:
- Move to {next phase} ({description})

## Model Usage Guidelines

- **IMPORTANT**: Be mindful of model capabilities. When context window has less than 15% availability, suggest creating a new session and save prompt to next-session-prompt.md

## Session Time Management

**Estimated Session Duration**: {X-Y} hours
- Setup & Context: {X} min
- Implementation: {X} hours  
- Testing: {X} min
- Documentation: {X} min

## Related Context (Optional)

- **Integration Points**: {How this connects to other components}
- **Downstream Dependencies**: {What depends on this work}
- **Parallel Work**: {Any work happening in parallel}

---

**Session Goal**: {One-sentence summary of what success looks like}

**Last Updated**: {Date}
**Next Review**: {When to check progress}