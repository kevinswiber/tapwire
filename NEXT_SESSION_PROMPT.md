# Next Session: Transport Context Refactor - Phase 0, Tasks A.0-A.1: MCP Specification and Usage Analysis

## Context

We are beginning a critical architectural refactor of Shadowcat's transport layer. This refactor is a **prerequisite** for SSE proxy integration and addresses a fundamental design issue where the `TransportMessage` enum conflates multiple protocol layers.

### Current Situation
- **Problem**: `TransportMessage` mixes three distinct layers:
  1. **Transport Layer** (HTTP/SSE/stdio) - How bytes move
  2. **MCP Protocol Layer** - Application semantics (bidirectional notifications!)
  3. **JSON-RPC Layer** - Message framing and correlation
- **Critical Issue**: Notifications lack direction (source/destination)
- **Impact**: Used in 90 files with 658 occurrences
- **Blocker**: SSE integration cannot proceed without proper layer separation
- **Solution**: Design proper layer separation with MessageEnvelope system

### What Has Been Completed
- **Transport Context Refactor Tracker created** (2025-08-08)
  - Comprehensive plan for separating protocol from transport concerns
  - Identified need for backward compatibility during migration
  - Established 5-phase implementation approach

- **MCP Parser Enhanced** (2025-08-08)
  - Added SSE integration support to minimal parser
  - Created TransportMessage conversion methods
  - However, discovered the fundamental architectural issue requiring this refactor

## Objectives

### Task A.0: Analyze MCP Protocol Specifications (2 hours)
Understand the proper protocol layering by studying the MCP specifications to ensure our refactor aligns with the actual protocol design.

### Task A.1: Analyze TransportMessage Usage (3 hours)
Conduct a comprehensive analysis of how `TransportMessage` is currently used across the codebase to inform the design of the new `MessageEnvelope` system and migration strategy.

## Essential Context Files to Read

### MCP Specifications (CRITICAL - Read First!)
1. **MCP 2025-06-18 Spec**: `specs/mcp/docs/specification/2025-06-18/`
   - `protocol/index.mdx` - Protocol overview
   - `transports/index.mdx` - Transport layer design
   - `transports/http-sse.md` - HTTP/SSE specific requirements
2. **MCP 2025-03-26 Spec**: `specs/mcp/docs/specification/2025-03-26/`
   - `basic/architecture.mdx` - Architecture overview
   - `basic/transports.mdx` - Transport requirements
   - Focus on how notifications work bidirectionally

### Implementation Files

1. **Primary Tracker**: `plans/transport-context-refactor/transport-context-tracker.md`
2. **Current Transport Definition**: `shadowcat/src/transport/mod.rs`
3. **Transport Implementations**: 
   - `shadowcat/src/transport/stdio.rs`
   - `shadowcat/src/transport/http.rs`
   - `shadowcat/src/transport/http_mcp.rs`
4. **Proxy Usage**:
   - `shadowcat/src/proxy/forward.rs`
   - `shadowcat/src/proxy/reverse.rs`
5. **Session Management**: `shadowcat/src/session/manager.rs`

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Task Details

### Task A.0 Deliverables: MCP Specification Analysis

1. **Protocol Layer Analysis** (`plans/transport-context-refactor/analysis/mcp-protocol-layers.md`):
   - Document the actual protocol layers in MCP
   - Clarify notification directionality
   - Map MCP concepts to transport concepts
   - Identify what belongs at each layer

2. **Key Findings**:
   - How do notifications actually work? (bidirectional?)
   - What metadata does MCP require vs what transports provide?
   - How should we handle message direction/routing?
   - What's the relationship between JSON-RPC and MCP layers?

### Task A.1 Deliverables: Usage Analysis

1. **Usage Analysis Report** (`plans/transport-context-refactor/analysis/transport-message-usage.md`):
   - Categorize the 90 files by how they use TransportMessage
   - Identify which files just import vs actively manipulate
   - Find patterns in how transport metadata is currently handled
   - Locate where transport-specific logic is mixed with protocol logic

