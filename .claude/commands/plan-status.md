---
description: Show detailed status of all active development plans
allowed-tools:
  - ls
  - grep
  - basename
  - wc
  - xargs
---

# Development Plans Status Report

## Overall Progress Summary

Active plans in the plans directory:
!`ls -d plans/*/ | grep -v archive | grep -v template | wc -l`

## List Active Plans

!`ls -d plans/*/ | grep -v archive | grep -v template | xargs -n1 basename`

## Plan Details

For each plan listed above, review:

### Tracker Files
Look for *tracker.md files to understand:
- Current phase and status
- Task progress (✅ 🔄 ⬜ ❌)
- Estimated duration
- Key findings

### Next Session Setup
Check for next-session-prompt.md to see:
- Current objectives
- Mission for next session
- Dependencies

### Resources
Count available resources:
- Task files in tasks/
- Analysis documents in analysis/
- Supporting documentation

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