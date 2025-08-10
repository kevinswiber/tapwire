# Rollback Plan

## Overview
This document outlines procedures for safely rolling back changes if issues arise during the CLI refactoring process.

## Rollback Triggers

### Critical Issues (Immediate Rollback)
- ❌ Core functionality broken (commands don't work)
- ❌ Data corruption or loss
- ❌ Security vulnerability introduced
- ❌ Performance degradation > 20%
- ❌ Binary size increase > 25%

### Non-Critical Issues (Evaluate)
- ⚠️ Minor functionality differences
- ⚠️ Performance degradation 5-20%
- ⚠️ Test coverage decrease
- ⚠️ Documentation gaps
- ⚠️ Code quality issues

## Git-Based Rollback Strategy

### Branch Protection
```bash
# Create feature branch for refactoring
git checkout -b cli-refactor

# Create backup branch before starting
git branch cli-refactor-backup

# Tag stable points
git tag -a refactor-checkpoint-1 -m "After common module"
git tag -a refactor-checkpoint-2 -m "After replay extraction"
# ... etc
```

### Rollback Procedures

#### Full Rollback (Nuclear Option)
```bash
# Abandon all changes, return to main
git checkout main
git branch -D cli-refactor

# Or reset to backup
git checkout cli-refactor
git reset --hard cli-refactor-backup
```

#### Partial Rollback (Specific Phase)
```bash
# List all commits
git log --oneline

# Revert to specific checkpoint
git reset --hard refactor-checkpoint-2

# Or revert specific commit
git revert <commit-hash>
```

#### Cherry-Pick Recovery
```bash
# Create new branch from main
git checkout -b cli-refactor-recovery main

# Cherry-pick good commits
git cherry-pick <good-commit-1>
git cherry-pick <good-commit-2>
# Skip problematic commits
```

## Code-Based Rollback Strategy

### Feature Flags Approach
```rust
// Cargo.toml
[features]
default = ["old-cli"]
new-cli = []
old-cli = []

// main.rs
#[cfg(feature = "new-cli")]
mod cli;

#[cfg(feature = "old-cli")]
use crate::old_implementations::*;

#[cfg(feature = "new-cli")]
use crate::cli::*;

fn main() {
    #[cfg(feature = "old-cli")]
    {
        old_main_logic();
    }
    
    #[cfg(feature = "new-cli")]
    {
        new_main_logic();
    }
}
```

### Gradual Migration Approach
```rust
// Keep old implementations with deprecation
#[deprecated(since = "2.0.0", note = "Use cli::forward::execute_stdio")]
async fn run_stdio_forward_old(/* ... */) -> Result<()> {
    // Original implementation
}

// New implementation
mod cli {
    pub mod forward {
        pub async fn execute_stdio(/* ... */) -> Result<()> {
            // New implementation
        }
    }
}

// Main dispatch can choose
match std::env::var("USE_NEW_CLI") {
    Ok(_) => cli::forward::execute_stdio(args).await,
    Err(_) => run_stdio_forward_old(args).await,
}
```

## Testing Before Rollback Decision

### Quick Validation Tests
```bash
#!/bin/bash
# quick_validation.sh

echo "Running quick validation..."

# Test basic commands work
if ! cargo run -- --help > /dev/null 2>&1; then
    echo "FAIL: Help command broken"
    exit 1
fi

if ! echo '{"jsonrpc":"2.0","method":"ping","id":1}' | \
     cargo run -- forward stdio -- echo > /dev/null 2>&1; then
    echo "FAIL: Forward stdio broken"
    exit 1
fi

# Check binary size
ORIGINAL_SIZE=5000000  # 5MB baseline
CURRENT_SIZE=$(stat -f%z target/release/shadowcat 2>/dev/null || \
               stat -c%s target/release/shadowcat 2>/dev/null)

if [ "$CURRENT_SIZE" -gt $((ORIGINAL_SIZE * 125 / 100)) ]; then
    echo "FAIL: Binary size increased > 25%"
    exit 1
fi

echo "Quick validation PASSED"
```

### Performance Validation
```bash
#!/bin/bash
# perf_validation.sh

# Run benchmarks and compare
cargo bench --bench cli_bench -- --save-baseline before
# After changes
cargo bench --bench cli_bench -- --baseline before

# Check for regression
if grep -q "slower" target/criterion/report.txt; then
    REGRESSION=$(grep "slower" target/criterion/report.txt | \
                 sed 's/.*(\(.*\)% slower).*/\1/')
    if [ "$REGRESSION" -gt 20 ]; then
        echo "FAIL: Performance regression > 20%"
        exit 1
    fi
fi
```

## Rollback Execution Steps

### Step 1: Stop and Assess
1. **Stop all work** on the refactoring
2. **Document the issue** that triggered rollback
3. **Save any useful changes** for future reference
4. **Notify team** of rollback decision

### Step 2: Execute Rollback
```bash
# 1. Commit any uncommitted work (to preserve it)
git add -A
git commit -m "WIP: Saving work before rollback"

# 2. Create a branch to preserve the attempt
git branch failed-attempt-$(date +%Y%m%d)

# 3. Execute chosen rollback strategy
git checkout main  # or other rollback approach

# 4. Verify rollback successful
cargo test --all
./quick_validation.sh
```

### Step 3: Post-Rollback Actions
1. **Run full test suite** to verify stability
2. **Document lessons learned** in migration notes
3. **Update rollback plan** with new insights
4. **Plan revised approach** if re-attempting

## Incremental Rollback Options

### Module-Level Rollback
If a specific module extraction fails:
```bash
# Revert just that module
git revert <module-commit>

# Or manually restore
git checkout main -- src/main.rs
git checkout HEAD~1 -- src/cli/problematic_module.rs
```

### Function-Level Rollback
Keep both implementations temporarily:
```rust
// main.rs
const USE_NEW_FORWARD: bool = false;  // Toggle for testing

if USE_NEW_FORWARD {
    cli::forward::execute_stdio(args).await
} else {
    run_stdio_forward_old(args).await
}
```

## Recovery Procedures

### From Partial Success
If some modules work but others don't:
1. Keep successful extractions
2. Revert problematic ones
3. Document what worked/failed
4. Adjust approach for failures

### From Complete Failure
If entire refactor fails:
1. Full rollback to main
2. Analyze root causes
3. Create new plan addressing issues
4. Consider smaller increments

## Rollback Documentation

### Required Documentation
```markdown
# Rollback Report - [Date]

## Trigger
- What issue caused the rollback
- When it was discovered
- Impact assessment

## Actions Taken
- Rollback strategy used
- Commands executed
- Time to restore service

## Root Cause
- Why the issue occurred
- What could prevent it

## Lessons Learned
- What worked well
- What failed
- Improvements for next attempt

## Next Steps
- Revised approach
- Additional testing needed
- Timeline adjustments
```

## Preventive Measures

### Before Each Phase
- ✅ Create git tag for checkpoint
- ✅ Run full test suite
- ✅ Document current metrics
- ✅ Create rollback branch

### During Development
- ✅ Commit frequently (small commits)
- ✅ Test after each change
- ✅ Monitor performance metrics
- ✅ Keep old code until verified

### After Each Phase
- ✅ Validate all functionality
- ✅ Compare with baseline
- ✅ Get peer review
- ✅ Update documentation

## Emergency Contacts

### If Rollback Needed
1. **Technical Lead**: Review and approve rollback
2. **Team Members**: Notify of changes
3. **Stakeholders**: Update on timeline impact

## Rollback Decision Matrix

| Issue Severity | Time to Fix | Business Impact | Action |
|---------------|-------------|-----------------|---------|
| Critical | > 4 hours | High | Immediate rollback |
| Critical | < 4 hours | High | Attempt fix first |
| Major | > 1 day | Medium | Rollback to checkpoint |
| Major | < 1 day | Medium | Fix forward |
| Minor | Any | Low | Fix forward |

## Success Criteria for Avoiding Rollback

### Technical Criteria
- ✅ All tests passing
- ✅ Performance within 5% of baseline
- ✅ Binary size within 10% of baseline
- ✅ No security vulnerabilities
- ✅ No data corruption

### Process Criteria
- ✅ Code reviewed by peer
- ✅ Documentation updated
- ✅ Migration notes complete
- ✅ Rollback plan tested

## Post-Mortem Template

If rollback occurs, conduct post-mortem:
```markdown
# Post-Mortem: CLI Refactor Rollback

## Date: [Date]
## Duration: [Start] - [End]
## Impact: [Description]

## Timeline
- [Time]: Issue discovered
- [Time]: Rollback decision made
- [Time]: Rollback executed
- [Time]: Service restored

## What Went Wrong
[Detailed description]

## What Went Right
[Things that worked]

## Action Items
- [ ] [Preventive action 1]
- [ ] [Preventive action 2]
```