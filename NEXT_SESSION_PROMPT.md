# Next Session: Transport Context Refactor - Phase 3 (Binary & Tests)

## 🎉 Phase 2 Complete!
The library now compiles successfully with the new MessageEnvelope architecture. All core modules have been updated and type aliases removed.

## Current Status
Read `shadowcat/plans/transport-context-refactor/PROGRESS.md` for full details of what was accomplished.

### What's Done:
- ✅ MessageEnvelope replaces Frame everywhere in the library
- ✅ MessageDirection replaces Direction 
- ✅ ProtocolMessage used directly (no TransportMessage alias)
- ✅ All recorder, proxy, transport, and session modules updated
- ✅ Library builds with zero errors!

### What's Left:
- ❌ main.rs still uses old Frame references (9 errors)
- ❌ Tests haven't been run yet
- ❌ Minor warnings about unused imports

## Phase 3 Goals (~4.5 hours)

### 1. Fix main.rs Binary (2 hours)
The main binary has 9 compilation errors, all related to old Frame usage:
- 4 instances of `shadowcat::session::Frame::new` need updating
- Transport.send() calls need MessageEnvelope instead of ProtocolMessage
- Fix transport_message_to_json function
- Update frame.direction access to frame.context.direction

### 2. Run and Fix Tests (2 hours)
```bash
cd /Users/kevin/src/tapwire/shadowcat
cargo test
```
Fix any test failures related to the refactor.

### 3. Clean Up (30 minutes)
- Remove unused imports causing warnings
- Run `cargo clippy --all-targets -- -D warnings`
- Run `cargo fmt`

## Key Patterns to Apply

When you see Frame::new, replace with:
```rust
let context = MessageContext::new(
    session_id,
    MessageDirection::ClientToServer, // or ServerToClient
    TransportContext::stdio(), // or appropriate transport
);
let envelope = MessageEnvelope::new(message, context);
```

When Transport.send() expects MessageEnvelope:
```rust
// Wrap the ProtocolMessage
let envelope = MessageEnvelope::new(
    protocol_message,
    MessageContext::new(session_id, direction, transport_context)
);
transport.send(envelope).await?;
```

## Success Criteria

- [ ] `cargo build` succeeds completely (binary and lib)
- [ ] `cargo test` passes
- [ ] No clippy warnings with `-D warnings`
- [ ] Update tracker.md with completion

## Remember

This is the final phase of a very successful refactor that's already saved 75% of the estimated time. The hard architectural work is done - this is just updating the binary and ensuring quality.

Good luck!