# Task R.1: Implement StdioRawTransport

## Objective
Implement the raw stdio transport that handles byte-level I/O for stdin/stdout without any protocol knowledge.

## Key Requirements
1. Implement the `RawTransport` trait
2. Handle stdin reading and stdout writing
3. Support both incoming (read stdin) and outgoing (subprocess) modes
4. Proper buffering and line-based framing
5. No JSON-RPC or MCP knowledge

## Implementation Steps

### 1. Create StdioRawIncoming
For reading from the current process's stdin:
```rust
pub struct StdioRawIncoming {
    stdin_rx: mpsc::Receiver<Vec<u8>>,
    stdout_tx: mpsc::Sender<Vec<u8>>,
    connected: bool,
}
```

### 2. Create StdioRawOutgoing  
For subprocess I/O (replaces process spawning in StdioTransport):
```rust
pub struct StdioRawOutgoing {
    process_handle: ProcessHandle,
    stdin_tx: mpsc::Sender<Vec<u8>>,
    stdout_rx: mpsc::Receiver<Vec<u8>>,
    connected: bool,
}
```

### 3. Key Implementation Details
- Use tokio channels for async I/O
- Line-based framing (messages end with \n)
- Buffer management with configurable size
- Graceful shutdown handling

## Testing Requirements
- Unit tests for both incoming and outgoing modes
- Test buffering and framing
- Test connection lifecycle
- Test error handling

## Success Criteria
- [ ] Implements RawTransport trait
- [ ] Handles stdin/stdout correctly
- [ ] Proper async I/O with tokio
- [ ] Line-based framing works
- [ ] All tests pass
- [ ] No protocol knowledge in implementation

## Files to Create/Modify
- `src/transport/raw/stdio.rs` - New implementation
- `src/transport/raw/mod.rs` - Export new types
- `tests/raw_transport_tests.rs` - Unit tests

## Dependencies
- Requires Phase 1 foundation (✅ Complete)
- ProcessManager for subprocess mode

## Estimated Duration: 3 hours