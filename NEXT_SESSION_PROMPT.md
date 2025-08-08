# Next Session: Complete Transport Context Refactor Phase 0

## Context

We are completing Phase 0 of the Transport Context Refactor for Shadowcat. This refactor is **critical** for SSE proxy integration and addresses the fundamental issue where `TransportMessage` lacks direction information for notifications.

### Current Status
- **Phase 0**: 40% complete (2 of 5 tasks done)
- **Analysis completed**: Protocol layers understood, usage mapped, impact assessed
- **Remaining work**: Design the solution and migration strategy

## Key Resources

### Primary Tracker
**Read this first**: `plans/transport-context-refactor/transport-context-tracker.md`

### Completed Analysis (Reference as needed)
- `plans/transport-context-refactor/analysis/mcp-protocol-layers.md` - Protocol understanding
- `plans/transport-context-refactor/analysis/transport-message-usage.md` - 34 files, 330 occurrences
- `plans/transport-context-refactor/analysis/migration-impact.md` - 60 hour estimate
- `plans/transport-context-refactor/analysis/current-workarounds.md` - 17 patterns to eliminate
- `plans/transport-context-refactor/analysis/architecture-clarification.md` - Layer separation

## Tasks for This Session

### Task A.2: Design MessageEnvelope Structure (2 hours)
**File**: `plans/transport-context-refactor/tasks/A.2-design-message-envelope.md`

**Goal**: Create concrete Rust type definitions for:
- `MessageEnvelope` wrapper type
- `TransportContext` for transport metadata  
- `SessionContext` for MCP session info
- Conversion traits for compatibility

**Key Requirements**:
- Must solve the notification direction problem
- Must preserve all 17 workaround functionalities
- Must be zero-cost where possible
- Must allow incremental migration

### Task A.3: Create Migration Strategy (2 hours)
**File**: `plans/transport-context-refactor/tasks/A.3-create-migration-strategy.md`

**Goal**: Define the step-by-step migration plan:
- Compatibility layer design
- Migration phases and checkpoints
- Feature flags or version toggles
- Testing strategy for each phase

### Task A.4: Document Breaking Changes (1 hour)
**File**: `plans/transport-context-refactor/tasks/A.4-document-breaking-changes.md`

**Goal**: Create clear documentation for:
- What will break and why
- Migration guide for developers
- Timeline and deprecation notices

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Success Criteria

- [ ] MessageEnvelope types defined in design document
- [ ] Migration strategy with clear phases documented  
- [ ] Breaking changes documented for stakeholders
- [ ] All outputs in `plans/transport-context-refactor/analysis/` directory
- [ ] Tracker updated with completion status

## Note

This completes Phase 0. The actual implementation (Phases 1-4) will be done in subsequent sessions, starting with core infrastructure, then transports, then proxy/session layers.