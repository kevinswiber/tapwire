# MCP Unified Architecture - Session Guide

## 🎯 FOR EVERY NEW CLAUDE SESSION

### Step 1: Orient Yourself (2 min)
```bash
# Where are we in the plan?
cat plans/mcp-unified-architecture/next-session-prompt.md | head -30

# Quick status check
echo "=== CURRENT SPRINT ==="
grep "Sprint" plans/mcp-unified-architecture/mcp-tracker-v2-critical-path.md | grep "🔄\|✅" | head -1 || echo "Sprint 1 - Starting fresh"

echo "=== COMPLETED TASKS ==="
grep "✅" plans/mcp-unified-architecture/mcp-tracker-v2-critical-path.md | wc -l

echo "=== IN PROGRESS ==="
grep "🔄" plans/mcp-unified-architecture/mcp-tracker-v2-critical-path.md
```

### Step 2: Understand the System (3 min)

**We have TWO trackers** (this is intentional!):
- **v2 Critical Path** = What to do (execution)
- **v1 Comprehensive** = How to do it (reference)

**Always follow v2 for WHAT to work on**  
**Always check v1 for HOW to implement**

### Step 3: Begin Work

1. **Read the full session prompt**: `next-session-prompt.md`
2. **Find your task in v2**: Look for next ⬜ or current 🔄
3. **Get details from v1**: Check corresponding task file in `tasks/`
4. **Start coding**: Follow the implementation guide

## 📋 Task Status Symbols

- ⬜ Not Started - Ready to begin
- 🔄 In Progress - Currently working on
- ✅ Complete - Done and tested
- ❌ Blocked - Has issues
- ⏸️ Paused - Temporarily stopped

## 🔄 During Your Session

### Every 2 hours:
```bash
# Quick progress check
echo "Have I completed a meaningful chunk?"
echo "Should I commit my progress?"
echo "Am I still on the critical path?"
```

### If you get confused:
1. Re-read `next-session-prompt.md`
2. Check `SESSION-GUIDE.md` (this file)
3. Look at Sprint definition in v2 tracker
4. Find task details in v1 tracker

### If task seems too big:
- It's OK to do partial completion
- Mark as 🔄 In Progress
- Document what's left in next-session-prompt

## 🏁 End of Session Checklist

### 1. Update v2 Tracker Status
```bash
# Edit the tracker
vim plans/mcp-unified-architecture/mcp-tracker-v2-critical-path.md
# Change ⬜ to 🔄 or ✅ as appropriate
```

### 2. Update v1 Tracker (if applicable)
```bash
# Mark corresponding phase items
vim plans/mcp-unified-architecture/mcp-unified-architecture-tracker.md
```

### 3. Update README Current Status
```bash
vim plans/mcp-unified-architecture/README.md
# Update the "Current Status" section
```

### 4. Create Next Session Prompt
```bash
vim plans/mcp-unified-architecture/next-session-prompt.md
# Clear, specific instructions for next session
# Include:
# - What task is next
# - Any context from this session
# - Specific files to start with
```

### 5. Commit Your Work
```bash
git add -A
git commit -m "feat(mcp): complete Sprint X Task Y - brief description"
git push
```

## 🚨 Common Pitfalls to Avoid

1. **DON'T** try to do everything in v1 tracker - it's reference only
2. **DON'T** skip updating next-session-prompt - future you will be lost
3. **DON'T** work on non-critical path items - stay focused
4. **DON'T** combine multiple tasks without updating trackers
5. **DON'T** forget to test incrementally - don't save it all for the end

## 💡 Pro Tips

1. **Small Commits**: Commit working code frequently
2. **Test First**: Run existing tests before changing code
3. **Reference Shadowcat**: Look at `/src/` for patterns
4. **Ask Why**: If v2 says do X, v1 explains why
5. **Stay on Path**: Sprint goals > individual task perfection

## 📊 Quick Progress Metrics

```bash
# How much is done?
echo "Tasks Complete: $(grep -c "✅" plans/mcp-unified-architecture/mcp-tracker-v2-critical-path.md)/38"

# What sprint are we in?
grep "### Sprint" plans/mcp-unified-architecture/mcp-tracker-v2-critical-path.md | grep -n "Sprint" | grep "✅\|🔄"

# Hours completed (rough estimate)
echo "Hours: $(grep "✅" plans/mcp-unified-architecture/mcp-tracker-v2-critical-path.md | grep -oE "[0-9]+h" | grep -oE "[0-9]+" | paste -sd+ | bc) done"
```

## 🎯 Remember the Goal

**Sprint 1**: Get a working proxy with metrics  
**Sprint 2**: Add persistence and SSE  
**Sprint 3**: Production essentials  
**Sprint 4**: Advanced features  
**Sprint 5**: Battle testing  

Each sprint delivers VALUE. Stay focused on the sprint goal!

---

**Questions?** Check these in order:
1. `next-session-prompt.md` - What to do now
2. `SESSION-GUIDE.md` - How to navigate (this file)
3. `mcp-tracker-v2-critical-path.md` - The execution plan
4. `TRACKER-MIGRATION-DECISION.md` - Why two trackers
5. Task files in `tasks/` - Detailed requirements