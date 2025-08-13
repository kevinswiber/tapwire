# Breaking Change Risk Assessment

**Date**: 2025-08-13  
**Author**: Architecture Analysis Team  
**Status**: Phase 0 - Risk Assessment Complete

## Executive Summary

This document identifies all breaking changes that will result from the IncomingTransport/OutgoingTransport refactor and provides mitigation strategies for each risk area.

## Risk Categories

### 1. Critical Breaking Changes (High Impact)

#### 1.1 Transport Trait Modification
**Current State**: Single `Transport` trait used for all transport types  
**Proposed Change**: Split into `IncomingTransport` and `OutgoingTransport`  
**Impact**:
- All 7 transport implementations must be rewritten
- All proxy code using Transport trait breaks
- Mock transports in tests need updating

**Affected Files**:
- `src/transport/mod.rs` - trait definition
- `src/transport/stdio.rs` - StdioTransport
- `src/transport/stdio_client.rs` - StdioClientTransport  
- `src/transport/http.rs` - HttpTransport
- `src/transport/http_mcp.rs` - HttpMcpTransport
- `src/transport/sse_transport.rs` - SseTransport
- `src/transport/sse_interceptor.rs` - InterceptedSseTransport
- `src/proxy/forward.rs` - forward proxy usage
- `src/proxy/reverse.rs` - reverse proxy usage
- All test files with mock transports

**Migration Strategy**:
1. Create compatibility shim: `LegacyTransport` wrapper
2. Implement both new traits for existing transports temporarily
3. Gradual migration with deprecation warnings
4. Provide automated migration tool/script

#### 1.2 Process Management Extraction
**Current State**: `StdioTransport` manages subprocess lifecycle  
**Proposed Change**: Extract to separate `ProcessManager`  
**Impact**:
- Connection pooling for StdioTransport breaks
- All code spawning processes via StdioTransport fails
- Cleanup handlers tied to transport lifecycle break

**Affected Code**:
```rust
// Current
let mut transport = StdioTransport::new(cmd);
transport.connect().await?; // Spawns process

// Proposed
let process = ProcessManager::spawn(cmd).await?;
let transport = SubprocessOutgoing::new(process);
```

**Migration Strategy**:
1. Keep process spawning in StdioTransport initially
2. Add ProcessManager as optional dependency
3. Deprecate process methods in StdioTransport
4. Provide migration examples in documentation

### 2. Major Breaking Changes (Medium Impact)

#### 2.1 CLI Interface Changes
**Current State**:
```bash
shadowcat forward stdio -- command
shadowcat forward http --url http://server
shadowcat forward sse --url http://server
```

**Proposed Change**:
```bash
shadowcat forward --from stdio --to subprocess -- command
shadowcat forward --from stdio --to streamable-http https://server/mcp
shadowcat reverse --listen :8080 --upstream subprocess -- command
```

**Impact**:
- All existing scripts break
- Documentation becomes invalid
- User workflows disrupted

**Migration Strategy**:
1. Support both old and new CLI formats initially
2. Show deprecation warnings for old format
3. Provide migration guide with examples
4. Auto-suggest new format when old is used

#### 2.2 TransportFactory API Changes
**Current State**: Factory creates transports without direction awareness  
**Proposed Change**: Separate factories for incoming/outgoing  

**Affected Code**:
```rust
// Current
let transport = factory.create(TransportSpec::Http { url, session_id })?;

// Proposed  
let outgoing = OutgoingFactory::create_http(url)?;
let incoming = IncomingFactory::create_stdio()?;
```

**Migration Strategy**:
1. Keep existing factory with adapter pattern
2. New factories delegate to old initially
3. Gradual migration of factory users
4. Clear deprecation timeline

### 3. Moderate Breaking Changes (Low Impact)

#### 3.1 Transport Naming Changes
**Current**: Confusing names (StdioTransport spawns processes)  
**Proposed**: Clear names (SubprocessOutgoing, StdioIncoming)  

**Impact**:
- Import statements break
- Type names in code change
- Documentation references invalid

**Migration Strategy**:
1. Type aliases for old names: `type StdioTransport = SubprocessOutgoing;`
2. Deprecation warnings on aliases
3. Automated refactoring tool
4. Clear mapping documentation

#### 3.2 Session Management Changes
**Current**: Transport creates SessionId  
**Proposed**: Session manager handles SessionId  

**Impact**:
- Session creation logic moves
- Transport constructors change signature
- Session lifecycle management changes

**Migration Strategy**:
1. Accept optional SessionId in new transports
2. Auto-create if not provided (compatibility)
3. Deprecate transport session methods
4. Provide session management examples

