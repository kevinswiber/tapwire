# Tracker Migration Decision

## Current State
- **v1 Tracker**: 7 phases, 250-280 hours, some tasks up to 12h
- **v2 Proposal**: 5 sprints, ~200 hours, all tasks ≤8h, critical path focused

## Key Differences

### Task Organization
| Aspect | v1 (Current) | v2 (Proposed) |
|--------|--------------|---------------|
| Structure | 7 week-based phases | 5 value-based sprints |
| Task Size | 4-12 hours | 3-8 hours max |
| Total Time | 250-280 hours | ~200 hours |
| Approach | Feature-grouped | Critical path |
| Testing | Separate phase (Week 6-7) | Continuous throughout |

### Major Reorganizations

**Moved Earlier**:
- Observability: Was E.3 (Week 5) → Now Sprint 1.1 (immediate)
- API Design: Was E.0 (Week 5) → Now Sprint 4.0 (still builder, but not blocking)
- Error Handling: Was D.3 (Week 4) → Now Sprint 3.1 (with interceptors)

**Split Tasks**:
- B.1 (12h) → Split into Server (6h) + Client (6h)
- B.2 (10h) → Split into SSE Server (8h) + integrated naturally
- F.0 (12h) → Split across multiple test tasks (6h each)

**Combined/Removed**:
- Client & Server interceptor chains → Single implementation task
- Duplicate monitoring/metrics tasks → Single observability setup
- Rules engine → Deferred to Sprint 4 (optional)

**New Additions**:
- Memory Session Store (4h) - Quick MVP before persistence
- Performance Benchmarks (2h) - Lightweight, early validation

## Migration Options

### Option A: Full Switch to v2
**Pros**:
- Cleaner, more focused approach
- Better for Claude sessions
- Delivers value faster
- Lower total hours

**Cons**:
- Need to recreate some task files
- Loses some of Gemini's specific tasks

### Option B: Hybrid Approach
Keep v1 structure but apply v2 improvements:
1. Split oversized tasks (B.1, B.2, C.3, D.4, F.0)
2. Move observability to Phase B
3. Move API design to Phase A
4. Keep all 7 phases

**Pros**:
- Keeps Gemini feedback intact
- Less disruptive
- Gradual improvement

**Cons**:
- Still 250+ hours
- Less focused

### Option C: Keep Both
- Use v2 as execution guide
- Keep v1 as comprehensive reference
- Pick tasks from v2 for actual sessions

**Pros**:
- Maximum flexibility
- No information lost
- Can switch strategies

**Cons**:
- Two trackers to maintain
- Potentially confusing

## Recommendation

**Go with Option C: Keep Both Trackers**

1. **v1 Tracker**: Comprehensive reference with all Gemini feedback
2. **v2 Tracker**: Execution guide for actual implementation

### Why This Works:
- v1 shows everything we might want to do
- v2 shows the critical path to get there
- Can reference v1 for details when implementing v2 tasks
- No need to recreate task files
- Can pivot between approaches

### Implementation Plan:
1. Start executing Sprint 1 from v2
2. Reference corresponding task files from v1 for details
3. If Sprint 1 goes well, continue with v2 approach
4. If we need more structure, switch back to v1

### Session 1 Execution:
From v2 Sprint 1:
- Task 1.0: Fix Async Antipatterns (8h)
- References v1 task B.0 for detailed requirements

This gives us the best of both worlds - comprehensive planning with pragmatic execution.

## Decision Checklist

- [ ] Keep v1 tracker as `mcp-unified-architecture-tracker.md` (reference)
- [ ] Keep v2 tracker as `mcp-tracker-v2-critical-path.md` (execution)
- [ ] Start with Sprint 1.0 from v2
- [ ] Reference v1 task files for implementation details
- [ ] Update both trackers as we progress
- [ ] Re-evaluate after Sprint 1 completion

## Next Immediate Actions

1. Review v2 Sprint 1.0 (Fix Async Antipatterns)
2. Find corresponding v1 Task B.0 file
3. Create `next-session-prompt.md` for Sprint 1.0
4. Begin implementation

This approach minimizes risk while maximizing flexibility.