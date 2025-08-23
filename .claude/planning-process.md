# Planning and Task Management Process

When tackling complex features or refactors, use this structured planning approach to manage context across multiple Claude sessions.

## 1. Create a Plan Directory

For significant features/refactors, create a dedicated directory under `plans/`:

```
plans/
├── feature-name/
│   ├── feature-tracker.md      # Main tracking document
│   ├── tasks/                   # Individual task files
│   │   ├── A.0-first-task.md
│   │   ├── A.1-second-task.md
│   │   └── ...
│   └── analysis/                # Output documents
│       ├── findings.md
│       └── README.md
```

## 2. Set Up the Tracker

Copy `plans/tracker-template.md` to create your feature tracker:

- Define problem and goals clearly
- Break work into incremental phases
- Create task table with duration estimates
- Include risk assessment and success criteria

## 3. Create Task Files

Each task should be:

- **Self-contained**: Completable in 1-3 hours within single session
- **Well-defined**: Clear objectives, deliverables, success criteria
- **Sequenced**: Dependencies clearly marked
- **Documented**: Outputs go to `analysis/` directory

Task files should include:

- Objective and key questions
- Step-by-step process
- Commands to run
- Expected deliverables with location/structure
- Success criteria checklist

## 4. Use next-session-prompt.md

At session end, update `next-session-prompt.md` in the plan directory:

- Focus on 1-3 tasks that fit in single session (5-10 hours work)
- Reference tracker for full context
- List specific task files to complete
- Define clear success criteria
- Keep concise - tracker holds full context

## 5. Track Progress

- Update task status in tracker and task files
- Write findings to `analysis/` directory
- Update tracker's key findings section
- Clear completed todos with TodoWrite tool

## Example Flow

1. **Session 1**: Complete analysis tasks, update tracker, create next-session-prompt for design
2. **Session 2**: Complete design tasks, update tracker, create next-session-prompt for Phase 1
3. **Session 3**: Implement Phase 1, update tracker, continue...

## Benefits

- **Context continuity** across sessions
- **Clear progress tracking**
- **Manageable scope** per session
- **Comprehensive documentation**
- **Reduced cognitive load** by focusing on specific tasks

