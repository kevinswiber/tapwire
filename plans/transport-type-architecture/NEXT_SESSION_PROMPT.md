# Next Session Prompt - Transport Architecture Phase D

## Context
You are continuing the transport architecture refactor. Phases A-C are complete:
- Phase A: Deep analysis completed
- Phase B: ResponseMode enum implemented, is_sse_session removed
- Phase C: Common utilities created then removed (over-engineered)
- Phase D: Re-evaluated and revised to targeted improvements only

## Current Focus: Phase D - Targeted Improvements

After lessons learned from Phase C (where we over-engineered shared utilities), Phase D has been revised from "Unify Proxy Architectures" to focused, practical improvements.

## Tasks for This Session (5 hours total)

### Task D.0: Create Unified HTTP Transport (4 hours) - FINAL
**File**: `plans/transport-type-architecture/tasks/D.0-unified-http-transport.md`

**Critical Decision: Use Hyper, Not Reqwest**
After thorough analysis, we must use hyper everywhere because:
- Reqwest has SSE buffering issues (why reverse proxy switched to hyper)
- Same issues would affect forward proxy if we used reqwest
- Hyper provides true streaming control and proxy transparency
- See analysis/reqwest-vs-hyper-decision.md for details

**Implementation Strategy**:
- Single HTTP transport handles JSON, SSE, and passthrough
- Content-Type header determines response handling
- True proxy transparency for unknown content types
- Consolidates 3 implementations into 1 (~500 lines saved)

### Task D.1: Document Architecture Decisions (1 hour)
**File**: `plans/transport-type-architecture/tasks/D.1-document-architecture-decisions.md`

Document why forward and reverse proxies have different architectures and provide guidelines to prevent future over-engineering:
- Create ADR (Architecture Decision Record)
- Update developer guide
- Document lessons learned
- Provide implementation guidelines

## Important Context

### Why NOT Full Unification
Based on Phase D re-evaluation (`analysis/phase-d-re-evaluation.md`):
- Forward and reverse proxies solve different problems
- Forcing unification would be over-engineering
- Phase C taught us that simple is better than "elegant"
- Different architectures reflect real requirements

### Key Files to Reference
- Tracker: `plans/transport-type-architecture/transport-type-architecture-tracker.md`
- Re-evaluation: `plans/transport-type-architecture/analysis/phase-d-re-evaluation.md`
- **HTTP Consolidation**: `plans/transport-type-architecture/analysis/http-client-consolidation.md`
- **HTTP/Streamable Analysis**: `plans/transport-type-architecture/analysis/http-streamable-consolidation.md`
- Current implementations: `src/transport/raw/http.rs`, `sse.rs`, `streamable_http.rs`
- Reverse proxy: `src/proxy/reverse/legacy.rs`

## Success Criteria
- [ ] Unused http_client.rs deleted
- [ ] Single unified HTTP transport handles JSON and SSE
- [ ] Content-Type auto-detection works
- [ ] SSE streaming works through OutgoingTransport trait
- [ ] Redundant sse.rs and streamable_http.rs removed
- [ ] Consistent snake_case.rs naming
- [ ] All tests passing
- [ ] ~500 lines of redundant code eliminated

## Commands to Run
```bash
# Navigate to shadowcat
cd shadowcat

# DELETE unused HTTP client first!
rm src/transport/http_client.rs

# Back up files before major changes
cp src/transport/raw/http.rs src/transport/raw/http.rs.backup
cp src/transport/raw/streamable_http.rs src/transport/raw/streamable_http.rs.backup

# After implementation, clean up redundant files
rm src/transport/raw/sse.rs
rm src/transport/raw/streamable_http.rs.backup

# Run tests after implementation
cargo test transport::raw::http
cargo test transport::directional::outgoing
cargo test reverse_proxy

# Verify no regressions
cargo test --lib
```

## Notes
- Keep it simple - we learned from Phase C that over-abstraction is harmful
- Focus on practical value, not architectural purity
- This is the final phase of the transport refactor
- After completion, the transport architecture work is DONE