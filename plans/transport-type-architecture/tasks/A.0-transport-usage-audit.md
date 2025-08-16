# Task A.0: Transport Usage Audit

## Objective

Create a comprehensive audit of how `TransportType`, `is_sse_session`, and related transport concepts are used throughout the shadowcat codebase to understand the full scope of changes needed.

## Background

We've identified that `is_sse_session` is a code smell, but before we can safely remove it, we need to understand:
- Every place `TransportType` is used and what it's used for
- All locations checking `is_sse_session` and what they're really checking
- How the forward proxy's directional transports work vs the reverse proxy's approach
- What backward compatibility concerns exist

## Key Questions to Answer

1. Where is `TransportType` enum used and for what purpose?
2. What code paths depend on `is_sse_session` checks?
3. How do forward and reverse proxies differ in transport handling?
4. What needs to be updated when we change the transport model?
5. Are there hidden dependencies on the current transport architecture?

## Step-by-Step Process

### 1. Analysis Phase (60 min)

Map all TransportType usage:

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Find all TransportType usage
rg "TransportType::" --type rust -A 2 -B 2

# Find all is_sse_session references
rg "is_sse_session" --type rust -A 3 -B 3

# Find transport_type field access
rg "\.transport_type" --type rust -A 2 -B 2

# Find mark_as_sse_session calls
rg "mark_as_sse_session" --type rust -A 2 -B 2
```

### 2. Categorization Phase (60 min)

Categorize usage patterns:
- **Configuration**: Where TransportType is used for config
- **Session Management**: Where it's used for session tracking
- **Routing Logic**: Where it determines code paths
- **Protocol Handling**: Where it affects message processing
- **Connection Management**: Where it affects transport lifecycle

### 3. Documentation Phase (60 min)

Create comprehensive documentation of findings.

## Expected Deliverables

### New Files
- `analysis/transport-usage-audit.md` - Complete audit of transport type usage
- `analysis/transport-dependency-map.md` - Visual map of dependencies

### Analysis Structure

The audit document should include:

```markdown
# Transport Usage Audit

## TransportType Enum Usage

### Configuration Context
- File: Path, Line: X, Purpose: ...
- File: Path, Line: Y, Purpose: ...

### Session Management Context
- File: Path, Line: X, Purpose: ...

### Routing Logic Context
- File: Path, Line: X, Logic: ...

## is_sse_session Boolean Usage

### Detection Points
- Where it's set to true
- What triggers the setting

### Check Points
- Where it's checked
- What behavior changes based on the check

## Transport Architecture Comparison

### Forward Proxy
- Uses IncomingTransport/OutgoingTransport
- Clean separation of concerns
- Examples of usage

### Reverse Proxy
- Direct transport handling
- Duplicate implementations
- Connection pooling approach

## Change Impact Analysis

### API Surface Changes
- What interfaces will change
- What modules are affected

### Internal Dependencies
- Cross-module dependencies
- Implicit assumptions

## Recommendations

### Immediate Changes
- What should be changed right away

### Phased Changes
- What needs coordinated updates

### Future Improvements
- Long-term architectural goals
```

## Success Criteria Checklist

- [ ] All TransportType usage locations documented
- [ ] All is_sse_session usage locations documented
- [ ] Forward vs reverse proxy differences clearly mapped
- [ ] Change impact clearly identified
- [ ] Dependency relationships documented
- [ ] Clear categorization of usage patterns
- [ ] Recommendations for clean migration path

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Missing hidden dependencies | HIGH | Thorough grep searches, test suite review |
| Underestimating scope | MEDIUM | Conservative estimates, phased approach |
| Breaking existing users | HIGH | Maintain backward compatibility layer |

## Duration Estimate

**Total: 3 hours**
- Analysis: 60 minutes
- Categorization: 60 minutes
- Documentation: 60 minutes

## Dependencies

None - this is the foundational analysis task

## Integration Points

- **Session Management**: How sessions track transport state
- **Configuration**: How upstreams specify transports
- **Message Routing**: How messages are routed based on transport
- **Connection Pooling**: How connections are managed per transport

## Notes

- This audit is critical - take time to be thorough
- Look for implicit assumptions about transport behavior
- Pay special attention to error handling paths
- Consider performance implications of current approach

## Commands Reference

```bash
# Working directory
cd /Users/kevin/src/tapwire/shadowcat

# Find enum definitions
rg "enum TransportType" --type rust -A 10

# Find struct fields
rg "transport_type:" --type rust -B 5 -A 2

# Find method calls
rg "\.transport_type\(\)" --type rust

# Find pattern matches
rg "match.*transport_type" --type rust -A 10

# Find SSE-specific code
rg "is_sse|mark_as_sse|SseStream" --type rust
```

---

**Task Status**: ⬜ Not Started
**Created**: 2025-08-16
**Last Modified**: 2025-08-16
**Author**: Transport Architecture Team