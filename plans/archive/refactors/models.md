Use Claude Sonnet (Faster, Cost-Effective) For:

  Phase 1: Critical Safety Tasks

  - Task 001: Remove unwraps - Mechanical, repetitive work
  - Task 002: Fix duplicate errors - Simple deletion/cleanup
  - Task 003: Add size limits - Straightforward implementation with clear patterns

  Phase 3: Dead Code & Optimization

  - Task 010: Remove dead code - Mostly deletion, following compiler warnings
  - Task 012: Optimize strings - Pattern-based replacements
  - Task 013: Fix Arc RwLock - Mechanical refactoring

  Simple Phase 4 Tasks

  - Task 016: Add audit logging - Adding log statements
  - Task 017: Complete TODOs - If they're simple/well-defined

  Why: These tasks are well-defined, repetitive, and don't require deep architectural understanding.
  Sonnet can handle them efficiently.

  Use Claude Opus (More Capable) For:

  Phase 2: Core Features (All Critical)

  - Task 005: Implement record command - Complex new feature
  - Task 006: Implement replay command - Complex state management
  - Task 007: Implement rate limiting - Algorithm implementation
  - Task 008: Session matching - Complex logic
  - Task 009: Session cleanup - Concurrent programming

  Complex Phase 1 Tasks

  - Task 004: Fix blocking IO - Requires understanding async/await deeply

  Complex Phase 3 Tasks

  - Task 011: Optimize clones - Requires analyzing data flow and ownership

  Complex Phase 4 Tasks

  - Task 014: Security validation - Security critical
  - Task 015: Circuit breaker - Complex pattern implementation
  - Task 018: Add metrics - System-wide integration

  Why: These require architectural understanding, complex logic, or have high stakes (security/core
  features).

  Recommended Workflow:

  1. Start each phase with Opus to review the plan and identify any issues
  2. Use Sonnet for mechanical tasks within that phase
  3. Switch to Opus when you hit complexity or need architectural decisions
  4. Use Opus for final review of each phase before moving on

  Quick Decision Tree:

  Is it a new feature implementation? → Opus
  Is it security-critical? → Opus
  Does it require understanding multiple modules? → Opus
  Is it mostly find-and-replace? → Sonnet
  Is it following a clear pattern from the task doc? → Sonnet
  Are you stuck or confused? → Switch to Opus

  Cost-Optimization Strategy:

  For maximum efficiency, you could even break down individual tasks:
  - Use Sonnet to find all instances (e.g., all unwraps)
  - Use Opus to handle the complex cases
  - Use Sonnet to apply the pattern to remaining simple cases

  This approach would minimize Opus usage while ensuring quality on critical parts.