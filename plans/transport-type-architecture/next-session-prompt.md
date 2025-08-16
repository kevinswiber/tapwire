# Next Session: Phase B - Quick Fix Implementation

## Project Context

We've completed the comprehensive Phase A analysis and architecture design for the transport type refactor. The analysis revealed that `is_sse_session` is completely dead code (never set), and we've designed a clean architecture to eliminate duplication and establish proper domain boundaries.

**Project**: Transport Type Architecture Refactor
**Tracker**: `plans/transport-type-architecture/transport-type-architecture-tracker.md`
**Status**: Phase A Complete - Ready for Phase B Implementation

## What Has Been Completed

### Phase A - Deep Analysis (✅ 10 hours completed)

All analysis tasks completed with comprehensive documentation:

1. **Transport Usage Audit** - Mapped all 174 TransportType usages
2. **Directional Transport Analysis** - Analyzed trait architecture opportunities
3. **Response Mode Investigation** - Discovered is_sse_session is dead code
4. **Architecture Proposal** - Created comprehensive solution design with:
   - Clean component architecture
   - ResponseMode enum design
   - Unified transport abstractions
   - Three-phase implementation plan
   - Complete API designs

### Key Decisions Made

1. **ResponseMode Enum**: Separate from TransportType for orthogonal concerns
2. **DirectionalTransports**: Adopt for reverse proxy to eliminate duplication
3. **Phased Approach**: Three phases for incremental, safe migration
4. **Module Structure**: Organize by technical layer (transport/protocol/proxy)

## Your Mission

Implement Phase B - the quick fix to eliminate the `is_sse_session` code smell and introduce proper response mode tracking.

### Phase B Tasks (4-6 hours total)

1. **B.0: Add ResponseMode Enum** (1 hour)
   - Create `src/transport/core/response_mode.rs`
   - Add detection methods from Content-Type
   - Export from transport module
   - Add comprehensive tests

2. **B.1: Update Session Structure** (2 hours)
   - Remove `is_sse_session` field
   - Add `response_mode: Option<ResponseMode>`
   - Update session creation and methods
   - Update all session tests

3. **B.2: Migrate Usage Sites** (2 hours)
   - Update hyper_client.rs response detection
   - Update legacy.rs response routing
   - Update SSE resilience module
   - Remove all is_sse_session references

4. **B.3: Test and Validate** (1 hour)
   - Run full test suite
   - Verify no regressions
   - Check performance impact
   - Update documentation

## Essential Files to Reference

### Design Documents
- `analysis/architecture-proposal.md` - Complete architecture design
- `analysis/implementation-roadmap.md` - Detailed step-by-step guide
- `analysis/design-decisions.md` - Rationale for choices

### Task Details
- `tasks/B.0-add-response-mode.md` - ResponseMode implementation
- `tasks/B.1-update-session-structure.md` - Session changes
- `tasks/B.2-migrate-usage-sites.md` - Migration steps
- `tasks/B.3-test-validate.md` - Validation requirements

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
git checkout refactor/transport-type-architecture
```

## Implementation Checklist

### B.0: ResponseMode Enum
- [ ] Create response_mode.rs with enum definition
- [ ] Add from_content_type() detection method
- [ ] Add is_streaming() helper method
- [ ] Export from transport/core/mod.rs
- [ ] Add unit tests for detection logic

### B.1: Session Updates
- [ ] Remove is_sse_session field from Session struct
- [ ] Add response_mode field
- [ ] Add client_capabilities field (using bitflags)
- [ ] Remove mark_as_sse_session() method
- [ ] Remove is_sse() method
- [ ] Add set_response_mode() method
- [ ] Add is_streaming() method
- [ ] Update Session::new() constructor
- [ ] Add bitflags dependency to Cargo.toml

### B.2: Usage Migration
- [ ] Update HyperResponse to use ResponseMode only (no is_sse() compatibility)
- [ ] Update process_via_http_hyper routing logic
- [ ] Update SSE resilience checks
- [ ] Search and remove all is_sse_session references
- [ ] Remove HyperResponse::is_sse() completely (no backwards compatibility needed)
- [ ] Update test fixtures

### B.3: Validation
- [ ] Run `cargo build` - must compile
- [ ] Run `cargo test` - all tests must pass
- [ ] Run `cargo clippy` - no warnings
- [ ] Run benchmarks - verify <5% overhead
- [ ] Update CHANGELOG.md

## Code Snippets to Use

### ResponseMode Enum
```rust
use mime::Mime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseMode {
    Json,       // Standard JSON-RPC
    SseStream,  // Server-Sent Events
    Passthrough,// Any other content type - stream without processing
}

impl ResponseMode {
    pub fn from_content_type(content_type: &str) -> Self {
        match content_type.parse::<Mime>() {
            Ok(mime) => match (mime.type_(), mime.subtype()) {
                (mime::APPLICATION, mime::JSON) => Self::Json,
                (mime::TEXT, subtype) if subtype == "event-stream" => Self::SseStream,
                _ => Self::Passthrough,
            },
            Err(_) => Self::Passthrough,
        }
    }
}
```

### Session Updates
```rust
pub struct Session {
    pub id: SessionId,
    pub transport_type: TransportType,
    pub response_mode: Option<ResponseMode>, // New
    pub supports_streaming: bool, // New
    pub upstream_session_id: Option<SessionId>, // For reverse proxy session mapping
    // Remove: is_sse_session
}
```

### Response Detection
```rust
// Use proper MIME parsing
let response_mode = ResponseMode::from_content_type(
    hyper_response.content_type().unwrap_or("")
);

match response_mode {
    ResponseMode::SseStream => forward_sse_stream(...),
    ResponseMode::Json => handle_json_response(...),
    ResponseMode::Passthrough => forward_raw_response(...), // Stream without buffering
}
```

## Success Criteria

- ✅ No instances of `is_sse_session` remain in codebase
- ✅ ResponseMode enum properly tracks response formats
- ✅ All existing tests continue to pass
- ✅ No new clippy warnings introduced
- ✅ Performance overhead <5%
- ✅ Clean git history with atomic commits

## Important Notes

- This is a quick fix phase - don't over-engineer
- Keep changes focused on ResponseMode introduction
- Save larger refactoring for Phase C
- Test thoroughly - this affects core session handling
- Use atomic commits for easy rollback if needed

## Risks to Watch

1. **Session Storage**: Ensure SQLite schema updates if persisted
2. **Test Fixtures**: Many tests create sessions - update carefully
3. **SSE Resilience**: Module depends on session state - test thoroughly
4. **Performance**: Response mode detection on hot path - optimize

## Next Steps After Phase B

Once B.3 validation is complete:
1. Tag the commit: `git tag phase-b-complete`
2. Update tracker to show Phase B completion
3. Prepare for Phase C (Transport Consolidation)

---

**Session Goal**: Complete Phase B implementation with ResponseMode enum
**Estimated Duration**: 4-6 hours
**Last Updated**: 2025-08-16
**Critical**: Focus on clean, working implementation - don't expand scope