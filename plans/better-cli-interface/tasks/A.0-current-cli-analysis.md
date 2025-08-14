# Task A.0: Current CLI Analysis

## Objective

Analyze the existing Shadowcat CLI implementation to understand the current command structure, argument parsing, and execution flow. This analysis will inform the design of the improved interface.

## Background

The current CLI uses "forward" and "reverse" terminology that many developers find confusing. Before implementing improvements, we need to thoroughly understand:
- How commands are currently structured
- Where the parsing logic lives
- How arguments flow through the system
- What constraints exist from dependencies (clap, etc.)

## Key Questions to Answer

1. How is the current CLI structured in main.rs and related modules?
2. What are all the current command variations and their options?
3. Where is the command routing logic and how modular is it?
4. What would break if we change the command structure?
5. How are transports currently selected and initialized?

## Step-by-Step Process

### 1. Analysis Phase (30 min)
Examine the current CLI implementation structure

```bash
# Commands to understand current state
cd shadowcat
grep -r "enum.*Command" src/
grep -r "clap::" src/main.rs
grep -r "forward\|reverse" src/
```

### 2. Document Current Commands (45 min)
Create comprehensive documentation of all current commands

Commands to analyze:
- `shadowcat forward [options]`
- `shadowcat reverse [options]`
- `shadowcat record [options]`
- `shadowcat replay [options]`
- Any other commands

### 3. Trace Execution Flow (45 min)

Trace how commands flow from CLI to execution:
1. main.rs command parsing
2. Command enum structure
3. Match arms and routing
4. Transport initialization
5. Proxy creation and execution

## Expected Deliverables

### Analysis Document
- `analysis/current-cli-structure.md` - Complete documentation of current CLI
- Includes:
  - Command hierarchy diagram
  - All command options and flags
  - Execution flow diagram
  - Code coupling analysis
  - Breaking change assessment

### Key Findings
- List of all hardcoded command strings
- Dependencies on command structure
- Integration points that need updates
- Backward compatibility requirements

## Success Criteria Checklist

- [ ] All current commands documented
- [ ] Execution flow traced and documented
- [ ] Dependencies identified
- [ ] Breaking changes assessed
- [ ] Analysis document created
- [ ] Tracker updated with findings

## Risk Assessment

| Risk | Impact | Mitigation | 
|------|--------|------------|
| Complex command routing | MEDIUM | Document all paths thoroughly |
| Hidden dependencies | LOW | Search for string literals |
| Test dependencies | MEDIUM | Identify all test files using CLI |

## Duration Estimate

**Total: 2 hours**
- Analysis: 30 minutes
- Documentation: 45 minutes
- Flow tracing: 45 minutes

## Dependencies

None - this is the first task

## Integration Points

- **main.rs**: Primary CLI entry point
- **proxy module**: Command execution
- **transport module**: Transport selection
- **Tests**: Integration tests using CLI

## Notes

- Pay special attention to how stdio detection works
- Note any hardcoded strings that would need updating
- Consider how the help text is generated

## Commands Reference

```bash
# Quick reference of useful commands for this task
cd shadowcat

# Find command definitions
grep -n "enum.*Command" src/
grep -n "#\[derive.*Parser" src/
grep -n "#\[command" src/

# Find command usage
grep -r "Command::" src/
grep -r "\.forward" src/
grep -r "\.reverse" src/

# Understand structure
cat src/main.rs
tree src/ -I target
```

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-14
**Last Modified**: 2025-01-14
**Author**: Kevin