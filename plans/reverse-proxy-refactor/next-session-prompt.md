# Next Session: Phase A - Analysis & Architecture

## Project Context

The Shadowcat reverse proxy has grown to 3400+ lines in a single file and has critical issues with SSE streaming that cause client timeouts. This refactor will modularize the code, fix SSE handling, and implement proper session mapping for production readiness.

**Project**: Reverse Proxy Refactor
**Tracker**: `plans/reverse-proxy-refactor/tracker.md`
**Status**: Phase A - Analysis & Architecture (0% Complete)

## Current Status

### What Has Been Completed
- **Planning**: Comprehensive refactor plan created (✅ Completed 2025-01-15)
  - 5-phase implementation strategy defined
  - Task breakdown with time estimates
  - Reference implementations identified

### What's In Progress
- **A.0: Code Analysis** (Not Started)
  - Duration: 2 hours
  - Dependencies: None

## Your Mission

Perform deep analysis of the current reverse proxy implementation to understand its structure, dependencies, and pain points. This analysis will inform the refactoring strategy and ensure we don't break existing functionality.

### Priority 1: Code Analysis (2 hours)

1. **Function and Structure Mapping** (45 min)
   - Map all functions in `src/proxy/reverse.rs`
   - Document function dependencies
   - Identify logical sections and boundaries
   
2. **State and Concurrency Analysis** (30 min)
   - Document shared state (AppState, SessionManager)
   - Map synchronization points
   - Identify potential race conditions
   
3. **SSE Flow Analysis** (30 min)
   - Trace SSE request/response flow
   - Identify buffering points causing timeouts
   - Compare with reference implementations

4. **Problem Documentation** (15 min)
   - List all TODO/FIXME comments
   - Document known bugs
   - Identify code smells and duplication

### Priority 2: SSE Infrastructure Review (1.5 hours if time permits)
Review existing SSE modules and reference implementations to identify reusable components.

## Essential Context Files to Read

1. **Primary Tracker**: `plans/reverse-proxy-refactor/tracker.md` - Full project context
2. **Task Details**: `plans/reverse-proxy-refactor/tasks/A.0-code-analysis.md` - Analysis specifications
3. **Implementation**: `shadowcat/src/proxy/reverse.rs` - The 3400+ line file to analyze
4. **SSE Modules**: `shadowcat/src/transport/sse/` - Existing SSE infrastructure
5. **Current Issues**: `shadowcat/SSE_STATUS.md` - Known problems and attempted fixes

## Working Directory

```bash
cd ~/src/tapwire/shadowcat
```

## Commands to Run First

```bash
# Check current file size and structure
wc -l src/proxy/reverse.rs
grep -n "^pub fn\|^async fn\|^fn " src/proxy/reverse.rs | head -20

# Find SSE-related code
rg "SSE|event-stream|EventSource" src/proxy/reverse.rs

# Check for TODOs
rg "TODO|FIXME|HACK" src/proxy/reverse.rs

# Look for large functions
awk '/^(async )?fn / {name=$0; count=0} {count++} /^}$/ {if(count>50) print name " - " count " lines"}' src/proxy/reverse.rs
```

## Implementation Strategy

### Phase 1: Setup & Initial Review (15 min)
1. Open `src/proxy/reverse.rs` and get familiar with structure
2. Create `analysis/` directory for findings
3. Set up analysis document templates

### Phase 2: Deep Analysis (1.5 hours)
1. Map all public functions and their responsibilities
2. Trace request flow for both JSON and SSE
3. Document shared state and synchronization
4. Identify module boundaries for refactoring

### Phase 3: Reference Comparison (30 min)
1. Review Inspector SSE implementation at `~/src/modelcontextprotocol/inspector/src/client/sse.ts`
2. Check TypeScript SDK at `~/src/modelcontextprotocol/typescript-sdk/src/transports/sse.ts`
3. Note patterns we should adopt

### Phase 4: Documentation (15 min)
1. Write findings to `analysis/current-architecture.md`
2. Document dependencies in `analysis/dependencies.md`
3. Update tracker with completion status

## Success Criteria Checklist

- [ ] Complete function map with call graph
- [ ] SSE flow documented with bottlenecks identified
- [ ] Shared state and synchronization documented
- [ ] Module boundaries proposed
- [ ] Reference implementation patterns noted
- [ ] Analysis documents created in `analysis/` directory
- [ ] Tracker updated with findings

## Key Commands

```bash
# Analysis commands
rg "^(pub |async |fn )" src/proxy/reverse.rs
rg "struct|enum" src/proxy/reverse.rs
rg "Arc<|Mutex<|RwLock<" src/proxy/reverse.rs

# Check existing SSE modules
ls -la src/transport/sse/
rg "pub struct|pub trait" src/transport/sse/

# Reference implementations
cat ~/src/modelcontextprotocol/inspector/src/client/sse.ts
cat ~/src/modelcontextprotocol/typescript-sdk/src/transports/sse.ts
```

## Important Notes

- **Focus on understanding, not judging** - Document what exists objectively
- **Pay special attention to SSE handling** - This is the critical bug to fix
- **Note integration points** - We need to maintain compatibility
- **Look for reusable patterns** - Existing SSE modules might solve our problems
- **Document assumptions** - Implicit behaviors that aren't obvious

## Key Design Considerations

1. **Streaming vs Buffering**: Current code tries to buffer SSE streams, which is fundamentally wrong
2. **Session Mapping**: Need to support proxy-managed sessions separate from upstream
3. **Module Size**: 3400 lines is too large - need logical boundaries for ~500 line modules

## Risk Factors & Blockers

- **Breaking Changes**: Must maintain backward compatibility
- **Performance**: Can't exceed 5% latency overhead requirement
- **Complexity**: SSE streaming with interceptors is non-trivial

## Next Steps After This Task

Once analysis is complete:
- **A.1**: SSE Infrastructure Review (1.5 hours)
- **A.2**: Design New Architecture (2 hours)
- **A.3**: Migration Strategy (1 hour)

After Phase A completion:
- Move to Phase B - Modularization (6-8 hours of refactoring)

## Model Usage Guidelines

- **IMPORTANT**: This is primarily an analysis task - avoid making code changes
- When context window has less than 15% availability, save findings and create new session

## Session Time Management

**Estimated Session Duration**: 2-3.5 hours
- Setup & Context: 15 min
- Analysis: 2 hours
- Documentation: 30 min
- Optional SSE Review: 1.5 hours

## Related Context

- **Integration Points**: Session manager, interceptor chain, transport layer
- **Downstream Dependencies**: CLI commands, integration tests
- **Parallel Work**: Session mapping plan in `plans/reverse-proxy-session-mapping/`

---

**Session Goal**: Complete thorough analysis of `reverse.rs` to understand current architecture and identify refactoring boundaries

**Last Updated**: 2025-01-15
**Next Review**: After A.0 completion