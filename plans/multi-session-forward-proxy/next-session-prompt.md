# Next Session Prompt - Multi-Session Forward Proxy

## Current Status
We've identified that the forward proxy only handles one client-server connection at a time, which is a significant limitation. A plan has been created to enhance it to support multiple concurrent sessions.

## Key Discoveries
1. **Forward Proxy Limitations**:
   - Single `session_id` field
   - Accepts one connection then blocks
   - No concurrent client handling
   - Exits after one session completes

2. **Reverse Proxy Differences**:
   - Already handles multiple sessions
   - HTTP request-based model
   - Dynamic session extraction from headers
   - Load balancing capable

3. **Related Work**:
   - Reverse proxy session mapping plan exists
   - Transport layer can remain mostly unchanged
   - Session management patterns already exist

## Next Session Focus
Begin Phase A (Research & Analysis) - 2-3 hours:

### Primary Tasks
1. **Task A.0**: Analyze current implementation (`tasks/A.0-analyze-current-implementation.md`)
   - Review ForwardProxy structure
   - Identify single-session assumptions
   - Document in `analysis/current-forward-proxy.md`

2. **Task A.1**: Research connection pooling (`tasks/A.1-research-connection-pooling.md`)
   - Determine pooling feasibility
   - Security implications
   - Create `analysis/connection-pooling-strategy.md`

3. **Task A.2**: Design architecture (`tasks/A.2-design-multi-session-architecture.md`)
   - Session registry design
   - Resource management
   - Document in `analysis/multi-session-architecture.md`

### Key Context
- Forward proxy uses `IncomingTransport` and `OutgoingTransport` traits
- Currently in `src/proxy/forward.rs`
- CLI interface in `src/cli/forward.rs`
- Session manager already exists and could be reused

### Success Criteria for Next Session
- [ ] Complete understanding of current limitations
- [ ] Connection pooling strategy decided
- [ ] Multi-session architecture designed
- [ ] Ready for Phase B implementation

## Related Plans
- Reverse Proxy Session Mapping: `../reverse-proxy-session-mapping/`
- Both plans may share some infrastructure

## Commands to Start
```bash
cd /Users/kevin/src/tapwire/shadowcat
# Review the plan
cat ../plans/multi-session-forward-proxy/multi-session-forward-proxy-tracker.md
# Start with task A.0
cat ../plans/multi-session-forward-proxy/tasks/A.0-analyze-current-implementation.md
# Review current forward proxy
cat src/proxy/forward.rs
```