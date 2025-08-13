---
description: List all available development plans
---

# List Available Development Plans

## Active Plans
Plans currently in development (excluding archive and templates):

!`ls -d plans/*/ | grep -v archive | grep -v template | xargs -n1 basename | sed 's/^/- /' | sort`

## Check Plan Details
For each active plan, show key information:

!`ls -d plans/*/ | grep -v archive | grep -v template`

Check each plan's contents manually for:
- next-session-prompt.md file
- *tracker.md file
- tasks/*.md files

## Recently Archived Plans
Completed work in the archive:

!`ls plans/archive/*.md | xargs -n1 basename | sed 's/.md$//' | sed 's/^/- /' | tail -5`

## Quick Actions

To start working on a plan, use:
- `/plan <plan-name>` - Load a specific plan for work
- `/plan-status` - Show detailed status of all plans
- `/plan-complete <plan-name> <phase>` - Mark a phase as complete

## Create New Plan

To create a new plan from templates:
1. Copy the template files: `cp -r plans/template plans/<new-plan-name>`
2. Customize the tracker and next-session-prompt
3. Create initial task files in `plans/<new-plan-name>/tasks/`

Refer to @plans/template/README.md for detailed instructions.