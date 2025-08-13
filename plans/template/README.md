# Creating a New Plan

This document explains how to create and structure a new plan for the Shadowcat/Tapwire project. Plans help manage complex features, refactors, or investigations that span multiple Claude sessions.

## Overview

A plan is a structured approach to tackling a significant piece of work. Each plan consists of:
- A tracker document that coordinates the overall effort
- A next-session prompt for setting up work sessions
- Individual task files that break down the work
- Analysis/output documents that capture findings

## Directory Structure

```
plans/
├── {plan-name}/
│   ├── {plan-name}-tracker.md        # Main tracking document (from template/tracker.md)
│   ├── next-session-prompt.md        # Session setup (from template/next-session-prompt.md)
│   ├── tasks/                        # Individual task files (from template/task.md)
│   │   ├── A.1-{task-name}.md       # Phase A, Task 1
│   │   ├── A.2-{task-name}.md       # Phase A, Task 2
│   │   ├── B.1-{task-name}.md       # Phase B, Task 1
│   │   └── ...
│   └── analysis/                     # Output documents
│       ├── findings.md               # Analysis results
│       ├── design-decisions.md       # Design choices made
│       └── README.md                 # Summary of outputs
```

## Step-by-Step Process

### 1. Start with Analysis

**IMPORTANT**: Most plans should begin with an analysis phase before committing to write code. This helps:
- Understand the problem space thoroughly
- Investigate potential solutions
- Identify risks and dependencies
- Make informed design decisions
- Avoid wasted implementation effort

Typical analysis tasks:
- **A.0**: Problem Analysis - Understand current state and limitations
- **A.1**: Solution Research - Investigate potential approaches
- **A.2**: Feasibility Study - Evaluate technical constraints
- **A.3**: Design Proposal - Document recommended approach

### 2. Create the Plan Directory

```bash
# Create the plan structure
mkdir -p plans/{plan-name}/tasks
mkdir -p plans/{plan-name}/analysis

# Example for a new feature
mkdir -p plans/sse-transport-enhancement/tasks
mkdir -p plans/sse-transport-enhancement/analysis
```

### 3. Create the Tracker

Copy and customize the tracker template:

```bash
cp plans/template/tracker.md plans/{plan-name}/{plan-name}-tracker.md
```

Edit the tracker to:
- Define clear goals and success criteria
- Break work into phases (Analysis → Design → Implementation → Testing)
- Create task entries with realistic time estimates (2-4 hours each)
- Identify dependencies between tasks
- Include risk assessment

Example phases:
- **Phase 0**: Analysis & Investigation (understand before building)
- **Phase 1**: Design & Architecture (plan the solution)
- **Phase 2**: Core Implementation (build the basics)
- **Phase 3**: Integration (connect with existing systems)
- **Phase 4**: Testing & Validation (ensure quality)
- **Phase 5**: Documentation & Polish (prepare for users)

### 4. Create the Next Session Prompt

Copy and customize the session template:

```bash
cp plans/template/next-session-prompt.md plans/{plan-name}/next-session-prompt.md
```

Configure for the first session:
- Focus on 1-3 analysis tasks
- List specific questions to answer
- Define clear deliverables (usually documents in `analysis/`)
- Set realistic time expectations

### 5. Create Initial Task Files

For each task in your tracker, create a task file:

```bash
cp plans/template/task.md plans/{plan-name}/tasks/A.0-problem-analysis.md
```

Task naming convention:
- `{Phase}.{Number}-{descriptive-name}.md`
- Examples:
  - `A.0-problem-analysis.md`
  - `A.1-solution-research.md`
  - `B.1-implement-core-logic.md`
  - `C.1-integration-tests.md`

### 6. Define Analysis Outputs

Create placeholders for analysis results:

```bash
touch plans/{plan-name}/analysis/README.md
touch plans/{plan-name}/analysis/findings.md
touch plans/{plan-name}/analysis/design-decisions.md
```

## Best Practices

### Task Sizing
- **2-4 hours**: Ideal task size for a Claude session
- **1 hour**: Minimum for meaningful progress
- **6 hours**: Maximum before context becomes unwieldy
- Break larger work into multiple tasks

