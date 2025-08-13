---
description: Show detailed status of all active development plans
allowed-tools:
  - ls
  - find
  - wc
---

# Development Plans Status Report

## Overall Progress Summary

Active plans in the plans directory:
!`find plans -maxdepth 1 -type d -not -name "plans" -not -name "archive" -not -name "template" | wc -l`

## List Active Plans

!`find plans -maxdepth 1 -type d -not -name "plans" -not -name "archive" -not -name "template"`

## Plan Details

!`ls -la plans/*/`

### What to Review

For each plan above, check:
- **Tracker Files**: Current phase, task progress (✅ 🔄 ⬜ ❌), estimates
- **Next Session**: Objectives and mission in next-session-prompt.md
- **Resources**: Task files, analysis docs, supporting documentation

## Recent Activity

To check for recently modified files, run:
```bash
find plans -name "*.md" -type f -mtime -7 | grep -v archive | grep -v template
```

## Recommendations

Based on the status analysis above:

1. **Ready to Work**: Plans with next-session-prompt.md configured
2. **Needs Setup**: Plans missing next-session-prompt or tracker
3. **Active Development**: Plans with "In Progress" tasks
4. **Blocked Items**: Plans with blocked tasks needing attention

## Quick Actions

- `/plan <plan-name>` - Start working on a specific plan
- `/plan-complete <plan-name> <phase>` - Mark a phase complete
- `/plan-list` - See simplified list of plans

To update a plan's status, edit its tracker.md file directly.