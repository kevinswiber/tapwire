---
description: List all available development plans
---

# List Available Development Plans

## Active Plans
Plans currently in development (excluding archive and templates):

!`for dir in plans/*/; do [ -d "$dir" ] && [ "$dir" != "plans/archive/" ] && [ "$dir" != "plans/template/" ] && echo "- $(basename "$dir")" || true; done | sort`

## Check Plan Details
For each active plan, show key information:

!`for dir in plans/*/; do 
    if [ -d "$dir" ] && [ "$dir" != "plans/archive/" ] && [ "$dir" != "plans/template/" ]; then
        plan=$(basename "$dir")
        echo "### $plan"
        if [ -f "$dir/next-session-prompt.md" ]; then
            echo "  ✓ Has next-session-prompt"
        else
            echo "  ✗ Missing next-session-prompt"
        fi
        tracker=$(find "$dir" -name "*tracker.md" -type f 2>/dev/null | head -1)
        if [ -n "$tracker" ]; then
            echo "  ✓ Has tracker: $(basename "$tracker")"
            # Try to extract status from tracker
            grep -m1 "Status:" "$tracker" 2>/dev/null | sed 's/^/  /' || true
        else
            echo "  ✗ Missing tracker"
        fi
        task_count=$(find "$dir/tasks" -name "*.md" -type f 2>/dev/null | wc -l | tr -d ' ')
        echo "  📝 Task files: $task_count"
        echo ""
    fi
done`

## Recently Archived Plans
Completed work in the archive:

!`ls -la plans/archive/*.md 2>/dev/null | tail -5 | awk '{print "- " $NF}' | xargs -I {} basename {} .md || echo "No archived root-level plans"`

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