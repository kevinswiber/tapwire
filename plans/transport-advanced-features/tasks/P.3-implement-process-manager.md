# Task P.3: Implement ProcessManager in SubprocessOutgoing

## Objective
Integrate ProcessManager with SubprocessOutgoing and StdioRawOutgoing to enable enhanced subprocess lifecycle management, monitoring, and graceful shutdown capabilities.

## Duration
2 hours

## Dependencies
- P.1: Subprocess handling analysis (Complete)
- P.2: ProcessManager integration design (Complete)

## Key Questions
1. How to maintain backward compatibility while adding ProcessManager?
2. What's the cleanest way to inject ProcessManager into transports?
3. How to handle I/O channel extraction from managed processes?
4. What error types need to be added for process management failures?

## Implementation Steps

### Step 1: Extend Error Types (15 min)
- [ ] Add ProcessManager-related error variants to TransportError
- [ ] Include ProcessSpawnFailed, ProcessTerminationFailed, ProcessNotFound
- [ ] Add ProcessRecoveryFailed for auto-recovery scenarios

### Step 2: Modify ProcessManager for I/O Access (30 min)
- [ ] Add method to extract Child I/O handles from managed process
- [ ] Ensure SimpleProcessManager can provide stdin/stdout/stderr
- [ ] Maintain ownership model for safe concurrent access

### Step 3: Update SubprocessOutgoing (30 min)
- [ ] Add optional ProcessManager field
- [ ] Add ProcessHandle field for tracking
- [ ] Create `with_process_manager()` constructor
- [ ] Update connect() to use ProcessManager when available
- [ ] Add process_status() and is_healthy() methods

### Step 4: Update StdioRawOutgoing (30 min)
- [ ] Add ProcessManager and managed_handle fields
- [ ] Modify connect() for managed spawning
- [ ] Implement graceful shutdown in close()
- [ ] Add SIGTERM support for Unix systems
- [ ] Update Drop impl for proper cleanup

### Step 5: Write Tests (15 min)
- [ ] Unit test for backward compatibility (no ProcessManager)
- [ ] Unit test for ProcessManager integration
- [ ] Mock ProcessManager for controlled testing
- [ ] Integration test for graceful shutdown

## Success Criteria
- [ ] Existing code works without modification
- [ ] ProcessManager integration is opt-in
- [ ] Graceful shutdown works with configurable timeout
- [ ] All existing tests pass
- [ ] New tests verify ProcessManager integration
- [ ] No clippy warnings

## Deliverables
1. Modified `src/transport/directional/outgoing.rs` with ProcessManager support
2. Modified `src/transport/raw/stdio.rs` with graceful shutdown
3. Updated `src/error.rs` with new error variants
4. Enhanced `src/process/mod.rs` for I/O handle access
5. Unit and integration tests
6. Updated tracker with completion status

## Code Locations
- `shadowcat/src/transport/directional/outgoing.rs` - SubprocessOutgoing
- `shadowcat/src/transport/raw/stdio.rs` - StdioRawOutgoing  
- `shadowcat/src/process/mod.rs` - ProcessManager trait
- `shadowcat/src/error.rs` - Error types

## Testing Commands
```bash
cd shadowcat
cargo test transport::directional::outgoing
cargo test transport::raw::stdio
cargo test process::
cargo clippy --all-targets -- -D warnings
```

## Notes
- Focus on backward compatibility - this is critical
- Start with minimal integration, monitoring can come later
- Graceful shutdown is the key immediate improvement
- Consider using feature flag for ProcessManager if changes are extensive