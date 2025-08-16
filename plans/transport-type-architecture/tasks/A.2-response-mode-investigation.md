# Task A.2: Response Mode Investigation

## Objective

Investigate what the `is_sse_session` boolean is actually tracking and design a proper `ResponseMode` enum to replace it with clear, type-safe semantics.

## Background

The `is_sse_session` boolean appears to be tracking whether a session is currently streaming SSE responses vs returning JSON responses. This is actually about response mode, not transport type. A single StreamableHttp transport can return either JSON or SSE depending on the server's response to each request.

We need to understand:
- When and why `is_sse_session` is set
- What behavior changes based on this flag
- What other response modes might exist
- How to properly model this with an enum

## Key Questions to Answer

1. What triggers `is_sse_session` to be set to true?
2. What code paths behave differently when it's true vs false?
3. Are there other response modes besides JSON and SSE?
4. How does response mode relate to the Accept header?
5. Should response mode be per-session or per-request?

## Step-by-Step Process

### 1. Flag Usage Analysis (45 min)

Trace the lifecycle of `is_sse_session`:

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Find where it's set to true
rg "mark_as_sse_session|is_sse_session.*=.*true" --type rust -B 5 -A 5

# Find where it's checked
rg "if.*is_sse_session|is_sse\(\)" --type rust -B 3 -A 5

# Find related SSE detection
rg "text/event-stream|Accept.*sse" --type rust -B 3 -A 3

# Look for response type detection
rg "Content-Type|application/json" --type rust -B 2 -A 2
```

### 2. Response Pattern Analysis (30 min)

Understand different response patterns:

```bash
# Look for JSON response handling
rg "application/json|serde_json::from" src/proxy/reverse/ -B 2 -A 2

# Look for SSE response handling
rg "SseStream|EventStream|text/event-stream" src/proxy/reverse/ -B 2 -A 2

# Check for other content types
rg "Content-Type" src/proxy/reverse/ | grep -v "json\|event-stream"

# Look for response buffering vs streaming
rg "buffer|stream|chunk" src/proxy/reverse/ -B 2 -A 2
```

### 3. Design Phase (30 min)

Design the ResponseMode enum and related types:

```rust
// Potential design
pub enum ResponseMode {
    Unknown,           // Not yet determined
    Json,             // application/json responses
    SseStream,        // text/event-stream responses
    Binary,           // application/octet-stream (future)
    Text,            // text/plain (future)
}

// Per-request tracking?
pub struct RequestContext {
    pub accepts_sse: bool,
    pub response_mode: ResponseMode,
}
```

### 4. Documentation Phase (15 min)

Document findings and proposed design.

## Expected Deliverables

### New Files
- `analysis/response-mode-investigation.md` - Complete analysis of response modes
- `analysis/response-mode-design.md` - Proposed ResponseMode enum design

### Analysis Structure

```markdown
# Response Mode Investigation

## Current is_sse_session Usage

### Setting Points
- Location: file:line - Trigger: ...
- Location: file:line - Trigger: ...

### Checking Points
- Location: file:line - Behavior when true: ...
- Location: file:line - Behavior when false: ...

## Response Mode Patterns

### JSON Response Pattern
- Detection: Content-Type header
- Handling: Buffered, parsed
- Session impact: None

### SSE Stream Pattern
- Detection: Content-Type: text/event-stream
- Handling: Streamed, chunked
- Session impact: Sets is_sse_session

### Other Patterns
- Binary responses: ...
- Text responses: ...
- Error responses: ...

## Accept Header Analysis

### Client Capabilities
- How Accept header is parsed
- What client accepts (json, sse, both)

### Server Response Choice
- How server chooses response type
- Negotiation logic

## Session vs Request Scope

### Current Approach
- is_sse_session is session-scoped
- Problems with this approach

### Recommended Approach
- Response mode per request
- Session tracks capabilities

## Proposed ResponseMode Design

### Enum Definition
```rust
pub enum ResponseMode {
    Unknown,
    Json,
    SseStream,
    // Future additions
}
```

### Integration Points
- Where to store ResponseMode
- When to determine ResponseMode
- How to transition between modes

### Migration from is_sse_session
- Mapping: is_sse_session=true → ResponseMode::SseStream
- Mapping: is_sse_session=false → ResponseMode::Json or Unknown

## Benefits of ResponseMode

### Type Safety
- Explicit states vs boolean

### Extensibility
- Easy to add new modes

### Clarity
- Clear what's being tracked

## Implementation Strategy

### Phase 1: Add ResponseMode
- Add enum alongside is_sse_session

### Phase 2: Parallel Usage
- Set both for compatibility

### Phase 3: Migration
- Replace is_sse_session checks

### Phase 4: Cleanup
- Remove is_sse_session

## Testing Considerations

### Test Scenarios
- JSON-only sessions
- SSE-only sessions
- Mixed mode sessions
- Mode transitions

### Backward Compatibility
- Existing code expectations
- API surface changes
```

## Success Criteria Checklist

- [ ] All is_sse_session usage points documented
- [ ] Response patterns clearly identified
- [ ] ResponseMode enum designed
- [ ] Migration strategy defined
- [ ] Benefits clearly articulated
- [ ] Test scenarios identified
- [ ] Backward compatibility addressed

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Misunderstanding response patterns | HIGH | Thorough code analysis, testing |
| Breaking existing behavior | HIGH | Parallel implementation, gradual migration |
| Missing edge cases | MEDIUM | Comprehensive test scenarios |

## Duration Estimate

**Total: 2 hours**
- Flag Usage Analysis: 45 minutes
- Response Pattern Analysis: 30 minutes
- Design Phase: 30 minutes
- Documentation: 15 minutes

## Dependencies

None - can be done in parallel with A.0 and A.1

## Integration Points

- **Session Management**: How sessions track response mode
- **HTTP Headers**: Accept and Content-Type negotiation
- **Message Processing**: Different handling per mode
- **SSE Streaming**: Special handling for SSE mode

## Notes

- Focus on understanding the "why" behind is_sse_session
- Consider future response modes we might need
- Think about per-request vs per-session tracking
- Consider MCP protocol requirements

## Commands Reference

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Find all boolean usage
rg "is_sse_session" --type rust --stats

# Check session methods
rg "impl.*Session" --type rust -A 20 | grep -A 5 "sse"

# Look for response handling
rg "handle.*response|process.*response" src/proxy/reverse/ -A 10

# Content negotiation
rg "Accept|Content-Type" src/proxy/reverse/ --type rust
```

---

**Task Status**: ⬜ Not Started
**Created**: 2025-08-16
**Last Modified**: 2025-08-16
**Author**: Transport Architecture Team