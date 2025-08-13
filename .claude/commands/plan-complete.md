---
description: Mark a plan phase or task as complete and prepare for next phase
argument-hint: <plan-name> [phase-or-task-id]
---

# Complete Plan Phase: $ARGUMENTS

## Parse Arguments

Arguments provided: $ARGUMENTS

Usage: /plan-complete <plan-name> [phase-or-task-id]

## Current Plan Status

Load the tracker to understand current state:
@plans/$ARGUMENTS/*tracker.md

## Completion Checklist

Before marking this phase/task complete, verify:

### 1. Code Quality
- [ ] All tests passing
- [ ] No clippy warnings (`cargo clippy --all-targets -- -D warnings`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Documentation updated

### 2. Task Deliverables
Review the task/phase requirements and confirm all deliverables are complete.

### 3. Update Tracker

Please update the tracker to:
1. Mark the specified phase/task as ✅ Complete
2. Add completion date
3. Update progress percentages
4. Note any important findings or decisions

### 4. Archive Completed Materials

If completing an entire phase:
Consider moving completed task files to archive if appropriate.

### 5. Prepare Next Phase

Create or update the next-session-prompt.md for the next phase:

**Template for next-session-prompt update:**
```markdown
# Next Session: [Next Phase Name]

## Project Context
[Brief context about what was just completed]

## Current Status

### What Has Been Completed
- **$ARGUMENTS** (✅ Completed $(date +%Y-%m-%d))
  - [Key achievement 1]
  - [Key achievement 2]
  - [Metrics if applicable]

### What's In Progress
- **[Next Task ID]: [Task Name]** (Not Started)
  - Duration: X hours
  - Dependencies: [List or "None"]

## Your Mission
[Clear description of next phase objectives]

[Continue with rest of template...]
```

### 6. Update README if Major Milestone

If this is a significant completion, update @plans/README.md to reflect it in the "Recently Completed" section.

## Generate Completion Report

Based on the tracker, please provide:

1. **Summary of Completed Work**
   - What was accomplished
   - Key decisions made
   - Problems solved

2. **Metrics**
   - Time taken vs estimate
   - Tests added
   - Performance improvements

3. **Lessons Learned**
   - What went well
   - What could improve
   - Technical insights

4. **Next Steps**
   - Immediate next tasks
   - Dependencies or blockers
   - Recommended approach

## Git Commit

After updating all documentation:

```bash
git add plans/$ARGUMENTS/
git commit -m "docs(plans): complete $ARGUMENTS

- Mark phase/task as complete in tracker
- Update next-session-prompt for next phase
- Document achievements and metrics
- Prepare for [next phase name]"
```

## Actions to Take

Please now:
1. Update the tracker with completion status
2. Create/update next-session-prompt.md
3. Generate the completion report
4. Prepare git commit with changes

This ensures smooth transition to the next work session!