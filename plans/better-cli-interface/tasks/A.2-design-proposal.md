# Task A.2: Design Proposal

## Objective

Create a comprehensive design proposal for the new Shadowcat CLI interface based on analysis and research findings. This design will guide the implementation of the improved developer experience.

## Background

Based on the current CLI analysis (A.0) and UX research (A.1), we need to design a new interface that:
- Makes common cases magical (just work)
- Provides explicit control when needed
- Maintains backward compatibility
- Scales to future transport types

## Key Questions to Answer

1. What should the primary command structure be?
2. How should auto-detection work (specific heuristics)?
3. What commands need explicit versions?
4. How do we handle ambiguous cases?
5. What's the migration path for existing users?

## Step-by-Step Process

### 1. Design Core Commands (45 min)
Define the new command structure

```bash
# Primary magical commands
shadowcat my-server          # Auto-detect forward proxy
shadowcat :8080             # Auto-detect gateway mode
shadowcat session.tape      # Auto-detect replay

# Explicit commands
shadowcat forward [options]  # Forward proxy (client → shadowcat → server)
shadowcat gateway [options]  # API gateway (client → shadowcat → backends)
shadowcat record [options]   # Record session
shadowcat replay [options]   # Replay session
```

### 2. Define Auto-detection Logic (60 min)

Create clear heuristics:
- Executable file → forward proxy (stdio)
- Port notation (:8080) → gateway mode
- .tape file → replay
- URL → forward proxy (http)
- Special cases and ambiguity handling

### 3. Design Help System (45 min)

Structure help text for discoverability:
- Main help shows common workflows
- Subcommand help shows details
- Examples for each use case
- Progressive disclosure

### 4. Plan Migration Strategy (30 min)

Ensure smooth transition:
- Replace "reverse" with "gateway"
- Clear error messages for old commands
- Update all documentation
- Help users understand the new mental model

## Expected Deliverables

### Design Document
- `analysis/cli-design-proposal.md` - Complete design specification
- Includes:
  - Command hierarchy
  - Auto-detection flowchart
  - Help text mockups
  - Migration plan
  - Example usage scenarios

### Implementation Specifications
- Exact command structures
- Flag and option definitions
- Error message templates
- Help text templates

## Success Criteria Checklist

- [ ] Primary command structure defined
- [ ] Auto-detection logic specified
- [ ] Help system designed
- [ ] Migration path planned
- [ ] Edge cases considered
- [ ] Design document complete
- [ ] Examples for all use cases

## Risk Assessment

| Risk | Impact | Mitigation | 
|------|--------|------------|
| Auto-detection too magical | HIGH | Provide --mode override |
| Breaking changes | HIGH | Maintain aliases |
| Ambiguous inputs | MEDIUM | Clear error messages |
| Complex implementation | MEDIUM | Start with simple heuristics |

## Duration Estimate

**Total: 3 hours**
- Command design: 45 minutes
- Auto-detection logic: 60 minutes
- Help system design: 45 minutes
- Migration planning: 30 minutes

## Dependencies

- A.0: Current CLI Analysis (understand constraints)
- A.1: User Experience Research (apply patterns)

## Integration Points

- **Implementation (B.x)**: Will build this design
- **Testing (C.1)**: Will validate this design
- **Documentation (C.3)**: Will document this design

## Notes

- Keep auto-detection simple and predictable
- Prioritize common workflows
- Consider future transport types (WebSocket, gRPC)
- Think about how this works with interceptors and policies

## Design Examples

### Auto-detection Flowchart
```
Input Analysis:
  ├─ Starts with ':' or contains port? → Gateway Mode
  ├─ Ends with '.tape' or '.json'? → Replay Mode
  ├─ Starts with 'http://' or 'https://'? → Forward Proxy (HTTP)
  ├─ Contains '/' or is executable? → Forward Proxy (Stdio)
  └─ Else → Show helpful error with suggestions
```

### Command Structure
```
shadowcat
  ├─ [TARGET] (magical auto-detect)
  ├─ forward (explicit forward proxy)
  │   ├─ --transport {stdio|http|sse}
  │   ├─ --record
  │   └─ [options]
  ├─ gateway (API gateway mode)
  │   ├─ --port
  │   ├─ --upstream
  │   └─ [options]
  ├─ record
  │   ├─ --output
  │   └─ -- [command]
  └─ replay
      ├─ --port
      └─ [tape-file]
```

### Help Text Template
```
Shadowcat - MCP Developer Proxy

USAGE:
    shadowcat [TARGET]              Auto-detect and run
    shadowcat forward [OPTIONS]     Forward proxy to MCP server
    shadowcat gateway [OPTIONS]     API gateway for MCP clients
    shadowcat record -- [COMMAND]   Record MCP session
    shadowcat replay [TAPE]         Replay recorded session

COMMON EXAMPLES:
    shadowcat my-mcp-server         Forward proxy to local command
    shadowcat :8080                  Gateway listening on port 8080
    shadowcat http://api.example    Forward proxy to HTTP endpoint
    shadowcat session.tape           Replay a recording

For more information, try:
    shadowcat help [COMMAND]
    shadowcat --help
```

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-14
**Last Modified**: 2025-01-14
**Author**: Kevin