2. **Impact Assessment** (`plans/transport-context-refactor/analysis/migration-impact.md`):
   - Critical paths that must be migrated first
   - Components that can use compatibility layer
   - Potential breaking changes that cannot be avoided
   - Performance-sensitive areas requiring benchmarks

3. **Current Workarounds Catalog**:
   - Document how HTTP headers are currently passed around
   - Find where session IDs are tracked outside TransportMessage
   - Identify any existing transport metadata handling patterns

### Implementation Strategy

#### Phase 0: MCP Specification Study (2 hours)
1. Read MCP 2025-06-18 and 2025-03-26 specifications
2. Document protocol layering architecture
3. Understand notification model (bidirectional? source/dest?)
4. Map JSON-RPC to MCP to Transport relationships
5. Identify required metadata at each layer

#### Phase 1: Quantitative Analysis (45 min)
1. Use grep/ripgrep to find all TransportMessage occurrences
2. Categorize by file type and module
3. Count actual usage vs imports
4. Create usage heat map

#### Phase 2: Qualitative Analysis (1.5 hours)
1. Examine each major component's usage pattern
2. Identify where transport metadata is needed but missing
3. Find existing workarounds for transport-specific data
4. Document coupling between protocol and transport

#### Phase 3: Migration Planning (45 min)
1. Prioritize components for migration
2. Identify compatibility requirements
3. Design incremental migration path
4. Document risks and mitigation strategies

## Commands to Use

```bash
# Find all files with TransportMessage
rg "TransportMessage" --type rust -l | sort > transport-message-files.txt

# Count occurrences per file
rg "TransportMessage" --type rust -c | sort -t: -k2 -nr

# Find actual usage patterns (not just imports)
rg "TransportMessage::" --type rust -A 2 -B 2

# Find where headers are handled
rg "HeaderMap|headers" --type rust -l | grep -E "(transport|proxy|session)"

# Find session ID handling
rg "SessionId|session_id" --type rust -l

# Check for existing metadata patterns
rg "metadata|context|envelope" --type rust -l | grep -E "(transport|proxy)"

# Analyze imports vs usage
rg "use.*TransportMessage" --type rust
rg "impl.*TransportMessage" --type rust
rg "match.*TransportMessage" --type rust
```

## Analysis Categories

### 1. Import-Only Files
Files that only import TransportMessage but don't manipulate it directly.
- Can likely remain unchanged during initial migration
- Will automatically benefit from compatibility layer

### 2. Message Creators
Files that construct TransportMessage instances.
- Need migration to include context creation
- Priority based on transport type

### 3. Message Consumers
Files that match on or destructure TransportMessage.
- Need updates to handle context
- May need both old and new paths during migration

### 4. Message Transformers
Files that modify or convert TransportMessage.
- Critical for migration
- Need careful context preservation

### 5. Transport Implementations
The actual transport layer implementations.
- First to be migrated
- Source of transport metadata

## Success Criteria Checklist

- [ ] All 90 files categorized by usage pattern
- [ ] 658 occurrences mapped to actual usage vs imports
- [ ] Critical migration paths identified
- [ ] Existing transport metadata workarounds documented
- [ ] Performance-sensitive areas flagged
- [ ] Compatibility requirements clear
- [ ] Migration phases prioritized
- [ ] Analysis reports created in proper locations
- [ ] Tracker updated with findings

## Important Notes

- **Always use TodoWrite tool** to track your progress through the task
- **Start with examining existing code** to understand current architecture
- **Follow established patterns** from previous implementations
- **Test incrementally** as you build each component
- **Run `cargo fmt`** after implementing new functionality
- **Run `cargo clippy --all-targets -- -D warnings`** before any commit
- **Update the refactor tracker** when the task is complete
- **Focus on the current phase objectives**

## Key Questions to Answer

### Protocol Understanding (Task A.0)
1. **Are notifications truly bidirectional in MCP?**
2. **How does MCP handle message routing/direction?**
3. **What's the proper separation between JSON-RPC, MCP, and Transport layers?**
4. **What metadata is required vs optional at each layer?**
5. **How do different transports (HTTP vs SSE vs stdio) map to MCP semantics?**

