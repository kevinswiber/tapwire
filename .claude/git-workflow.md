# Git Workflow

## Submodule Management
**Critical**: Shadowcat is a git submodule - commit there first!

### Initial Setup
- Clone with submodules: `git clone --recursive <repo>`
- Initialize existing: `git submodule init && git submodule update`

### Working with Shadowcat Submodule
```bash
# 1. Enter submodule
cd shadowcat

# 2. Make changes on appropriate branch
git checkout main
# ... make changes ...

# 3. Commit in submodule FIRST
git add .
git commit -m "feat: implement feature"
git push

# 4. Update parent repo pointer
cd ..
git add shadowcat
git commit -m "chore: update shadowcat submodule"
git push
```

## Commit Guidelines
- **DO NOT add Claude as co-author**
- **DO NOT mention Claude Code in messages**
- Use conventional commits: `feat:`, `fix:`, `docs:`, `chore:`, `test:`
- Keep messages focused on technical changes

## Branch Strategy
- Main branch: `main`
- Feature branches: `feature/description`
- Bugfix branches: `fix/description`
- Always PR to main, never direct push

## Common Git Commands
- Status: `git status`
- Diff staged: `git diff --staged`
- Recent commits: `git log --oneline -10`
- Update submodule: `git submodule update --remote`
- Fetch all: `git fetch --all --recurse-submodules`