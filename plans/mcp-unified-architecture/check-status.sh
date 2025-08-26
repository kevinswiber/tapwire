#!/bin/bash
# Quick status check for MCP Unified Architecture plan

echo "════════════════════════════════════════════════════════"
echo "       MCP UNIFIED ARCHITECTURE - STATUS CHECK"
echo "════════════════════════════════════════════════════════"
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get directory of this script
PLAN_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo -e "${BLUE}📍 Current Location:${NC}"
echo "   Plan: $PLAN_DIR"
echo "   Code: ${PLAN_DIR/plans*/crates}/mcp"
echo ""

echo -e "${BLUE}📊 Overall Progress:${NC}"
TOTAL_TASKS=$(grep -E "^\| [0-9]" "$PLAN_DIR/mcp-tracker-v2-critical-path.md" 2>/dev/null | wc -l | tr -d ' ')
COMPLETED=$(grep "✅" "$PLAN_DIR/mcp-tracker-v2-critical-path.md" 2>/dev/null | wc -l | tr -d ' ')  
IN_PROGRESS=$(grep "🔄" "$PLAN_DIR/mcp-tracker-v2-critical-path.md" 2>/dev/null | wc -l | tr -d ' ')
REMAINING=$((TOTAL_TASKS - COMPLETED - IN_PROGRESS))
echo "   Completed: ${COMPLETED} tasks"
echo "   In Progress: ${IN_PROGRESS} tasks"
echo "   Remaining: ${REMAINING} tasks"
echo "   Total: ${TOTAL_TASKS} tasks"
echo ""

echo -e "${BLUE}🎯 Current Sprint:${NC}"
# Find current sprint (first one with incomplete tasks)
CURRENT_SPRINT=$(grep -A 50 "### Sprint" "$PLAN_DIR/mcp-tracker-v2-critical-path.md" 2>/dev/null | grep -B 1 "⬜\|🔄" | grep "Sprint" | head -1)
if [ -z "$CURRENT_SPRINT" ]; then
    if [ "$COMPLETED" = "$TOTAL_TASKS" ] && [ "$TOTAL_TASKS" -gt "0" ]; then
        echo -e "   ${GREEN}✅ ALL SPRINTS COMPLETE!${NC}"
    else
        echo "   Sprint 1: Core Foundation (Starting)"
    fi
else
    echo "   $CURRENT_SPRINT"
fi
echo ""

echo -e "${BLUE}📝 Current/Next Task:${NC}"
# Find first in-progress or not started task from v2 tracker
CURRENT_TASK=$(grep "| [0-9]" "$PLAN_DIR/mcp-tracker-v2-critical-path.md" | grep "🔄" | head -1)
if [ -z "$CURRENT_TASK" ]; then
    # No in-progress, look for next not started
    NEXT_TASK=$(grep "| [0-9]" "$PLAN_DIR/mcp-tracker-v2-critical-path.md" | head -1)
    if [ -z "$NEXT_TASK" ]; then
        echo -e "   ${GREEN}✅ All tasks complete!${NC}"
    else
        # Extract task name from the line
        TASK_NAME=$(echo "$NEXT_TASK" | cut -d'|' -f3 | sed 's/^ *//;s/ *$//')
        echo "   Next: $TASK_NAME"
    fi
else
    TASK_NAME=$(echo "$CURRENT_TASK" | cut -d'|' -f3 | sed 's/^ *//;s/ *$//')
    echo -e "   ${YELLOW}In Progress: $TASK_NAME${NC}"
fi
echo ""

echo -e "${BLUE}📚 Key Documents:${NC}"
echo "   • next-session-prompt.md - Start here!"
echo "   • SESSION-GUIDE.md - If confused"
echo "   • mcp-tracker-v2-critical-path.md - Execution plan"
echo "   • mcp-unified-architecture-tracker.md - Full reference"
echo ""

echo -e "${BLUE}🚀 Quick Start:${NC}"
echo "   1. cat $PLAN_DIR/next-session-prompt.md"
echo "   2. cd ${PLAN_DIR/plans*/crates}/mcp"
echo "   3. Start implementing!"
echo ""

echo "════════════════════════════════════════════════════════"

# Check if we're in a git repo and have uncommitted changes
if command -v git &> /dev/null; then
    if [ -d .git ] || git rev-parse --git-dir > /dev/null 2>&1; then
        CHANGES=$(git status --porcelain 2>/dev/null | wc -l)
        if [ "$CHANGES" -gt 0 ]; then
            echo -e "${YELLOW}⚠️  You have $CHANGES uncommitted changes${NC}"
            echo "════════════════════════════════════════════════════════"
        fi
    fi
fi