### Usage Analysis (Task A.1)
1. **How many of the 658 occurrences are actual usage vs imports?**
2. **Which components are tightly coupled to TransportMessage structure?**
3. **Where is transport metadata currently being passed outside of TransportMessage?**
4. **What existing patterns can we leverage for the new context system?**
5. **Which files can remain unchanged with a compatibility layer?**
6. **What are the riskiest parts of the migration?**
7. **Where are notifications currently handled and do they track direction?**

## Model Usage Guidelines

- **IMPORTANT** Be mindful of model capabilities. Assess whether Claude Opus or Claude Sonnet would be best for each step. When there's a benefit to a model change, pause and recommend it. Be mindful of the context window. When the context window has less than 15% availability, suggest creating a new Claude session and output a good prompt, referencing all available plans, tasks, and completion files that are relevant. Save the prompt into NEXT_SESSION_PROMPT.md.

## Development Workflow

1. Create todo list with TodoWrite tool to track progress
2. Examine existing codebase architecture and established patterns
3. Study current implementations related to the task
4. Design the solution approach and identify key components
5. Implement functionality incrementally with frequent testing
6. Add comprehensive error handling following project patterns
7. Create tests demonstrating functionality works correctly
8. Run tests after each significant change to catch issues early
9. Run `cargo fmt` to ensure consistent code formatting
10. Run `cargo clippy -- -D warnings` to catch potential issues
11. Update project documentation and tracker as needed
12. Commit changes with clear, descriptive messages

## Expected Analysis Output Structure

### MCP Protocol Layer Analysis (A.0)

```markdown
# MCP Protocol Layer Analysis

## Protocol Layers
1. Transport Layer
   - Purpose: ...
   - Responsibilities: ...
   
2. MCP Protocol Layer
   - Purpose: ...
   - Responsibilities: ...
   
3. JSON-RPC Layer
   - Purpose: ...
   - Responsibilities: ...

## Notification Model
- Directionality: [Bidirectional/Unidirectional]
- Client→Server notifications: [Examples]
- Server→Client notifications: [Examples]
- Routing requirements: ...

## Required Refactor Changes
- TransportMessage should be renamed to McpMessage
- Notifications need direction field
- Transport metadata must be separate
```

### TransportMessage Usage Analysis (A.1)

```markdown
# TransportMessage Usage Analysis

## Summary
- Total files: 90
- Total occurrences: 658
- Actual usage: X
- Import-only: Y

## Usage Categories

### Category 1: Import-Only (X files)
- File list...
- Can use compatibility layer

### Category 2: Message Creators (X files)
- File list...
- Migration priority: HIGH/MEDIUM/LOW

### Category 3: Message Consumers (X files)
...

## Critical Paths
1. Path 1: Description
2. Path 2: Description

## Existing Workarounds
- Workaround 1: Description
- Workaround 2: Description

## Migration Recommendations
1. Phase 1: Components...
2. Phase 2: Components...
```

## Next Steps After This Task

Once A.1 is complete, the next tasks in Phase 0 are:
- **A.2**: Design MessageEnvelope Structure (2 hours, depends on A.1)
- **A.3**: Create Migration Strategy (2 hours, depends on A.2)
- **A.4**: Document Breaking Changes (1 hour, depends on A.3)

After Phase 0 analysis, we'll move to Phase 1 implementation of the core infrastructure.

## Related Context

This analysis is critical because:
- It determines the scope of the refactor
- It identifies risks and compatibility requirements
- It shapes the MessageEnvelope design
- It prioritizes the migration order
- It prevents breaking changes in unexpected places

The findings will directly influence:
- The design of TransportContext and MessageEnvelope
- The compatibility layer implementation
- The migration phases and timeline
- The testing strategy

---

**Session Goal**: Complete a thorough analysis of TransportMessage usage across the codebase, producing actionable reports that will guide the design and implementation of the transport context refactor. This analysis is the foundation for successfully separating protocol from transport concerns without breaking the existing system.