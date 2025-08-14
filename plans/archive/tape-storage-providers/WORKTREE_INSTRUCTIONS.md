# Git Worktree Instructions for Tape Storage Providers

## 🔴 CRITICAL: All Shadowcat Development Must Use the Worktree

### Worktree Details
- **Location**: `shadowcat-tape-storage-providers`
- **Branch**: `feat/tape-storage-providers`
- **Initial Commit**: e57bb75

### Why Use a Worktree?

Using a git worktree allows parallel development without interfering with other ongoing work:
- Main shadowcat directory stays on main branch for transport-refactor work
- This feature development happens isolated in its own branch
- No need to stash/switch branches constantly
- Can run tests in both directories simultaneously

### Essential Commands

```bash
# Navigate to worktree (ALWAYS DO THIS FIRST)
cd shadowcat-tape-storage-providers

# Verify you're in the right place
git status
# Should show: On branch feat/tape-storage-providers

pwd
# Should end with: shadowcat-tape-storage-providers

# Make changes, test, commit as normal
cargo test
cargo clippy --all-targets -- -D warnings
git add .
git commit -m "feat(tape-storage): implement XYZ"
git push origin feat/tape-storage-providers
```

### DO NOT

- ❌ Edit shadowcat files in the main tapwire/shadowcat directory
- ❌ Run tape-storage tests in the main shadowcat directory
- ❌ Forget to verify branch before making changes

### DO

- ✅ Always `cd shadowcat-tape-storage-providers` first
- ✅ Verify with `git status` before starting work
- ✅ Include worktree reminder in all next-session-prompt.md updates
- ✅ Mention worktree in commit messages if relevant

### For Next Session Prompts

Always include this reminder at the top:

```markdown
## 🔴 CRITICAL: Use Git Worktree

**ALL SHADOWCAT WORK MUST BE DONE IN THE WORKTREE:**
\```bash
cd shadowcat-tape-storage-providers
git status  # Verify: On branch feat/tape-storage-providers
\```
```

### Relationship to Main Repository

```
tapwire/
├── shadowcat/                    # Main submodule (stays on main branch)
├── shadowcat-tape-storage-providers/  # Worktree (feat/tape-storage-providers branch)
├── plans/
│   └── tape-storage-providers/   # Plan documents (in main tapwire)
└── ...
```

### When Feature is Complete

After merging the feature:
1. Switch to main shadowcat directory
2. Pull the merged changes
3. Remove the worktree: `git worktree remove shadowcat-tape-storage-providers`

---

**Remember**: Every shadowcat code change for this feature happens in the worktree!