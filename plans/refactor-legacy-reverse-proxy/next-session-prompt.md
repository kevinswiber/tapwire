# Next Session: Implementation Phase B - Core Extraction

## Project Context

Refactoring the monolithic 3,682-line `legacy.rs` reverse proxy into clean modules. Phase A (Analysis & Design) is COMPLETE with refined architecture that addresses naming conflicts, leverages transport module, and removes admin UI.

**Project**: Refactor Legacy Reverse Proxy
**Tracker**: `plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`
**Status**: Phase A Complete - Ready for Phase B Implementation

## Current Status

### What Has Been Completed
- **Phase A - Analysis & Design** (✅ Completed 2025-01-18)
  - Analyzed 3,682-line legacy.rs structure
  - Designed clean architecture with 14 modules, all <300 lines
  - Refined plan to avoid conflicts and reuse transport
  - Created [Final Architecture](analysis/final-architecture.md)
  - Identified [Transport Overlap](analysis/transport-overlap-analysis.md) for analysis

### Key Architecture Decisions
- **Remove admin UI entirely** (~900 lines to delete)
- **Rename to avoid conflicts**: upstream/ instead of transport/, session_helpers.rs instead of session/
- **Leverage transport::sse** instead of reimplementing
- **Thin handlers** (<150 lines) that only orchestrate
- **Pipeline pattern** for cross-cutting concerns

## Your Mission

Begin Phase B implementation following the final architecture plan.

### Priority 0: Quick Analysis & Admin Removal (1 hour)

1. **Transport Overlap Analysis** (30 min)
   - Review transport module capabilities
   - Identify what proxy::reverse can reuse
   - Note what might move to transport later
   - Document findings in transport-overlap-analysis.md

2. **Remove Admin UI** (30 min)
   - Delete handle_admin_request function
   - Remove admin routes from router
   - Delete admin-related tests
   - Verify remaining tests pass

### Priority 1: Foundation Modules (3 hours)

3. **Extract Error Types** (30 min)
   - Create `src/proxy/reverse/error.rs`
   - Move ReverseProxyError enum and IntoResponse impl
   - Add re-export in legacy.rs

4. **Extract Config Types** (1 hour)
   - Create `src/proxy/reverse/config.rs`
   - Move all config structs (UpstreamConfig, LoadBalancingStrategy, etc.)
   - Include Default impls

5. **Extract Metrics & State** (30 min)
   - Create `src/proxy/reverse/metrics.rs` - ReverseProxyMetrics
   - Create `src/proxy/reverse/state.rs` - AppState

6. **Extract Helper Modules** (1 hour)
   - Create `src/proxy/reverse/headers.rs` - Header utilities
   - Create `src/proxy/reverse/session_helpers.rs` - Session operations
   - Create `src/proxy/reverse/pipeline.rs` - Intercept/pause/record

### Priority 2: Upstream Abstraction (if time permits)

7. **Create UpstreamService Trait** (30 min)
   - Create `src/proxy/reverse/upstream/mod.rs`
   - Define trait and response types

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Commands to Run First

```bash
# Create or checkout refactor branch
git checkout -b refactor/legacy-reverse-proxy || git checkout refactor/legacy-reverse-proxy

# Baseline tests (note the count)
cargo test proxy::reverse --lib | grep "test result"
# Should show: "test result: ok. 20 passed"

# Check transport module structure for reuse opportunities
ls -la src/transport/
grep -r "pub struct.*Transport" src/transport/
grep -r "impl.*Transport" src/transport/

# Count admin-related lines to remove
grep -n "admin" src/proxy/reverse/legacy.rs | wc -l

# Create module structure
mkdir -p src/proxy/reverse/{handlers,upstream/http}
```

## Implementation Strategy

### For Admin Removal:
```rust
// Find and delete:
// - handle_admin_request function
// - route("/admin", ...) in router
// - Any admin HTML generation
// - Admin-specific imports

// After deletion, run:
cargo test proxy::reverse --lib
// Some tests might fail if they were admin-related - remove those too
```

### For Each Module Extraction:
```rust
// 1. Create new file
// 2. Move types/functions with their imports
// 3. In legacy.rs, add:
mod error;  // or whatever module
use error::*;  // temporary compatibility

// 4. Test immediately
cargo test proxy::reverse --lib

// 5. Fix any compilation errors
// 6. Run clippy
cargo clippy --all-targets -- -D warnings

// 7. Commit if green
git add -A && git commit -m "refactor: extract {module} from legacy.rs"
```

## Success Criteria Checklist

- [ ] Admin UI completely removed (~900 lines gone)
- [ ] Transport overlap documented
- [ ] Error module created (<50 lines)
- [ ] Config module created (<250 lines)
- [ ] Metrics module created (<50 lines)
- [ ] State module created (<100 lines)
- [ ] Helper modules created (headers, session_helpers, pipeline)
- [ ] All non-admin tests still passing
- [ ] No clippy warnings
- [ ] Each extraction committed separately

## Key Architecture Points to Remember

1. **upstream/ not transport/** - Avoid naming conflict
2. **session_helpers.rs not session/** - Single file, not a module
3. **relay.rs not forward.rs** - Clearer naming in reverse proxy context
4. **Reuse transport::sse** - Don't reimplement SSE parsing
5. **Thin handlers** - Move logic to pipeline.rs and upstream/

## Example Extraction Pattern

```rust
// src/proxy/reverse/pipeline.rs
pub mod intercept {
    use crate::interceptor::{InterceptAction, InterceptContext};
    
    pub async fn apply_inbound(
        app: &AppState,
        msg: Value,
    ) -> Result<Value> {
        // Move interception logic here
    }
}

pub mod record {
    use crate::recorder::TapeRecorder;
    
    pub async fn record_request(
        recorder: Option<&Arc<TapeRecorder>>,
        envelope: MessageEnvelope,
    ) -> Result<()> {
        // Move recording logic here
    }
}
```

## Next Steps After This Session

Once foundation modules are extracted:
- **Phase C**: Create upstream implementations
- **Phase D**: Thin out handlers to <150 lines
- **Phase E**: Final cleanup and legacy.rs deletion

## Session Time Management

**Estimated Session Duration**: 4-5 hours
- Transport analysis: 30 min
- Admin removal: 30 min
- Type extractions: 2 hours
- Helper modules: 1 hour
- Testing & validation: 30 min
- Documentation: 30 min

---

**Session Goal**: Remove admin UI and extract foundation modules while maintaining test coverage

**Last Updated**: 2025-01-18
**Next Review**: After Priority 1 tasks complete