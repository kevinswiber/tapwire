# Claude Code Command Development Guide

This guide provides best practices and patterns for creating custom Claude Code slash commands based on the official documentation at https://docs.anthropic.com/en/docs/claude-code/slash-commands.

## Table of Contents
- [Command Structure](#command-structure)
- [Bash Command Guidelines](#bash-command-guidelines)
- [File References](#file-references)
- [Tool Permissions](#tool-permissions)
- [Common Patterns](#common-patterns)
- [Troubleshooting](#troubleshooting)

## Command Structure

### Basic Command Template

```markdown
---
description: Brief description of what the command does
argument-hint: <optional-arguments>
allowed-tools:
  - tool1
  - tool2
---

# Command Title

Command content goes here...
```

### Header Fields

| Field | Required | Description | Example |
|-------|----------|-------------|---------|
| `description` | Yes | Brief description shown in command list | `List all available development plans` |
| `argument-hint` | No | Shows expected arguments in UI | `<plan-name> [phase-id]` |
| `allowed-tools` | No | List of bash commands allowed without approval | `- ls`<br>`- grep`<br>`- find` |

## Bash Command Guidelines

**⚠️ IMPORTANT**: External bash scripts (`.sh` files) do NOT work in Claude Code commands. All bash commands must be inline within the command file itself.

### 1. Keep Commands Simple

❌ **Avoid complex shell operations:**
```markdown
!`ls -la plans/$ARGUMENTS/ 2>/dev/null || echo "Plan '$ARGUMENTS' not found"`
```

✅ **Use simple, single operations:**
```markdown
!`ls -la plans/$ARGUMENTS/`
```

### 2. Avoid These Shell Features

Commands with these features will require user approval:

- **Command substitution**: `$(command)` or `` `command` ``
- **Pipes**: `|`
- **Redirections**: `>`, `>>`, `<`, `2>`, `&>`
- **Conditional execution**: `&&`, `||`
- **Semicolons**: `;`
- **Background execution**: `&`
- **Newlines in commands** (except in quoted strings)
- **Complex loops and conditionals**

### 3. Use Allowed-Tools for Common Commands

Add frequently used commands to the `allowed-tools` header:

```markdown
---
allowed-tools:
  - ls
  - find
  - grep
  - cat
  - echo
  - pwd
  - basename
  - dirname
  - wc
  - head
  - tail
  - sort
  - xargs
  - sed
---
```

### 4. Pattern-Based Permissions

You can specify patterns for more flexibility:

```markdown
---
allowed-tools:
  - ls:*           # Allow any ls command
  - grep:*.md      # Allow grep on .md files
  - find:plans/*   # Allow find in plans directory
---
```

## File References

### Loading Files with @

Use `@` to load file contents directly:

```markdown
## Load Specific File
@path/to/file.md

## Load with Wildcards
@plans/$ARGUMENTS/*.md

## Multiple Files
@src/main.rs
@src/lib.rs
```

### Dynamic File Paths

Use `$ARGUMENTS` for dynamic file references:

```markdown
## Load plan files
@plans/$ARGUMENTS/tracker.md
@plans/$ARGUMENTS/next-session-prompt.md
```

## Tool Permissions

### Understanding Permission Errors

When you see errors like:
```
Error: Bash command permission check failed for pattern "!`command`": 
This Bash command contains multiple operations. The following part requires approval: ...
```

This means the command violates the simple command rules. Solutions:

1. **Simplify the command** - Remove pipes, redirections, and conditional logic
2. **Add to allowed-tools** - If it's a simple command, add it to the header
3. **Break into multiple commands** - Use separate bash blocks
4. **Use simple inline commands** - Avoid external scripts as they don't work in Claude Code

### Avoiding External Scripts

**IMPORTANT**: External bash scripts (`.sh` files) do NOT work in Claude Code commands. They will fail with permission errors even when added to `allowed-tools`.

❌ **This will NOT work:**
```markdown
---
allowed-tools:
  - .claude/scripts/list-plans.sh
---

!`.claude/scripts/list-plans.sh`
```

✅ **Instead, use inline commands:**
```markdown
---
allowed-tools:
  - ls
  - find
---

!`ls -d plans/*/`
!`find plans -maxdepth 1 -type d`
```

For complex logic that would normally require a script, either:
1. Break it into multiple simple inline commands
2. Use Claude's capabilities to process the output
3. Provide instructions for manual filtering/processing

## Common Patterns

### Pattern 1: List Items with Conditions

Instead of:
```markdown
!`ls -d plans/*/ | grep -v archive | grep -v template`
```

Create a simple script or use multiple commands:
```markdown
!`ls -d plans/*/`

Then manually filter out archive and template in the command text.
```

### Pattern 2: Parse Arguments

Instead of complex bash parsing:
```markdown
!`echo "$ARGUMENTS" | { read plan phase; echo "Plan: $plan"; }`
```

Use simple display:
```markdown
Arguments provided: $ARGUMENTS

Usage: /command <plan> [phase]
```

### Pattern 3: Check File Existence

Instead of:
```markdown
!`[ -f "file.txt" ] && echo "exists" || echo "missing"`
```

Use:
```markdown
Check if file exists:
@file.txt
```

### Pattern 4: Count Items

Instead of:
```markdown
!`find . -name "*.md" | wc -l`
```

Use allowed tools:
```markdown
---
allowed-tools:
  - find
  - wc
---

!`find . -name "*.md"`
!`wc -l`
```

## Example Commands

### Simple Information Command

```markdown
---
description: Show project statistics
---

# Project Statistics

## File Counts
!`ls -la src/`

## Recent Changes
Check git status for recent modifications.

## Documentation
@README.md
```

### Command with Arguments

```markdown
---
description: Load a specific module for analysis
argument-hint: <module-name>
allowed-tools:
  - ls
  - find
---

# Analyze Module: $ARGUMENTS

## Module Structure
!`ls -la src/$ARGUMENTS/`

## Module Files
!`find src/$ARGUMENTS -name "*.rs"`

## Main Module File
@src/$ARGUMENTS/mod.rs
```

### Interactive Development Command

```markdown
---
description: Set up development environment for a feature
argument-hint: <feature-name>
allowed-tools:
  - mkdir
  - touch
  - ls
---

# Setup Feature: $ARGUMENTS

## Create Feature Directory
!`mkdir -p features/$ARGUMENTS`

## Create Basic Files
!`touch features/$ARGUMENTS/README.md`
!`touch features/$ARGUMENTS/implementation.md`

## Load Template
@templates/feature-template.md

## Next Steps
1. Edit the README.md with feature description
2. Update implementation.md with technical details
3. Create task breakdown in the tracker
```

## Troubleshooting

### Problem: Command Not Found

**Error**: "Command not found" when using `/mycommand`

**Solution**: Ensure file is in `.claude/commands/` and ends with `.md`

### Problem: Permission Denied

**Error**: "Bash command permission check failed"

**Solutions**:
1. Add the command to `allowed-tools` in header
2. Simplify the command to remove pipes/redirections
3. Break complex commands into multiple simple ones

### Problem: Arguments Not Working

**Error**: `$ARGUMENTS` not being replaced

**Solution**: Ensure you're using exactly `$ARGUMENTS` (all caps)

### Problem: Files Not Loading

**Error**: File references with `@` not working

**Solutions**:
1. Check file path is correct (relative to working directory)
2. Ensure no spaces around the `@` symbol
3. Wildcards only work for simple patterns

### Problem: Complex Logic Needed

**Scenario**: Need to perform complex conditional logic or loops

**Solution**: 
1. Break down into multiple simple commands
2. Use `find`, `ls`, `grep` with simple patterns
3. Let Claude process and filter the output
4. Provide manual instructions for complex filtering

## Best Practices

1. **Start Simple**: Begin with basic commands and add complexity gradually
2. **Test Incrementally**: Test each bash command separately before combining
3. **Document Usage**: Always include usage examples in your commands
4. **Provide Feedback**: Use echo or cat to provide user feedback
5. **Handle Errors Gracefully**: Assume commands might fail and provide context
6. **Keep Commands Focused**: Each command should do one thing well
7. **Use Descriptive Names**: Command filenames should be self-explanatory

## Testing Your Commands

Before deploying commands:

1. **Test Bash Commands**: Run each bash command manually first
2. **Check Permissions**: Verify all commands work with specified `allowed-tools`
3. **Test with Arguments**: Try different argument combinations
4. **Test Edge Cases**: Empty arguments, missing files, etc.
5. **Review Output**: Ensure output is clear and helpful

## Security Considerations

1. **Never Use eval**: Avoid `eval` or similar dangerous constructs
2. **Validate Arguments**: Don't pass `$ARGUMENTS` directly to dangerous commands
3. **Limit File Access**: Use specific paths rather than wildcards when possible
4. **Avoid Sensitive Data**: Don't include commands that might expose secrets
5. **Review Allowed Tools**: Only allow necessary commands in `allowed-tools`

## References

- [Official Documentation](https://docs.anthropic.com/en/docs/claude-code/slash-commands)
- [Claude Code Overview](https://docs.anthropic.com/en/docs/claude-code)
- [Example Commands](.claude/commands/) - See this directory for examples

---

*This guide is based on Claude Code documentation and practical experience developing commands for the Tapwire project.*