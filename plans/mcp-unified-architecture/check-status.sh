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
echo "   Code: ${PLAN_DIR/plans*/shadowcat-mcp-compliance/crates}/mcp"
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
# Find current sprint (first one with tasks that aren't complete)
# Look for lines that DON'T have ✅ in the Notes column
CURRENT_SPRINT=""
while IFS= read -r line; do
    if [[ "$line" == *"### Sprint"* ]]; then
        SPRINT_NAME="$line"
    elif [[ "$line" == *"|"* ]] && [[ "$line" =~ \|[[:space:]]*[0-9] ]]; then
        # Check if this task line doesn't have ✅ in it
        if [[ "$line" != *"✅"* ]]; then
            CURRENT_SPRINT="$SPRINT_NAME"
            break
        fi
    fi
done < "$PLAN_DIR/mcp-tracker-v2-critical-path.md"

if [ -z "$CURRENT_SPRINT" ]; then
    if [ "$COMPLETED" = "$TOTAL_TASKS" ] && [ "$TOTAL_TASKS" -gt "0" ]; then
        echo -e "   ${GREEN}✅ ALL SPRINTS COMPLETE!${NC}"
    else
        echo "   Sprint 1: Core Foundation"
    fi
else
    # Clean up the sprint name
    SPRINT_NAME=$(echo "$CURRENT_SPRINT" | sed 's/### //')
    echo "   $SPRINT_NAME"
fi
echo ""

echo -e "${BLUE}📝 Current/Next Task:${NC}"
# Find first task that doesn't have ✅ in the Notes column
NEXT_TASK=""
while IFS= read -r line; do
    # Check if it's a task line (has | and starts with a number in ID column)
    if [[ "$line" == *"|"* ]] && [[ "$line" =~ \|[[:space:]]*[0-9] ]]; then
        # Check if this task line doesn't have ✅ in it
        if [[ "$line" != *"✅"* ]]; then
            NEXT_TASK="$line"
            break
        fi
    fi
done < "$PLAN_DIR/mcp-tracker-v2-critical-path.md"

if [ -z "$NEXT_TASK" ]; then
    echo -e "   ${GREEN}✅ All tasks complete!${NC}"
else
    # Extract task ID and name from the line
    TASK_ID=$(echo "$NEXT_TASK" | cut -d'|' -f2 | sed 's/^ *//;s/ *$//')
    TASK_NAME=$(echo "$NEXT_TASK" | cut -d'|' -f3 | sed 's/^ *//;s/ *$//')
    # Check if it has 🔄 to indicate in-progress
    if [[ "$NEXT_TASK" == *"🔄"* ]]; then
        echo -e "   ${YELLOW}In Progress: $TASK_ID - $TASK_NAME${NC}"
    else
        echo "   Next: $TASK_ID - $TASK_NAME"
    fi
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
echo "   2. cd ${PLAN_DIR/plans*/shadowcat-mcp-compliance/crates}/mcp"
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
