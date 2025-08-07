Shadowcat Refactor: Task 001 - Remove All Unwrap Calls

  You are helping me refactor the Shadowcat Rust proxy codebase. Please read the following context
  files first to understand the task:

  1. Task Definition: /Users/kevin/src/tapwire/plans/refactors/task-001-remove-unwraps.md - Contains
  detailed instructions for removing unwrap calls
  2. Overall Refactor Plan: /Users/kevin/src/tapwire/plans/refactors/shadowcat-refactor-tracker.md -
  Shows where this task fits in the larger refactor
  3. Code Review: /Users/kevin/src/tapwire/reviews/shadowcat-comprehensive-review-2025-08-06.md -
  Original review that identified 1,338 unwrap calls as critical issue #1

  Your Objective

  Replace all .unwrap() calls in non-test code with proper error handling to prevent runtime panics.
  There are currently 1,338 instances that need to be fixed.

  Working Directory

  /Users/kevin/src/tapwire/shadowcat

  Approach

  1. Start by analyzing - Count and categorize the unwraps by module
  2. Work module by module - Start with highest priority modules listed in the task file
  3. Add missing error variants before fixing each module
  4. Test after each module to catch issues early

  Model Usage Strategy

  Please explicitly tell me when to switch between Claude Opus and Claude Sonnet:

  Use OPUS for:

  - Initial analysis and categorization of unwrap types
  - Complex unwraps in core logic (session manager, transport layer)
  - Any unwrap where the correct error handling isn't obvious
  - Creating new error variants or modifying error types
  - Final review of the completed work

  Use SONNET for:

  - Mechanical replacement of simple unwraps (like .expect() in tests)
  - Applying established patterns repeatedly
  - Unwraps in configuration parsing (once pattern is established)
  - Unwraps in CLI code (usually straightforward)
  - Running validation commands and counting remaining unwraps

  Tell me explicitly: "Switch to SONNET now" or "Switch to OPUS now" when appropriate.

  First Steps

  1. First, run a count of current unwraps to establish baseline
  2. Analyze and categorize them by module and complexity
  3. Create a plan for which order to tackle them
  4. Tell me whether to continue with Opus or switch to Sonnet for the mechanical work

  Success Criteria

  - Zero unwrap() calls in non-test code (verify with: rg '\.unwrap\(\)' --type rust -g '!tests/**' -g
   '!test/**' | wc -l)
  - All tests still pass
  - No new clippy warnings

  Important Notes

  - The task file has specific patterns for common unwrap replacements - follow those
  - Some unwraps might be genuinely safe but should still be replaced with .expect() with a
  descriptive message
  - Don't use unwrap_or for Results as it silently ignores errors
  - Test error paths to ensure error handling works correctly

  Begin by reading the three context files mentioned above, then start the analysis phase.