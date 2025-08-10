# Shadowcat/Tapwire Comprehensive Rust Code Review — Tracker

## Overview
Coordinated plan to perform a thorough, high‑signal Rust code review of the Shadowcat proxy (and surrounding Tapwire integration) using `CURSOR_RUST_CODE_REVIEWER.md`. This tracker breaks the work into phases and tasks, with deliverables saved under `reviews/cursor/`.

Note: Planning and analysis artifacts live in the main Tapwire repo under `reviews/cursor/*`. The actual codebase under review is the detached Shadowcat worktree at `shadowcat-cursor-review/` (a sibling of `shadowcat/`), serving as a read‑only snapshot for analysis.

**Last Updated**: 2025‑08‑10  
**Total Estimated Duration**: 18–28 hours  
**Status**: In Progress

## Goals
1. **Safety and Correctness** — Audit unsafe code, lifetime/ownership, and concurrency boundaries.
2. **Performance** — Identify hot‑path allocations, algorithmic issues, and logging overhead; target < 5% p95 latency overhead.
3. **API and Design Quality** — Ensure clean trait boundaries, documented public APIs, and consistent error handling.
4. **Testing and Tooling** — Improve test coverage for critical paths; ensure `cargo fmt`, `clippy -D warnings`, and `cargo test` are green.
5. **Security and Compliance** — Verify no client tokens are forwarded upstream; OAuth 2.1 compliance basics and transport security checks.

## Architecture Vision
```
Client ↔ Transports (stdio/http/sse) ↔ Proxy Engine (forward/reverse) ↔ Interceptors ↔ Upstream MCP Servers
                              ↘ Session Manager ↘ Recorder ↘ Auth Gateway ↘ Metrics
```

## Work Phases

### Phase A: Foundations and Inventory (Week 1)
Establish baseline understanding, inventory critical modules, and define measurement points.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.1 | **Repo inventory and map critical modules** | 2h | None | ✅ Complete | | Deliverable: module map in `analysis/findings/modules.md` |
| A.2 | **Review build, lint, and test status** | 2h | A.1 | ✅ Complete | | All green on `shadowcat-cursor-review@eec52c8`; results in `analysis/tests/status.md` |
| A.3 | **Identify hot paths and workloads** | 1.5h | A.1 | ✅ Complete | | Payload classes and scenarios documented in `analysis/perf/workloads.md` |
| A.4 | **Define review scope and priorities** | 1h | A.1–A.3 | ⬜ Not Started | | Draft scope doc in `analysis/findings/scope.md` |

**Phase A Total**: 6.5 hours

### Phase B: Safety, Async, and Concurrency Audit (Week 1)
Deep audit of unsafe/FFI, cancellation safety, and concurrency controls.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.1 | **Unsafe/FFI audit** | 3h | A.* | ✅ Complete | | Expanded Drop behavior review (forward proxy, SSE types) and metrics accumulation proposal; still no `unsafe`/FFI. See `analysis/safety/unsafe-audit.md`. |
| B.2 | **Cancellation safety review** | 2.5h | A.* | ✅ Complete | | Concrete shutdown/token patterns added for forward proxy, health checker, stdio; linked API sketches in `analysis/async/proposals.md`. |
| B.3 | **Send/Sync and locking analysis** | 2h | A.* | ✅ Complete | | Proposed lock-free metrics and await-outside-lock for replay receive; minor doc notes. See `analysis/async/locking.md`. |

**Phase B Total**: 7.5 hours

### Phase C: API, Error Handling, and Module Boundaries (Week 2)
Assess public APIs, trait design, error types, and module visibility.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | **Transport trait and implementations review** | 2h | A.* | 🔄 In Progress | | Updated prep with proposals on defaults, shutdown, and impl notes; `analysis/api/transport.md`. |
| C.2 | **Proxy engine and session lifecycle review** | 2h | A.* | 🔄 In Progress | | Added API notes and lifecycle proposals; `analysis/api/proxy-session.md`. |
| C.3 | **Error handling and Result flows** | 1.5h | A.* | 🔄 In Progress | | Added reverse proxy mapping notes and taxonomy proposal; `analysis/api/errors.md`. |
| C.4 | **Public API docs and examples** | 1h | C.1–C.3 | ⬜ Not Started | | Gaps and proposals `analysis/api/docs.md` |

**Phase C Total**: 6.5 hours

