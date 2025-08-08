# Next Session: Transport Context Refactor - Phase 0 Analysis

## Context

We are beginning a critical architectural refactor of Shadowcat's transport layer. This refactor is a **prerequisite** for SSE proxy integration and addresses a fundamental design issue where the `TransportMessage` enum conflates multiple protocol layers.

### Current Situation
- **Problem**: `TransportMessage` mixes transport, MCP protocol, and JSON-RPC layers
- **Critical Issue**: Notifications lack direction (source/destination)
- **Impact**: Used in 90 files with 658 occurrences
- **Blocker**: SSE integration cannot proceed without proper layer separation

## Phase 0 Tasks

All task details are in: `plans/transport-context-refactor/tasks/`

1. **[A.0: Analyze MCP Protocol Specifications](plans/transport-context-refactor/tasks/A.0-mcp-protocol-analysis.md)** (2 hours)
2. **[A.1: Analyze TransportMessage Usage](plans/transport-context-refactor/tasks/A.1-transport-message-usage-analysis.md)** (3 hours)
3. **[A.2: Design MessageEnvelope Structure](plans/transport-context-refactor/tasks/A.2-design-message-envelope.md)** (2 hours)
4. **[A.3: Create Migration Strategy](plans/transport-context-refactor/tasks/A.3-create-migration-strategy.md)** (2 hours)
5. **[A.4: Document Breaking Changes](plans/transport-context-refactor/tasks/A.4-document-breaking-changes.md)** (1 hour)

## Analysis Output

**IMPORTANT**: Write all analysis findings to: `plans/transport-context-refactor/analysis/`

Expected deliverables:
- `mcp-protocol-layers.md` - Protocol layer analysis from A.0
- `architecture-clarification.md` - Architecture understanding from A.0
- `transport-message-usage.md` - Usage analysis report from A.1
- `migration-impact.md` - Impact assessment from A.1
- `current-workarounds.md` - Existing patterns catalog from A.1
- `message-envelope-design.md` - Design document from A.2
- `migration-guide.md` - Migration guide from A.3
- `migration-progress.md` - Progress tracker from A.3
- `compatibility-matrix.md` - Compatibility details from A.3
- `breaking-changes.md` - Breaking changes document from A.4

## Essential Context Files

### Tracker and Tasks
- **Primary Tracker**: `plans/transport-context-refactor/transport-context-tracker.md`
- **Task Details**: `plans/transport-context-refactor/tasks/A.*.md`

### Implementation Files to Analyze
- **Transport Definition**: `shadowcat/src/transport/mod.rs`
- **Transport Implementations**: `shadowcat/src/transport/*.rs`
- **Proxy Usage**: `shadowcat/src/proxy/*.rs`
- **Session Management**: `shadowcat/src/session/manager.rs`

### MCP Specifications
- `specs/mcp/docs/specification/2025-06-18/`
- `specs/mcp/docs/specification/2025-03-26/`

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Key Questions to Answer

From the task files:
1. Are notifications truly bidirectional in MCP?
2. How many of the 658 TransportMessage occurrences are actual usage vs imports?
3. Which components are tightly coupled to TransportMessage?
4. What existing metadata workarounds can we leverage?
5. What are the unavoidable breaking changes?

## Development Workflow

1. Start with task A.0 - read the task file for full details
2. Write findings to the analysis directory
3. Use TodoWrite tool to track progress
4. Run the analysis commands from the task files
5. Update the tracker when tasks complete

## Success Criteria

- [ ] All Phase 0 tasks completed
- [ ] Analysis documents created in `analysis/` directory
- [ ] Clear understanding of refactor scope
- [ ] Migration strategy defined
- [ ] Breaking changes documented
- [ ] Tracker updated with findings

---

**Session Goal**: Complete Phase 0 analysis, producing comprehensive documentation in the `analysis/` directory that will guide the implementation phases of the transport context refactor.