### Dependency Management
- Clearly mark task dependencies in the tracker
- Order tasks to minimize blocking
- Identify parallel work opportunities
- Create stub implementations when needed

### Session Management
- Update `next-session-prompt.md` at the end of each session
- Include completed work and remaining tasks
- Reference specific task files for the next session
- Keep prompts focused (1-3 tasks maximum)

### Documentation
- Write findings to `analysis/` directory as you go
- Update task status immediately upon completion
- Document decisions and rationale
- Include code examples in task files

### Quality Gates
Each phase should have clear quality gates:
- **Analysis**: Questions answered, risks identified
- **Design**: Architecture documented, approach validated
- **Implementation**: Tests passing, no clippy warnings
- **Integration**: End-to-end flows working
- **Documentation**: User guide complete, examples provided

## Example: Creating an SSE Enhancement Plan

```bash
# 1. Create structure
mkdir -p plans/sse-enhancement/tasks
mkdir -p plans/sse-enhancement/analysis

# 2. Copy templates
cp plans/template/tracker.md plans/sse-enhancement/sse-enhancement-tracker.md
cp plans/template/next-session-prompt.md plans/sse-enhancement/next-session-prompt.md

# 3. Create analysis tasks
cp plans/template/task.md plans/sse-enhancement/tasks/A.0-current-state-analysis.md
cp plans/template/task.md plans/sse-enhancement/tasks/A.1-mcp-spec-review.md
cp plans/template/task.md plans/sse-enhancement/tasks/A.2-design-proposal.md

# 4. Create implementation tasks
cp plans/template/task.md plans/sse-enhancement/tasks/B.1-transport-abstraction.md
cp plans/template/task.md plans/sse-enhancement/tasks/B.2-sse-client.md
cp plans/template/task.md plans/sse-enhancement/tasks/B.3-sse-server.md

# 5. Edit tracker with specific goals and phases
vim plans/sse-enhancement/sse-enhancement-tracker.md

# 6. Set up first session
vim plans/sse-enhancement/next-session-prompt.md
# Focus on A.0-A.2 analysis tasks
```

## Common Plan Types

### Feature Implementation
1. Analysis: Understand requirements and constraints
2. Design: Create architecture and API design
3. Implementation: Build core functionality
4. Integration: Connect with existing systems
5. Testing: Comprehensive test coverage
6. Documentation: User guides and examples

### Refactoring
1. Analysis: Identify problems and measure impact
2. Planning: Design migration strategy
3. Preparation: Create compatibility layers
4. Migration: Incrementally refactor
5. Cleanup: Remove old code
6. Validation: Ensure no regressions

### Bug Investigation
1. Reproduction: Create minimal test case
2. Analysis: Root cause analysis
3. Solution Design: Evaluate fix approaches
4. Implementation: Apply fix
5. Testing: Verify fix and check for regressions
6. Documentation: Update relevant docs

### Performance Optimization
1. Profiling: Identify bottlenecks
2. Analysis: Understand performance characteristics
3. Design: Plan optimization strategy
4. Implementation: Apply optimizations
5. Benchmarking: Measure improvements
6. Documentation: Record performance gains

## Tracking Progress

### Status Indicators
- ⬜ Not Started
- 🔄 In Progress
- ✅ Complete
- ❌ Blocked
- ⏸️ Paused
- 🔍 Under Review

### Progress Updates
After each session:
1. Update task status in tracker
2. Check off completed items
3. Add completion dates
4. Note any new findings or blockers
5. Update next-session-prompt.md
6. Commit changes with clear message

## Tips for Success

1. **Start Small**: Begin with analysis before committing to large changes
2. **Be Specific**: Clear task definitions lead to better outcomes
3. **Track Everything**: Document decisions, findings, and rationale
4. **Maintain Context**: Keep session prompts focused and updated
5. **Test Incrementally**: Validate work as you go
6. **Communicate Status**: Keep tracker current for team visibility

## Related Templates

- `plans/template/tracker.md` - Main tracker template
- `plans/template/next-session-prompt.md` - Session prompt template
- `plans/template/task.md` - Individual task template
- `CLAUDE.md` - Project-wide guidelines

---

Remember: The goal of planning is to break complex work into manageable pieces that can be completed successfully across multiple Claude sessions while maintaining context and quality.