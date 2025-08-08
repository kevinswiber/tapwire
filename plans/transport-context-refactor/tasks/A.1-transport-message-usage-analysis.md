# Task A.1: Analyze TransportMessage Usage

**Duration**: 3 hours  
**Dependencies**: A.0 (MCP Protocol Analysis)  
**Status**: ✅ Completed  

## Objective

Conduct a comprehensive analysis of how `TransportMessage` is currently used across the codebase (90 files, 658 occurrences) to inform the design of the new `MessageEnvelope` system and migration strategy.

## Key Questions to Answer

1. **How many of the 658 occurrences are actual usage vs imports?**
2. **Which components are tightly coupled to TransportMessage structure?**
3. **Where is transport metadata currently being passed outside of TransportMessage?**
4. **What existing patterns can we leverage for the new context system?**
5. **Which files can remain unchanged with a compatibility layer?**
6. **What are the riskiest parts of the migration?**
7. **Where are notifications currently handled and do they track direction?**

## Analysis Approach

### Phase 1: Quantitative Analysis (45 min)

Gather raw data about TransportMessage usage:

```bash
# Find all files with TransportMessage
rg "TransportMessage" --type rust -l | sort > transport-message-files.txt

# Count occurrences per file
rg "TransportMessage" --type rust -c | sort -t: -k2 -nr > transport-message-counts.txt

# Find import statements
rg "use.*TransportMessage" --type rust > transport-message-imports.txt

# Find actual usage (not just imports)
rg "TransportMessage::" --type rust -A 2 -B 2 > transport-message-usage.txt

# Find pattern matching on TransportMessage
rg "match.*TransportMessage|if let.*TransportMessage" --type rust > transport-message-matching.txt

# Find functions that take/return TransportMessage
rg "fn.*TransportMessage|\) -> .*TransportMessage" --type rust > transport-message-functions.txt
```

### Phase 2: Categorization (1 hour)

Categorize files into usage patterns:

#### Category 1: Import-Only Files
- Files that only import but don't manipulate TransportMessage
- Can use compatibility layer unchanged
- Low migration priority

#### Category 2: Message Creators
- Files that construct TransportMessage instances
- Need migration to include context creation
- Priority based on transport type

#### Category 3: Message Consumers  
- Files that match on or destructure TransportMessage
- Need updates to handle context
- May need both old and new paths

#### Category 4: Message Transformers
- Files that modify or convert TransportMessage
- Critical for migration
- Need careful context preservation

#### Category 5: Transport Implementations
- The actual transport layer implementations
- First to be migrated
- Source of transport metadata

### Phase 3: Metadata Analysis (45 min)

Find existing transport metadata patterns:

```bash
# Find where headers are handled
rg "HeaderMap|headers" --type rust -l | grep -E "(transport|proxy|session)"

# Find session ID handling
rg "SessionId|session_id" --type rust -l

# Check for existing metadata patterns
rg "metadata|context|envelope" --type rust -l | grep -E "(transport|proxy)"

# Find notification handling
rg "Notification|notification" --type rust -C 3 | grep -E "TransportMessage"

# Look for direction/routing patterns
rg "direction|source|destination|from|to" --type rust | grep -E "TransportMessage"
```

### Phase 4: Impact Assessment (30 min)

Identify critical paths and risks:

1. **Critical Paths**
   - Core proxy forwarding logic
   - Session management
   - Interceptor chain
   - Recorder/replay

2. **Performance-Sensitive Areas**
   - Message forwarding hot path
   - High-frequency parsing
   - Memory allocation patterns

3. **Breaking Change Risks**
   - Public API changes
   - Serialization format changes
   - Protocol compliance issues

## Deliverables

### 1. Usage Analysis Report
**Location**: `plans/transport-context-refactor/analysis/transport-message-usage.md`

Structure:
```markdown
# TransportMessage Usage Analysis

## Summary Statistics
- Total files: 90
- Total occurrences: 658
- Import-only files: X (Y%)
- Active usage files: X (Y%)
- Critical path files: X

## Usage Categories

### Category 1: Import-Only (X files)
Files that only import TransportMessage without manipulation.

**Files**: 
- file1.rs (2 occurrences)
- file2.rs (1 occurrence)
...

**Migration Strategy**: Use compatibility layer

### Category 2: Message Creators (X files)
Files that construct TransportMessage instances.

**High Priority** (transport implementations):
- transport/stdio.rs
- transport/http.rs
...

**Medium Priority** (proxy/session):
- proxy/forward.rs
...

### Category 3: Message Consumers (X files)
...

### Category 4: Message Transformers (X files)
...

### Category 5: Transport Implementations (X files)
...

## Critical Paths

### Path 1: Request Forwarding
- proxy/forward.rs -> transport/mod.rs -> ...
- Impact: Every proxied request
- Risk: HIGH

### Path 2: Session Management
...

## Existing Metadata Patterns

### Headers Handling
- Currently passed via: ...
- Files involved: ...

### Session Tracking
- Pattern: ...
- Files: ...

### Notification Direction
- Current approach: ...
- Issues: ...

## Migration Recommendations

### Phase 1: Core Infrastructure
1. transport/mod.rs - Add context support
2. transport/envelope.rs - New types
...

### Phase 2: Transport Implementations
...

### Phase 3: Proxy Layer
...

### Phase 4: Remaining Components
...
```

