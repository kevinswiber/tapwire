---
description: Load a development plan for focused work session
argument-hint: <plan-name>
---

# Load Development Plan: $ARGUMENTS

## Context Gathering

Current plan directory structure:
!`ls -la plans/$ARGUMENTS/`

## Load Plan Files

### 1. Next Session Prompt
Load the current objectives and session setup:
@plans/$ARGUMENTS/next-session-prompt.md

### 2. Project Tracker
Review the main tracker for overall progress:
@plans/$ARGUMENTS/*tracker.md

### 3. Task Files
Available task files in this plan:
!`find plans/$ARGUMENTS/tasks -name "*.md" -type f | head -10`

## Session Setup

Based on the loaded files, please:

1. **Summarize Current Status**
   - What phase are we in?
   - What's been completed recently?
   - What are the immediate priorities?

2. **Initialize Todo List**
   - Use TodoWrite to create todos for this session's tasks
   - Focus on tasks identified in the next-session-prompt

3. **Identify Key Files**
   - List the specific source files we'll be working with
   - Note any dependencies or related components

4. **Set Success Criteria**
   - What must be accomplished this session?
   - What tests should pass?
   - What documentation needs updating?

## Working Directory
!`pwd`

## Important Reminders
- Update the tracker as tasks are completed
- Run tests frequently during implementation
- Update next-session-prompt.md at session end
- Follow any plan-specific guidelines in CLAUDE.md

Let's begin by understanding the current objectives from the next-session-prompt above.