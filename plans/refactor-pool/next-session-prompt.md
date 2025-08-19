# Next Session: Phase A — Analysis & Design

## Project Context

Extract the generic connection pool into `shadowcat::pool`, align with sqlx patterns, and keep migration churn low.

**Project**: Refactor Pool to shadowcat::pool  
**Tracker**: `plans/refactor-pool/refactor-pool-tracker.md`  
**Status**: Phase A — Analysis & Design (0% Complete)

## Current Status

### What Has Been Completed
- **Pre-work** (✅ Completed Aug 19)
  - Stabilized current pool (receiver ownership, first-tick handling)
  - Backpressure-safe return path (close with timeout)
  - Last-reference async cleanup backstop

### What's In Progress
- **A.0**: Current State Analysis (⬜ Not Started)
  - Duration: 2h
  - Dependencies: None

## Your Mission

Focus on analysis deliverables to de-risk design/implementation.

### Priority 1: A.0 Current State Analysis (2h)

1. Inventory components in `shadowcat/src/proxy/pool.rs`
   - Deliverable: list of generic vs transport-specific items
   - Success: Clear separation plan (what moves to `shadowcat::pool`, what stays near consumers)

2. Identify migration touch points
   - Deliverable: list of import sites to be updated; re-export plan from old path
   - Success: Low-churn migration strategy

### Priority 2: A.1 sqlx Patterns Review (1h)
- Deliverable: succinct checklist of sqlx features we want now vs. later (CloseEvent, is_closed, RAII guard, ArrayQueue, hooks)
- Success: Prioritized adoption plan

### Priority 3: A.2 Design Proposal & API (2h)
- Deliverable: proposed `shadowcat::pool` module layout and public API (`Pool`, `PoolOptions`, `PoolConnection`, `PoolStats`, `PoolableResource`)
- Success: Signed-off API and file structure; ready for scaffolding

## Essential Context Files to Read

1. **Primary Tracker**: `plans/refactor-pool/refactor-pool-tracker.md`
2. **Findings**: `research/connection-pool-cleanup-gpt5/*`
3. **Current Implementation**: `shadowcat/src/proxy/pool.rs`
4. **sqlx Reference**: `~/src/sqlx/sqlx-core/src/pool/*`

## Working Directory

```bash
cd /Users/kevin/src/tapwire
# Worktree for code lives in shadowcat-connection-pooling (branch: refactor/pool)
# Plan docs live in plans/refactor-pool
```
