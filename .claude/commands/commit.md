---
description: Run clippy, fix issues, format, and commit changes in Rust project
argument-hint: [submodule] ["commit message"]
allowed-tools:
  - pwd
  - find:*Cargo.toml
  - cargo clippy:*
  - cargo fmt:*
  - git status
  - git add:*
  - git commit:*
  - git diff:*
  - basename:*
  - dirname:*
  - ls:*
  - cd:*
  - echo:*
---

# Commit Rust Project with Quality Checks

## Parse Arguments

Arguments provided: `$ARGUMENTS`

The command accepts:
- Optional first argument: submodule/worktree name (default: shadowcat)
- Remaining arguments: commit message

Examples:
- `/commit` - Use shadowcat with generated message
- `/commit "fix: update error handling"` - Use shadowcat with message
- `/commit shadowcat-cli-refactor "refactor: improve CLI structure"` - Specific worktree with message

## Determine Target Repository

Parsing arguments to determine target repository and commit message:

!`echo "Arguments: $ARGUMENTS"`

Setting up repository path based on first argument (default: shadowcat):

!`pwd`

## Navigate to Target Repository

Based on arguments, we'll work with the specified repository (or default to shadowcat).

### Check Available Repositories
!`ls -d shadowcat* 2>/dev/null || echo "No shadowcat directories found"`

### Verify Target Repository Exists
The target repository will be determined from arguments:
- If no arguments or arguments start with a quote/dash: use `shadowcat`
- Otherwise: use first argument as repository name

Check if target has Cargo.toml:
!`ls shadowcat/Cargo.toml 2>/dev/null || echo "shadowcat/Cargo.toml not found"`

## Working with the Target Repository

Now Claude will:

1. **Navigate to the target repository** (based on first argument or default to shadowcat)
2. **Run clippy** with strict warnings and fix any issues
3. **Run cargo fmt** to format the code
4. **Commit changes** with the provided message (or generate one)

### Steps to Execute:

```bash
# 1. Change to target directory (shadowcat by default, or first argument)
cd [target-repository]

# 2. Run clippy
cargo clippy --all-targets -- -D warnings

# 3. Fix any clippy issues if needed

# 4. Run formatter
cargo fmt

# 5. Check git status
git status --short

# 6. Stage changes
git add -A

# 7. Commit with message
git commit -m "[commit-message]"
```

## Check Parent Repository

After committing in the submodule/worktree, we need to update the parent tapwire repository:

### Check for parent repository changes:
!`git status --short`

### View submodule changes:
!`git diff --stat`

## Instructions for Claude

Based on the arguments provided, Claude will:

1. **Parse arguments** to identify:
   - Target repository (first arg if it looks like a repo name, otherwise "shadowcat")
   - Commit message (remaining args or generated)

2. **Navigate to target repository** and run quality checks:
   - `cd [target-repo]`
   - `cargo clippy --all-targets -- -D warnings`
   - Fix any clippy issues
   - `cargo fmt`

3. **Commit changes** in the target repository:
   - `git add -A`
   - `git commit -m "[message]"`

4. **Update parent tapwire repository** if needed:
   - Check for submodule changes
   - Commit submodule update

## Usage Examples

```bash
# Default: shadowcat with generated message
/commit

# Shadowcat with specific message
/commit "feat: add new transport layer"

# Specific worktree with message
/commit shadowcat-cli-refactor "refactor: update CLI structure"

# Worktree with multi-word message
/commit shadowcat-wassette "fix: resolve SSE connection issues"

# Just repo name (message will be generated)
/commit shadowcat-cursor-review
```

## How Arguments Are Parsed

- If `$ARGUMENTS` is empty → use shadowcat, generate message
- If `$ARGUMENTS` starts with quote or dash → use shadowcat, full args as message
- If first word looks like a shadowcat repo → use it as target, rest as message
- Otherwise → use shadowcat, full args as message

## Notes

- This command ensures code quality before committing
- Clippy warnings are treated as errors to maintain code standards
- Formatting is applied automatically
- Both the target repo and parent tapwire repo are handled
- For complex clippy fixes, Claude will help resolve them interactively
- Available repositories are detected automatically from the tapwire directory