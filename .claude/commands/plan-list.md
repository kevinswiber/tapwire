---
description: List all available development plans
allowed-tools:
  - ls
  - find
  - basename
  - wc
---

# List Available Development Plans

## Active Plans
Plans currently in development (excluding archive and templates):

!`ls -d plans/*/`

!`find plans -maxdepth 1 -type d -name "*" -not -name "plans" -not -name "archive" -not -name "template"`

## Recently Archived Plans
Completed work in the archive:

!`ls -la plans/archive/`

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