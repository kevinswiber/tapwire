# Risk Assessment

## Risk Matrix

| Risk | Probability | Impact | Risk Level | Status |
|------|------------|--------|------------|---------|
| Circular dependencies | Low | High | MEDIUM | Mitigated |
| Breaking public API | Low | High | MEDIUM | Planned |
| Test failures | High | Medium | HIGH | Accepted |
| Merge conflicts | Medium | Medium | MEDIUM | Planned |
| Performance impact | Low | Low | LOW | Monitored |
| Hidden dependencies | Low | Medium | LOW | Mitigated |

## Detailed Risk Analysis

### 1. Circular Dependencies
**Risk**: auth ↔ interceptor circular dependency prevents independent migration

**Probability**: Low
- Already identified in analysis
- Both modules already have Error types
- Only need to fix usage patterns

**Impact**: High
- Could block migration progress
- Might require significant refactoring

**Mitigation**:
- Both modules already have Error types
- Focus on fixing usage, not restructuring
- If needed, use trait objects or Box<dyn Error>
- Can migrate together in same commit

**Contingency**: 
- Introduce intermediate error type
- Use dynamic dispatch temporarily

### 2. Breaking Public API
**Risk**: Changes break existing users of the library

**Probability**: Low
- Main API through api.rs unchanged
- crate::Error remains for public API
- Internal changes only

**Impact**: High
- Would require major version bump
- Could affect downstream users

**Mitigation**:
- Keep crate::Error and crate::Result in lib.rs
- Only change internal module implementations
- Public API in api.rs continues using crate::Result
- Add #[from] conversions for compatibility

**Contingency**:
- Can add deprecated aliases temporarily
- Provide migration guide if needed

### 3. Test Failures
**Risk**: Tests fail after error type changes

**Probability**: High
- Many tests likely check for specific error types
- Error matching will break

**Impact**: Medium
- Need to update test assertions
- Time consuming but straightforward

**Mitigation**:
- Fix tests as part of each module migration
- Run tests after each change
- Update error matching patterns
- Use cargo test --no-fail-fast to see all failures

**Contingency**:
- Temporarily disable failing tests
- Fix in batches after migration

### 4. Merge Conflicts
**Risk**: Other development creates conflicts

**Probability**: Medium
- Active codebase
- Auth and proxy modules frequently changed

**Impact**: Medium
- Delays migration
- Requires careful resolution

**Mitigation**:
- Complete quickly (2-3 days)
- Migrate one module at a time
- Coordinate with team on timing
- Work on less active modules first

**Contingency**:
- Can pause and rebase
- Cherry-pick completed modules

### 5. Performance Impact
**Risk**: Additional error types increase binary size or runtime overhead

**Probability**: Low
- Error handling not on hot path
- Compiler optimizations effective
- Most conversions compile-time

**Impact**: Low
- Minor binary size increase expected
- No runtime performance impact

**Mitigation**:
- Monitor binary size after each phase
- Use #[inline] for simple conversions
- Rely on compiler optimizations

**Contingency**:
- Can consolidate error variants if needed
- Use Box<dyn Error> for rare errors

### 6. Hidden Dependencies
**Risk**: Undiscovered dependencies complicate migration

**Probability**: Low
- Comprehensive analysis completed
- Dependency graph mapped
- All violations identified

**Impact**: Medium
- Could require order changes
- Might delay timeline

**Mitigation**:
- Start with foundation modules (no deps)
- Run full test suite frequently
- Check compilation after each module

**Contingency**:
- Adjust migration order as needed
- Can leave problematic modules for later

## Phase-Specific Risks

### Phase 1: Foundation Modules
**Risks**: Minimal - no dependencies
**Confidence**: HIGH

### Phase 2: Service Modules  
**Risks**: audit depends on auth (not yet fixed)
**Mitigation**: auth already has Error type, just needs usage fix
**Confidence**: MEDIUM

### Phase 3: Fix Existing
**Risks**: Complex modules, OAuth handling
**Mitigation**: Careful testing, preserve logic
**Confidence**: MEDIUM

### Phase 4: Cleanup
**Risks**: Minimal - minor fixes only
**Confidence**: HIGH

## Rollback Plan

### Module-Level Rollback
Each module change is atomic:
```bash
git revert <commit-hash>  # Revert single module
cargo test --all          # Verify still working
```

### Phase-Level Rollback
Each phase in separate branch:
```bash
git checkout main
git merge phase-1  # Only merge successful phases
```

### Full Rollback
If critical issues:
```bash
git checkout main
# Continue without migration
# Document lessons learned
```

## Monitoring Plan

### During Migration
- Run tests after each module
- Check binary size: `cargo build --release && ls -lh target/release/shadowcat`
- Count violations: `grep -r "crate::Error" src/ | wc -l`
- Monitor CI pipeline

### After Migration
- Performance benchmarks
- Binary size comparison
- Memory usage tests
- Full integration tests

## Success Indicators

### Green Flags ✅
- Tests pass after each module
- Violation count decreasing
- Clean compilation
- No clippy warnings

### Yellow Flags ⚠️
- Test failures (expected, need updates)
- Binary size increase < 5%
- Some modules harder than expected

### Red Flags 🚪
- Circular dependencies can't be resolved
- Public API breakage
- Binary size increase > 10%
- Performance regression

## Decision Points

### Continue Conditions
- Violations decreasing
- Tests fixable
- No API breakage
- Timeline on track

### Pause Conditions
- Major architectural issue found
- Team needs modules unchanged
- Critical bug discovered

### Abort Conditions
- Public API must break
- Performance regression > 5%
- Circular dependencies unresolvable

## Communication Plan

### Before Starting
- Notify team of migration plan
- Schedule for low-activity period
- Get approval for auth/proxy changes

### During Migration
- Daily updates on progress
- Flag any blockers immediately
- Coordinate on active modules

### After Completion
- Document changes
- Update architecture docs
- Share lessons learned

## Conclusion

**Overall Risk Level**: MEDIUM

**Confidence Level**: HIGH

The migration is well-understood with clear patterns and mitigations. Main risks are test failures (expected) and merge conflicts (manageable). No architectural blockers identified.

**Recommendation**: Proceed with migration following the strategy. Complete quickly to minimize merge conflicts.