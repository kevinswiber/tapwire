---
description: Show detailed status of all active development plans
---

# Development Plans Status Report

## Overall Progress Summary

Active plans overview:
!`echo "Total active plans: $(find plans -maxdepth 1 -type d ! -name archive ! -name template ! -name plans | wc -l | tr -d ' ')"`

## Detailed Plan Status

Analyzing each active plan for progress indicators:

!`for dir in plans/*/; do 
    if [ -d "$dir" ] && [ "$dir" != "plans/archive/" ] && [ "$dir" != "plans/template/" ]; then
        plan=$(basename "$dir")
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "📁 PLAN: $plan"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        
        # Find and analyze tracker
        tracker=$(find "$dir" -name "*tracker.md" -type f 2>/dev/null | head -1)
        if [ -n "$tracker" ]; then
            echo "📊 Tracker: $(basename "$tracker")"
            
            # Extract key status lines
            grep -E "^\\*\\*Status\\*\\*:|^Status:" "$tracker" 2>/dev/null | head -1 || echo "Status: Not specified"
            grep -E "^\\*\\*Phase\\*\\*:|^Phase:" "$tracker" 2>/dev/null | head -1 || true
            grep -E "^\\*\\*Duration\\*\\*:|^\\*\\*Estimated Duration\\*\\*:" "$tracker" 2>/dev/null | head -1 || true
            
            # Count task statuses
            echo ""
            echo "Task Progress:"
            echo "  ✅ Complete: $(grep -c "✅" "$tracker" 2>/dev/null || echo 0)"
            echo "  🔄 In Progress: $(grep -c "🔄" "$tracker" 2>/dev/null || echo 0)"
            echo "  ⬜ Not Started: $(grep -c "⬜" "$tracker" 2>/dev/null || echo 0)"
            echo "  ❌ Blocked: $(grep -c "❌" "$tracker" 2>/dev/null || echo 0)"
        else
            echo "⚠️  No tracker found"
        fi
        
        # Check for next-session-prompt
        echo ""
        if [ -f "$dir/next-session-prompt.md" ]; then
            echo "📋 Next Session: Ready"
            # Extract mission or objective
            grep -A 2 "## Your Mission\\|## Objective\\|## Current Status" "$dir/next-session-prompt.md" 2>/dev/null | head -3 | sed 's/^/   /' || true
        else
            echo "📋 Next Session: Not configured"
        fi
        
        # Count resources
        echo ""
        task_count=$(find "$dir/tasks" -name "*.md" -type f 2>/dev/null | wc -l | tr -d ' ')
        analysis_count=$(find "$dir/analysis" -name "*.md" -type f 2>/dev/null | wc -l | tr -d ' ')
        echo "📚 Resources:"
        echo "   Task files: $task_count"
        [ "$analysis_count" -gt 0 ] && echo "   Analysis docs: $analysis_count"
        
        echo ""
    fi
done`

## Recent Activity

Check for recently modified files (last 7 days):

!`find plans -name "*.md" -type f ! -path "*/archive/*" ! -path "*/template/*" -mtime -7 2>/dev/null | while read file; do
    echo "$(date -r "$file" "+%Y-%m-%d") - $(echo "$file" | sed 's|plans/||')"
done | sort -r | head -10 || echo "No recent activity in the last 7 days"`

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