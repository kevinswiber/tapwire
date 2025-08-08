# Task A.0: Analyze MCP Protocol Specifications

**Duration**: 2 hours  
**Dependencies**: None  
**Status**: ⬜ Not Started  

## Objective

Understand the proper protocol layering by studying the MCP specifications to ensure our refactor aligns with the actual protocol design. This is critical to properly separate transport concerns from protocol semantics.

## Key Questions to Answer

1. **Are notifications truly bidirectional in MCP?**
   - Can clients send notifications to servers?
   - Can servers send notifications to clients?
   - How is the direction determined?

2. **How does MCP handle message routing/direction?**
   - Is direction implicit in the message type?
   - Is it determined by the transport?
   - Are there explicit direction markers?

3. **What's the proper separation between JSON-RPC, MCP, and Transport layers?**
   - What belongs in each layer?
   - How do they interact?
   - Where should session management live?

4. **What metadata is required vs optional at each layer?**
   - Transport metadata (headers, event IDs)
   - MCP metadata (session, version)
   - JSON-RPC metadata (correlation IDs)

5. **How do different transports map to MCP semantics?**
   - HTTP request/response vs MCP request/response
   - SSE events vs MCP notifications
   - stdio streams vs bidirectional communication

## Specifications to Study

### MCP 2025-06-18 Specification (Primary)
- `specs/mcp/docs/specification/2025-06-18/protocol/index.mdx` - Protocol overview
- `specs/mcp/docs/specification/2025-06-18/transports/index.mdx` - Transport layer design
- `specs/mcp/docs/specification/2025-06-18/transports/http-sse.md` - HTTP/SSE specific requirements

### MCP 2025-03-26 Specification (Reference)
- `specs/mcp/docs/specification/2025-03-26/basic/architecture.mdx` - Architecture overview
- `specs/mcp/docs/specification/2025-03-26/basic/transports.mdx` - Transport requirements

### Implementation to Review
- `shadowcat/src/transport/mod.rs` - Current transport abstraction
- `shadowcat/src/transport/http_mcp.rs` - MCP-specific HTTP implementation
- `shadowcat/src/session/manager.rs` - Session management approach

## Deliverables

### 1. Protocol Layer Analysis Report
**Location**: `plans/transport-context-refactor/analysis/mcp-protocol-layers.md`

Structure:
```markdown
# MCP Protocol Layer Analysis

## Protocol Layers
1. Transport Layer
   - Purpose: ...
   - Responsibilities: ...
   - Examples: HTTP, SSE, stdio, WebSocket
   
2. MCP Protocol Layer
   - Purpose: ...
   - Responsibilities: ...
   - Message types: Request, Response, Notification
   
3. JSON-RPC Layer
   - Purpose: ...
   - Responsibilities: ...
   - Structure: id, method, params, result, error

## Notification Model
- Directionality: [Bidirectional/Unidirectional]
- Client→Server notifications: [Examples]
- Server→Client notifications: [Examples]
- Routing requirements: ...

## Metadata Requirements
### Transport Layer
- Required: ...
- Optional: ...

### MCP Layer
- Required: session_id, protocol_version
- Optional: ...

### JSON-RPC Layer
- Required: ...
- Optional: ...

## Transport Mapping
### HTTP Transport
- Requests map to: ...
- Responses map to: ...
- Notifications handled by: ...

### SSE Transport  
- Events map to: ...
- Notifications natural fit
- Request/Response challenges: ...

### stdio Transport
- Bidirectional stream
- Natural fit for all message types

## Key Findings
1. ...
2. ...
3. ...

## Refactor Implications
- TransportMessage should be renamed to McpMessage
- Notifications need direction field
- Transport metadata must be separate
- Session context should be at MCP layer
```

### 2. Architecture Clarification Document
**Location**: `plans/transport-context-refactor/analysis/architecture-clarification.md`

This should clarify:
- Where each piece of metadata belongs
- How layers should interact
- What abstractions are needed
- How to maintain protocol compliance

## Process

### Step 1: Read Protocol Specifications (45 min)
1. Start with 2025-06-18 protocol overview
2. Understand transport abstraction
3. Focus on HTTP/SSE specifics
4. Note bidirectional notification examples

### Step 2: Analyze Current Implementation (30 min)
1. Review TransportMessage structure
2. Identify what's conflated
3. Find missing metadata
4. Note workarounds

### Step 3: Document Findings (30 min)
1. Create protocol layer analysis
2. Map current issues to proper layers
3. Identify required changes
4. Document implications

### Step 4: Validate Understanding (15 min)
1. Cross-reference with existing code
2. Ensure consistency with tracker vision
3. Identify any gaps or conflicts

## Success Criteria

- [ ] All 5 key questions answered definitively
- [ ] Protocol layers clearly documented
- [ ] Notification model understood
- [ ] Transport mappings defined
- [ ] Refactor implications clear
- [ ] Architecture clarification complete

## Notes

- Focus on understanding, not implementation
- Document everything, even obvious things
- Pay special attention to notification handling
- Consider future transport types (WebSocket)
- Note any spec ambiguities or contradictions

## Commands

```bash
# Navigate to specs
cd /Users/kevin/src/tapwire/specs/mcp/docs/specification

# Search for notification patterns
rg "notification" --type md

# Find bidirectional examples
rg "client.*notification|server.*notification" --type md -A 2 -B 2

# Check session handling
rg "session|SessionId|session_id" --type md
```

## Output Examples

Look for patterns like:
- "The client MAY send notifications to the server"
- "The server sends notifications to the client"
- "Bidirectional communication"
- "Direction is determined by"

## Related Tasks

- **Next**: A.1 - Analyze TransportMessage Usage (depends on this)
- **Enables**: A.2 - Design MessageEnvelope Structure
- **Blocks**: All implementation phases

---

**Task Owner**: _Unassigned_  
**Created**: 2025-08-08  
**Last Updated**: 2025-08-08