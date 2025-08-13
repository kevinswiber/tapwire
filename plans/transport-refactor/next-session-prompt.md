# Next Session: Phase 0 - Prerequisites and Analysis

## Project Overview

We're refactoring Shadowcat's transport layer to introduce clearer `IncomingTransport` and `OutgoingTransport` abstractions. This addresses architectural confusion where:
- `StdioTransport` spawns subprocesses (actually outgoing)
- `StdioClientTransport` reads stdin (actually incoming)  
- HTTP and SSE are artificially separated when MCP uses both together
- Transport mechanics are mixed with protocol semantics

## Project Context

We're refactoring Shadowcat's transport layer to introduce clearer abstractions and enable proper Streamable HTTP support.

**Project**: Transport Layer Refactor
**Tracker**: `plans/transport-refactor/transport-refactor-tracker.md`
**Status**: Phase 0 - Prerequisites and Analysis (0% Complete)

## Key Design Decisions

1. **IncomingTransport**: Transports the proxy exposes (accept connections)
   - StdioIncoming (read from stdin)
   - HttpServerIncoming (HTTP server)
   - StreamableHttpIncoming (HTTP server + SSE responses)

2. **OutgoingTransport**: Transports that connect to upstream targets
   - SubprocessOutgoing (spawn subprocess)
   - HttpClientOutgoing (HTTP client)
   - StreamableHttpOutgoing (HTTP POST + SSE client)

3. **Layer Separation**:
   - RawTransport: Handles bytes only
   - ProtocolHandler: Handles MCP/JSON-RPC
   - Direction-aware transports: Combine the above

## Current Status

### What Has Been Completed
- **SSE/MCP Integration** (✅ Completed 2025-08-13)
  - All 8 phases complete with 163+ tests
  - Performance targets met
  - Production ready

### What's In Progress
- **A.1: Document existing transport patterns** (Not Started)
  - Duration: 3 hours
  - Dependencies: None

## Your Mission

Analyze and document the current transport architecture to prepare for safe refactoring.

### Priority 1: Analysis Tasks (9 hours total)

1. **A.1: Document existing transport patterns** (3h)
   - Map all current transport implementations
   - Identify usage patterns in forward/reverse proxy
   - Document coupling points with protocol handling
   
2. **A.2: Create test suite for current behavior** (4h)
   - Comprehensive tests for existing transports
   - Performance benchmarks
   - Regression test suite
   
3. **A.3: Identify breaking change risks** (2h)
   - API compatibility assessment
   - Migration strategy planning
   - Risk mitigation approach

## Essential Context Files to Read

1. **Primary Tracker**: `plans/transport-refactor/transport-refactor-tracker.md` - Full project context
2. **Task Details**: `plans/transport-refactor/tasks/F.3-incoming-outgoing-traits.md` - Example task
3. **Current Transports**: `shadowcat/src/transport/*.rs` - Existing implementations
4. **MessageEnvelope**: `shadowcat/src/transport/envelope.rs` - Recent refactor result

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Commands to Run First

```bash
# List current transport implementations
ls -la src/transport/*.rs

# Check test coverage
cargo test transport:: --quiet

# Review transport usage in proxies
rg "impl.*Transport" src/proxy/
```

## Implementation Strategy

### Phase 1: Document Current State (3h)
1. List all transport types and their purposes
2. Map transport usage in forward/reverse proxy
3. Identify protocol coupling points
4. Create architecture diagram

### Phase 2: Create Test Suite (4h)
1. Unit tests for each transport
2. Integration tests for proxy flows
3. Performance benchmarks
4. Edge case coverage

### Phase 3: Risk Assessment (2h)
1. Identify all breaking changes
2. Plan migration strategy
3. Create compatibility shims if needed

## Success Criteria Checklist

- [ ] All transport patterns documented
- [ ] Comprehensive test suite created
- [ ] Risk assessment complete
- [ ] Migration strategy defined
- [ ] Tracker updated with findings
- [ ] Ready to proceed with Phase 1

## Important Notes

- **Always use TodoWrite tool** to track progress through tasks
- **DO NOT** modify existing transports in Phase 0
- **Document everything** - this analysis guides the entire refactor
- **Test thoroughly** - regression suite is critical

## Key Design Considerations

1. **Backward Compatibility**: Ensure existing code continues to work during migration
2. **Performance**: Maintain or improve current performance metrics
3. **Clarity**: New abstractions should be immediately understandable
4. **Testability**: Design for easy mocking and testing

## Next Steps After This Task

Once Phase 0 analysis is complete:
- **F.1-F.5**: Foundation tasks - Design new trait hierarchy (11h)
- **R.1-R.5**: Raw Transport Layer implementation (16h)

After completing Phase 0:
- Move to Phase 1 (Foundation) with approved design

---

**Session Goal**: Complete comprehensive analysis of current transport architecture to enable safe, effective refactoring.

**Last Updated**: 2025-08-13
**Next Review**: After Phase 0 completion