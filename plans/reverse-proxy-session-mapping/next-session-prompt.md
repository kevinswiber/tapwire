# Next Session Prompt - Reverse Proxy Session Mapping

## Current Status
We've identified that the reverse proxy needs to maintain its own session IDs separate from upstream servers to properly handle SSE reconnection, connection pooling, and failover scenarios.

## Work Completed
1. Fixed immediate issue with MCP Inspector by making session IDs optional for initialize requests
2. Created comprehensive plan for session mapping architecture
3. Set up task structure for implementation

## Next Session Focus
Begin Phase A (Research & Analysis) - 2-3 hours of work:

### Primary Tasks
1. **Task A.0**: Analyze current session management (`tasks/A.0-analyze-session-management.md`)
   - Review session components in `src/session/`
   - Understand current session lifecycle
   - Document findings in `analysis/current-session-architecture.md`

2. **Task A.1**: Map session ID usage (`tasks/A.1-map-session-id-usage.md`)
   - Search codebase for all SessionId usage
   - Create usage matrix
   - Document in `analysis/session-id-usage-map.md`

3. **Task A.2**: Design mapping architecture (`tasks/A.2-design-mapping-architecture.md`)
   - Design data structures
   - Plan SSE reconnection flow
   - Create `analysis/session-mapping-design.md`

### Key Context
- The reverse proxy currently doesn't distinguish between its own sessions and upstream sessions
- MCP spec says servers assign session IDs during initialization
- We need to track Last-Event-Id for SSE reconnection
- Must maintain backward compatibility

### References
- Implementation tracker: `reverse-proxy-session-mapping-tracker.md`
- MCP spec: `/specs/mcp/docs/specification/2025-03-26/basic/transports.mdx`
- Current fix: Git commit 16a8e8d

### Success Criteria for Next Session
- [ ] Complete understanding of current session architecture
- [ ] Full inventory of session ID usage points
- [ ] Detailed design for dual session tracking
- [ ] Ready to begin Phase B implementation

## Commands to Start
```bash
cd /Users/kevin/src/tapwire/shadowcat
# Review the plan
cat ../plans/reverse-proxy-session-mapping/reverse-proxy-session-mapping-tracker.md
# Start with task A.0
cat ../plans/reverse-proxy-session-mapping/tasks/A.0-analyze-session-management.md
```