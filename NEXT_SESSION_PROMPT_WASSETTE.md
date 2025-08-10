# Next Session: Wassette-Shadowcat Integration Phase C Completion

## Context
We've successfully implemented the core Wassette transport and CLI integration for Shadowcat. The basic proxy functionality is working, allowing Shadowcat to spawn and communicate with Wassette processes for WebAssembly-based MCP tool execution.

## Current Status

### ✅ Completed (Phase C partial)
- **C.0 Environment Setup**: Development environment ready
- **C.1 Basic Stdio Proxy**: WassetteTransport fully implemented with:
  - Process spawning and lifecycle management
  - Bidirectional stdio communication
  - CLI integration (`shadowcat forward wassette`)
  - Basic integration tests

### 🔄 In Progress (Phase C remaining)
- **C.2 Recording Integration**: Capture component operations to SQLite storage
- **C.3 Interceptor Implementation**: Enable message modification and debugging

### 📋 Pending (Phase D)
- **D.0 Integration Guide**: Production deployment documentation
- **D.1 Performance Analysis**: Benchmark and optimization
- **D.2 Security Assessment**: Final security review

## Session Objectives
Complete Phase C by implementing recording and interception capabilities for Wassette traffic, then move to Phase D for documentation and analysis.

## Tasks for This Session

### Task C.2: Recording Integration (3 hours)
Integrate Wassette transport with Shadowcat's existing recording infrastructure to capture all WebAssembly component interactions.

**Implementation Steps:**
1. Hook WassetteTransport into the recorder module
2. Capture component initialization and tool calls
3. Store Wassette-specific metadata (component names, capabilities)
4. Enable replay of recorded Wassette sessions
5. Test recording with sample WebAssembly components

**Key Files to Modify:**
- `src/transport/wassette.rs` - Add recording hooks
- `src/recorder/mod.rs` - Wassette-specific recording logic
- `src/recorder/storage.rs` - Schema for Wassette metadata
- `tests/wassette_integration_test.rs` - Recording tests

### Task C.3: Interceptor Implementation (4 hours)
Enable interception of Wassette messages for debugging, modification, and security enforcement.

**Implementation Steps:**
1. Integrate with existing interceptor chain
2. Add Wassette-specific interceptor actions
3. Implement token stripping for security boundary
4. Add debug breakpoints for component calls
5. Create rules for component access control

**Key Files to Modify:**
- `src/transport/wassette.rs` - Interceptor integration points
- `src/interceptor/mod.rs` - Wassette interceptor support
- `src/interceptor/rules.rs` - Component-specific rules
- `src/cli/forward.rs` - CLI options for interception

## Implementation Details

### Recording Architecture
```rust
// In WassetteTransport
impl WassetteTransport {
    async fn send_with_recording(&mut self, envelope: MessageEnvelope) -> TransportResult<()> {
        // Record outbound message
        if let Some(recorder) = &self.recorder {
            recorder.record_frame(Frame {
                session_id: self.session_id.clone(),
                direction: Direction::Outbound,
                message: envelope.message.clone(),
                timestamp: SystemTime::now(),
                metadata: json!({
                    "transport": "wassette",
                    "plugin_dir": self.config.plugin_dir,
                }),
            }).await?;
        }
        
        self.send(envelope).await
    }
}
```

### Interceptor Integration
```rust
// Token stripping interceptor
pub struct WassetteTokenStripper;

impl Interceptor for WassetteTokenStripper {
    async fn intercept(&self, envelope: &mut MessageEnvelope) -> InterceptAction {
        // Remove authentication tokens before sending to Wassette
        if let Some(headers) = envelope.context.metadata.as_mut() {
            headers.remove("authorization");
            headers.remove("x-api-key");
        }
        InterceptAction::Continue
    }
}
```

## Success Criteria

### For C.2 (Recording)
- [ ] All Wassette messages are recorded to SQLite
- [ ] Component metadata is captured (name, version, capabilities)
- [ ] Recorded sessions can be replayed
- [ ] Integration test demonstrates recording

### For C.3 (Interception)
- [ ] Messages can be intercepted and modified
- [ ] Token stripping works at security boundary
- [ ] Debug breakpoints can pause execution
- [ ] Component access rules are enforced

## Testing Strategy

### Integration Tests
```bash
# Test recording
cargo test test_wassette_recording -- --nocapture

# Test interception
cargo test test_wassette_interception -- --nocapture

# End-to-end test with real Wassette
cargo test test_wassette_e2e -- --ignored --nocapture
```

### Manual Testing
```bash
# Start proxy with recording
shadowcat forward wassette \
  --plugin-dir ./test-plugins \
  --record ./wassette-session.tape

# Start proxy with interception
shadowcat forward wassette \
  --plugin-dir ./test-plugins \
  --intercept-rules ./wassette-rules.yaml
```

## Key Challenges to Address

1. **State Management**: WebAssembly components are stateless - how to handle in replay?
2. **Component Discovery**: How to detect available components dynamically?
3. **Error Propagation**: Component errors need proper propagation through proxy
4. **Performance**: Minimize overhead of recording/interception

## Files to Reference

### Core Implementation
- `shadowcat-wassette/src/transport/wassette.rs` - Current Wassette transport
- `shadowcat-wassette/src/recorder/` - Recording infrastructure
- `shadowcat-wassette/src/interceptor/` - Interception framework

### Planning Documents
- `plans/wassette-integration/wassette-tracker.md` - Overall progress tracking
- `plans/wassette-integration/analysis/phase-c-implementation.md` - Current implementation status
- `plans/wassette-integration/analysis/proxy-architecture.md` - Architecture design

### Test Code
- `shadowcat-wassette/tests/wassette_integration_test.rs` - Existing tests
- `shadowcat-wassette/tests/integration/` - Integration test framework

## Commands to Start

```bash
# Navigate to the shadowcat-wassette worktree
cd shadowcat-wassette

# Ensure on the feat/wassette-integration branch
git status

# Run existing tests to verify baseline
cargo test wassette

# Start implementing recording integration
$EDITOR src/transport/wassette.rs
```

## Definition of Done

Phase C is complete when:
- [ ] Recording captures all Wassette traffic with metadata
- [ ] Recorded sessions can be successfully replayed
- [ ] Interceptors can modify/block Wassette messages
- [ ] Token stripping prevents credential leakage
- [ ] All tests pass including integration tests
- [ ] Code passes clippy and rustfmt checks
- [ ] Documentation updated with examples

After Phase C completion, we'll move to Phase D for final documentation, performance analysis, and security assessment before considering the Wassette integration production-ready.