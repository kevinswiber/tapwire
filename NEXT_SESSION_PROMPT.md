# Next Session: Transport Context Refactor - Aggressive Implementation

## Critical Context: No Released Users = Freedom to Break Things

**Important**: This project hasn't been released yet. We have ZERO external users. This means:
- ✅ Break any API we want
- ✅ Delete old code immediately  
- ✅ No backward compatibility needed
- ✅ No deprecation periods
- ✅ Complete freedom to refactor

## Revised Approach (30-40 hours vs 60)

### What We're Building
- **MessageEnvelope** with full context (direction, session, transport metadata)
- **Direct replacement** of TransportMessage (no compatibility layers)
- **Clean architecture** without workarounds

### Phase 1: Core Refactor (This Session, ~8-10 hours)

#### 1. Create New Types (2 hours)
**File**: `shadowcat/src/transport/envelope.rs`
- MessageEnvelope, MessageContext, MessageDirection
- TransportContext with Http/Sse/Stdio variants
- Delete old Direction enum immediately

#### 2. Replace TransportMessage (2 hours)
**File**: `shadowcat/src/transport/protocol.rs`
- Rename TransportMessage → ProtocolMessage everywhere
- No type alias, just direct replacement
- Update all imports

#### 3. Update Transport Trait (2 hours)
**Files**: `shadowcat/src/transport/mod.rs`, all transport implementations
```rust
// Just change it directly - no compatibility needed
pub trait Transport {
    async fn receive(&mut self) -> Result<MessageEnvelope>;
    async fn send(&mut self, envelope: MessageEnvelope) -> Result<()>;
}
```

#### 4. Update Core Components (4 hours)
- **SessionManager**: Use MessageEnvelope directly
- **Frame**: Delete it, just use MessageEnvelope
- **Proxy**: Update to use envelopes
- Fix tests as we go

## Working Directory
```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Aggressive Refactoring Checklist

- [ ] Delete `Direction` enum - use `MessageDirection`
- [ ] Delete `Frame` struct - use `MessageEnvelope`  
- [ ] Remove all 17 workaround patterns identified
- [ ] Rename TransportMessage → ProtocolMessage everywhere
- [ ] Update Transport trait - no backward compatibility
- [ ] Fix all compilation errors directly
- [ ] Update tests to use new types

## Key Files to Update

### Immediate Changes
1. `src/transport/mod.rs` - Add envelope module, update trait
2. `src/transport/stdio.rs` - Use MessageEnvelope
3. `src/transport/http.rs` - Extract HTTP context
4. `src/transport/http_mcp.rs` - Extract MCP headers
5. `src/session/manager.rs` - Use envelopes throughout
6. `src/session/store.rs` - Delete Frame, use MessageEnvelope

### Delete These
- Old Direction enum
- Frame struct  
- Session extraction heuristics
- Direction inference code
- All workarounds in `current-workarounds.md`

## Success Criteria

- [ ] Code compiles with new types
- [ ] Tests pass (after updating them)
- [ ] No TransportMessage remains
- [ ] No compatibility code
- [ ] Clean architecture

## Benefits of Being Aggressive

1. **Faster**: 30-40 hours instead of 60
2. **Cleaner**: No legacy code or compatibility layers
3. **Simpler**: One code path, not two
4. **Better**: Fix all naming and structure issues now

## Remember

- **We have no users to break**
- **The compiler is our friend** - it will find everything we need to update
- **Delete aggressively** - if it's old, remove it
- **Move fast** - we can always fix issues since no one is using this yet

Let's do this refactor RIGHT without any legacy baggage!