### 2. Migration Impact Assessment
**Location**: `plans/transport-context-refactor/analysis/migration-impact.md`

Structure:
```markdown
# Migration Impact Assessment

## Breaking Changes

### Unavoidable Breaks
1. Transport trait signature changes
2. ...

### Mitigatable Breaks
1. Can provide default implementations
2. ...

## Compatibility Requirements

### Must Maintain Compatibility
- Public CLI interface
- Wire protocol format
- ...

### Can Break (Internal)
- Internal module interfaces
- ...

## Performance Considerations

### Hot Paths
1. Message forwarding: X calls/sec
2. ...

### Memory Impact
- Current TransportMessage size: X bytes
- Projected MessageEnvelope size: Y bytes
- Impact: Z% increase

## Risk Matrix

| Component | Risk Level | Impact | Mitigation |
|-----------|------------|--------|------------|
| Forward Proxy | HIGH | Every request | Gradual migration |
| ... | ... | ... | ... |

## Testing Requirements

### Unit Tests Needed
- Envelope creation/conversion
- Context preservation
- ...

### Integration Tests Needed
- End-to-end proxy flow
- Session continuity
- ...

## Timeline Estimate

Based on analysis:
- Phase 1 (Core): 10 hours
- Phase 2 (Transports): 9 hours  
- Phase 3 (Proxy): 8 hours
- Phase 4 (Testing): 5 hours
- Buffer: 8 hours
- **Total**: 40 hours
```

### 3. Current Workarounds Catalog
**Location**: `plans/transport-context-refactor/analysis/current-workarounds.md`

Document existing patterns that work around TransportMessage limitations.

## Process Steps

### Step 1: Data Collection (45 min)
1. Run all analysis commands
2. Save outputs to analysis folder
3. Create initial statistics

### Step 2: File Categorization (45 min)
1. Review each file's usage pattern
2. Assign to categories
3. Note special cases

### Step 3: Pattern Analysis (45 min)
1. Identify metadata workarounds
2. Find notification handling
3. Document routing patterns

### Step 4: Impact Assessment (30 min)
1. Identify critical paths
2. Assess risks
3. Estimate timeline

### Step 5: Documentation (30 min)
1. Create usage analysis report
2. Write impact assessment
3. Document workarounds

### Step 6: Validation (15 min)
1. Cross-check findings
2. Validate with codebase
3. Update tracker

## Success Criteria

- [ ] All 90 files categorized
- [ ] 658 occurrences mapped to usage types
- [ ] Critical paths identified
- [ ] Existing workarounds documented
- [ ] Performance impacts assessed
- [ ] Migration phases prioritized
- [ ] Risk matrix completed
- [ ] Timeline validated

## Commands Reference

```bash
# Navigate to shadowcat
cd /Users/kevin/src/tapwire/shadowcat

# Quick statistics
echo "Files with TransportMessage:"
rg "TransportMessage" --type rust -l | wc -l

echo "Total occurrences:"
rg "TransportMessage" --type rust -o | wc -l

echo "Import statements:"
rg "use.*TransportMessage" --type rust | wc -l

echo "Actual usage (not imports):"
rg "TransportMessage::" --type rust | wc -l

# Top 10 files by occurrence count
rg "TransportMessage" --type rust -c | sort -t: -k2 -nr | head -10
```

## Notes

- Focus on understanding patterns, not fixing issues
- Document everything, even if it seems minor
- Pay attention to notification handling
- Look for implicit assumptions about message direction
- Note any creative workarounds for metadata

## Related Tasks

- **Depends on**: A.0 - MCP Protocol Analysis
- **Next**: A.2 - Design MessageEnvelope Structure  
- **Enables**: Migration strategy planning

---

**Task Owner**: _Unassigned_  
**Created**: 2025-08-08  
**Last Updated**: 2025-08-08