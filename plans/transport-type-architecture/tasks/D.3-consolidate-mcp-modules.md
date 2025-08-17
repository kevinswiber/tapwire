# Task D.3: Consolidate MCP Protocol Modules

## Status: 🔄 IN PROGRESS

## Objective
Consolidate all MCP protocol-related code from `src/mcp/`, `src/protocol/`, and `src/transport/protocol/` into a single, well-organized `src/mcp/` module at the top level.

## Rationale
- **Current problem**: Protocol logic is scattered across three different modules
- **Confusion**: Unclear where to find or add protocol-related code
- **Solution**: Single `mcp` module that contains all MCP protocol logic

## Target Structure
```
src/mcp/
├── mod.rs              # Main module with public re-exports
├── messages.rs         # ProtocolMessage, MessageEnvelope, MessageContext
├── handler.rs          # ProtocolHandler trait, McpProtocolHandler impl
├── handshake.rs        # McpHandshake, version negotiation
├── version.rs          # VersionState, version utilities
├── constants.rs        # Protocol constants, supported versions
├── types.rs            # Core types: SessionId, MessageDirection, etc.
├── encoding.rs         # Serialization/deserialization utilities
└── validation.rs       # Message validation logic
```

## Implementation Steps

### Phase 1: Inventory Current Code
- [ ] Map all types in `src/mcp/`
- [ ] Map all types in `src/protocol/`
- [ ] Map all types in `src/transport/protocol/`
- [ ] Identify dependencies and usage patterns

### Phase 2: Create New Structure
- [ ] Create new `src/mcp/` module structure
- [ ] Move and consolidate message types
- [ ] Move protocol handlers
- [ ] Move version negotiation logic
- [ ] Consolidate constants and utilities

### Phase 3: Update Imports
- [ ] Update all imports throughout codebase
- [ ] Fix compilation errors
- [ ] Update tests

### Phase 4: Cleanup
- [ ] Delete old modules
- [ ] Verify all tests pass
- [ ] Update documentation

## Breaking Changes
- All imports will change from `transport::protocol::*` to `mcp::*`
- Some type locations will change
- No backward compatibility needed (shadowcat unreleased)

## Success Criteria
- [ ] Single `src/mcp/` module contains all MCP logic
- [ ] Clear, logical file organization
- [ ] All tests passing
- [ ] Cleaner import statements throughout codebase
- [ ] No duplicate or scattered protocol code

## Estimated Duration
2-3 hours

## Notes
- This continues the architectural simplification started with transport module reorganization
- Aligns with principle: "one concept, one location"
- Makes codebase more maintainable and discoverable