### Phase D: Performance, Recording, and Interceptors (Week 2)
Focus on hot‑path performance, recording engine overhead, and interceptor chain efficiency.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.1 | **Hot‑path allocation and logging audit** | 2h | A.3 | ⬜ Not Started | | Use `tracing`; minimize clones; `analysis/perf/hot-paths.md` |
| D.2 | **Recorder overhead and memory usage** | 1.5h | A.3 | ⬜ Not Started | | Tape storage, streaming, buffering; `analysis/perf/recorder.md` |
| D.3 | **Interceptor chain performance** | 1.5h | A.3 | ⬜ Not Started | | Rule evaluation cost; `analysis/perf/interceptors.md` |

**Phase D Total**: 5 hours

### Phase E: Security and Compliance (Week 2)
Validate auth gateway basics, token handling, and transport security checks.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| E.1 | **Token handling and header scrubbing** | 1.5h | A.* | ⬜ Not Started | | Ensure no client tokens pass upstream; `analysis/security/tokens.md` |
| E.2 | **OAuth 2.1 and transport security checks** | 1.5h | A.* | ⬜ Not Started | | Origin validation, DNS rebinding, TLS; `analysis/security/transport.md` |

**Phase E Total**: 3 hours

### Status Legend
- ⬜ Not Started — Task not yet begun
- 🔄 In Progress — Currently being worked on
- ✅ Complete — Task finished and tested
- ❌ Blocked — Cannot proceed due to dependency or issue
- ⏸️ Paused — Temporarily halted

## Progress Tracking

### Week 1 (2025‑08‑11 → 2025‑08‑15)
- [x] A.1: Repo inventory and map critical modules
- [x] A.2: Review build, lint, and test status
- [x] A.3: Identify hot paths and workloads
- [x] Phase B kickoff: created `analysis/safety/unsafe-audit.md`, `analysis/async/cancellation.md`, `analysis/async/locking.md`

### Completed Tasks
- [x] A.1 — Modules mapped (`analysis/findings/modules.md`)
- [x] A.2 — Build/lint/tests baseline recorded (`analysis/tests/status.md`)
- [x] A.3 — Hot paths & workloads defined (`analysis/perf/workloads.md`)

## Success Criteria

### Functional Requirements
- ✅ Transport and proxy reviews cover both forward and reverse modes
- ✅ Session lifecycle and recording behavior documented
- ✅ Error handling reviewed and actionable edits identified

### Performance Requirements
- ✅ < 5% p95 latency overhead target maintained or improvement plan proposed
- ✅ Memory usage bounded and streaming used appropriately in hot paths

### Quality Requirements
- ✅ clippy passes with `-D warnings`
- ✅ Tests pass locally; new test gaps identified with proposals
- ✅ Public APIs documented or plan created

## Risk Mitigation
| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Large scope creates churn | MEDIUM | Phased plan, task scoping per session | Active |
| Performance regressions | HIGH | Benchmarks + logging review before changes | Planned |
| Submodule commit workflow mistakes | MEDIUM | Follow `CLAUDE.md` submodule rules strictly | Active |

## Session Planning Guidelines
Follow `CURSOR_RUST_CODE_REVIEWER.md` for review methodology, command hints, and templates.

### NEXT_SESSION_PROMPT content checklist (for reviews/cursor/*.md)
- Repo/context: working dir, branch, commit, scope (analysis-only vs edits)
- Pinned references: reviewer guide, tracker, analysis README
- Tasks: explicit, with file paths to update and commands to run
- Success criteria: measurable outcomes to mark tasks done
- Deliverables: exact files to create/update
- Notes: constraints (no code edits, no rebase), resolved items to avoid re-flagging

## Related Documents
- Reviewer guide: `./../../CURSOR_RUST_CODE_REVIEWER.md`
- Project rules and commands: `./../../CLAUDE.md`
- Plans template: `./../../plans/tracker-template.md`

## Next Actions
1. Continue Phase C: refine interceptor effects and lifecycle docs; start C.4 public API docs/examples.

---

**Document Version**: 0.3  
**Created**: 2025‑08‑10  
**Last Modified**: 2025‑08‑10  
**Author**: Cursor GPT‑5 Reviewer

## Revision History
| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025‑08‑10 | 0.1 | Initial tracker creation | Cursor GPT‑5 Reviewer |
| 2025‑08‑10 | 0.2 | Updated A.1 complete; added A.2/A.3 in progress | Cursor GPT‑5 Reviewer |
| 2025‑08‑10 | 0.3 | Added NEXT_SESSION_PROMPT content checklist | Cursor GPT‑5 Reviewer |