## Risk Matrix

| Risk | Likelihood | Impact | Mitigation Effort | Priority |
|------|------------|--------|------------------|----------|
| Transport trait break | Certain | Critical | High | P0 |
| Process extraction | Certain | High | Medium | P0 |
| CLI changes | Certain | Medium | Low | P1 |
| Factory changes | Certain | Medium | Medium | P1 |
| Naming changes | Certain | Low | Low | P2 |
| Session changes | Likely | Low | Low | P2 |

## Migration Timeline

### Phase 1: Compatibility Layer (Week 1)
- Create LegacyTransport wrapper
- Add type aliases for old names
- Support both CLI formats
- No breaking changes yet

### Phase 2: Deprecation Warnings (Week 2)
- Add deprecation warnings to old APIs
- Provide migration documentation
- Release migration tools
- Announce deprecation timeline

### Phase 3: New API Default (Week 3)
- New APIs become primary
- Old APIs still work but warned
- Update all documentation
- Migrate internal usage

### Phase 4: Old API Removal (Week 4+)
- Remove deprecated APIs
- Clean up compatibility code
- Final documentation update
- Version 2.0 release

## Compatibility Requirements

### Must Maintain Compatibility
1. Existing proxy deployments must not break
2. Recording/replay functionality must work
3. Session management must be preserved
4. Performance must not degrade

### Can Break With Migration Path
1. Transport trait API
2. CLI interface
3. Factory patterns
4. Internal module structure

### Can Break Immediately
1. Undocumented internal APIs
2. Test-only interfaces
3. Experimental features

## Testing Strategy

### Regression Prevention
1. Keep existing test suite running
2. Add compatibility tests
3. Test both old and new APIs
4. Performance regression tests

### Migration Testing
1. Test migration scripts
2. Validate compatibility layers
3. Test deprecation warnings
4. End-to-end migration scenarios

## Communication Plan

### Internal Communication
1. RFC document for design review
2. Migration guide for developers
3. Weekly progress updates
4. Breaking change announcements

### External Communication
1. Deprecation notices in release notes
2. Migration guide in documentation
3. Blog post explaining benefits
4. Support for migration questions

## Success Metrics

### Migration Success
- [ ] Zero unplanned breaking changes
- [ ] < 5% performance regression
- [ ] 100% test coverage maintained
- [ ] Clear migration path for all changes

### User Impact
- [ ] < 1 week migration effort for users
- [ ] Automated tools handle 80% of migration
- [ ] Documentation covers all scenarios
- [ ] Support tickets < 10 for migration

## Recommendations

### High Priority
1. **Build compatibility layer first** - Ensure zero downtime migration
2. **Extensive testing** - Cover all edge cases before release
3. **Clear communication** - Over-communicate changes
4. **Migration automation** - Provide tools to ease transition

### Medium Priority
1. **Phased rollout** - Don't change everything at once
2. **Feature flags** - Allow toggling between old/new
3. **Monitoring** - Track usage of deprecated APIs
4. **Feedback loops** - Gather user input during migration

### Low Priority
1. **Performance optimization** - Can be done after migration
2. **Additional features** - Focus on compatibility first
3. **Code cleanup** - Delay until old APIs removed

## Conclusion

The transport refactor presents significant breaking change risks, but with proper planning and migration strategies, these can be managed effectively. The key is to:

1. Provide a comprehensive compatibility layer
2. Give users ample time and tools to migrate
3. Communicate changes clearly and frequently
4. Test thoroughly at every stage

The benefits of the refactor (clarity, maintainability, performance) justify the migration effort, but execution must be careful and methodical to minimize user disruption.

## Appendix: Affected Public APIs

### Transport Module Exports
```rust
// Will change
pub use transport::{Transport, StdioTransport, StdioClientTransport};

// New exports
pub use transport::{IncomingTransport, OutgoingTransport};
pub use transport::{StdioIncoming, SubprocessOutgoing};
```

### CLI Commands
```bash
# Deprecated
shadowcat forward stdio
shadowcat forward http
shadowcat forward sse

# New
shadowcat forward --from <incoming> --to <outgoing>
shadowcat reverse --listen <addr> --upstream <outgoing>
```

### Factory APIs
```rust
// Deprecated
TransportFactory::create(spec)

# New
IncomingFactory::create(spec)
OutgoingFactory::create(spec)
```

---

**Risk Assessment Complete**: Ready to proceed with Phase 1 implementation with full awareness of breaking changes and mitigation strategies.