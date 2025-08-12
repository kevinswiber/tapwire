# Next Session Prompt

## Project Context

You are working on **Shadowcat**, a high-performance MCP (Model Context Protocol) proxy written in Rust. This is part of the larger Tapwire platform vision for MCP inspection, recording, and observability.

**Current Status**: Phase 0-5.5 ✅ COMPLETE, Ready for Phase 6 (Replay)

## Recent Achievements

### Phase 5: MCP-Aware Recorder (Completed 2025-01-13)
Successfully implemented all recording functionality:

1. **C.1-C.2: MCP Tape Format & Session Recorder** ✅
   - Created unified tape format with MessageEnvelope
   - Implemented SessionRecorder with buffer management
   - Thread-safe concurrent recording

2. **C.3: Storage Backend** ✅
   - Implemented `save_tape`, `load_tape`, `delete_tape` methods in TapeStorage
   - Added `export_tape` and `import_tape` for tape portability
   - Integrated TapeStorage into TapeRecorder
   - Automatic storage initialization on recorder startup

3. **C.4: SSE Recording Integration** ✅
   - Recording already integrated at ForwardProxy level
   - Works with StreamableHttp transport (SSE + HTTP)
   - Automatic frame recording through MessageProcessors

4. **C.5: Reverse Proxy Recording** ✅
   - Added TapeRecorder to reverse proxy AppState
   - Integrated recording in handle_mcp_request and SSE handlers
   - Start recording on new session creation
   - Records both client→server and server→client messages
   - Added recording configuration to ReverseProxyConfig

### Phase 5.5: Recorder Consolidation (Completed 2025-01-13)
Successfully unified the recording system:
- Migrated from dual tape formats to single unified format
- No backward compatibility needed (pre-release advantage)
- All tests passing with new structure

## Current Architecture

```rust
// Storage Integration
pub struct TapeRecorder {
    storage: Arc<RwLock<TapeStorage>>,
    active_tapes: Arc<RwLock<HashMap<SessionId, Tape>>>,
    frame_buffer: Arc<RwLock<HashMap<SessionId, Vec<MessageEnvelope>>>>,
}

// Reverse Proxy Integration
struct AppState {
    session_manager: Arc<SessionManager>,
    // ... other fields ...
    tape_recorder: Option<Arc<TapeRecorder>>,
}
```

## Completed Phases Summary

According to @plans/proxy-sse-message-tracker.md:
- ✅ Phase 0: Foundation Components (11 hours)
- ✅ Phase 1: SSE Transport with MCP Awareness (12 hours)
- ✅ Phase 2: Reverse Proxy Streamable HTTP (12 hours)
- ✅ Phase 3: Full MCP Parser and Correlation (16 hours)
- ✅ Phase 4: MCP-Aware Interceptor (17 hours)
- ✅ Phase 5: MCP-Aware Recorder (16 hours)
- ✅ Phase 5.5: Recorder Consolidation (16 hours, completed in ~3 hours)

**Total Completed**: 100 hours of implementation

## Next Tasks: Phase 6 - MCP-Aware Replay (15 hours total)

All tasks are unblocked and ready:

### P.1: Replay Engine Core (5h)
- Create core replay functionality
- Load tapes from storage
- Process frames in sequence
- Handle timing and delays

### P.2: Replay Controller (4h)
- Playback control system (play, pause, stop, seek)
- Speed control (1x, 2x, 0.5x, etc.)
- Frame stepping
- Breakpoint support

### P.3: Message Transformations (3h)
- Transform messages during replay
- Update timestamps
- Modify session IDs if needed
- Apply rules for modification

### P.4: SSE Replay Support (3h)
- SSE-specific replay features
- Stream reconstruction
- Event ID management
- Connection handling

## Phase 7: Testing and Integration (22 hours total)

Ready when Phase 6 completes:
- T.1-T.3: Integration tests (8h)
- T.4-T.7: Component tests (10h)
- T.8: Performance benchmarks (4h)

## Key Technical Context

### Recording System Features
- Filesystem-based storage with JSON format
- Index-based tape management
- Automatic session recording in both proxies
- Buffer-based frame collection for efficiency
- Thread-safe concurrent recording

### Performance Considerations
- Buffer limit of 1000 frames before flush
- Async background storage operations
- No blocking on recording operations
- Graceful failure handling (warn, don't error)

## Development Guidelines

### Code Quality Standards
```bash
# Before ANY commit, run:
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

### Testing Recording
```bash
# Test forward proxy recording
cargo run -- forward stdio --record ./recordings -- echo '{"jsonrpc":"2.0","method":"initialize","id":1}'

# Test reverse proxy recording (add to config)
enable_recording = true
recording_dir = "./recordings"

# Check recordings
cargo run -- tape list
cargo run -- tape info <tape-id>
```

## Quick Start Commands

```bash
# Navigate to shadowcat
cd shadowcat

# Run all tests
cargo test

# Test recording specifically
cargo test recorder::

# List recorded tapes
ls -la ./recordings/
```

## Session Focus

For the next session, focus on implementing Phase 6 (Replay System). Start with:

1. **P.1: Replay Engine Core** - The foundation for all replay functionality
2. **P.2: Replay Controller** - User control over playback
3. **P.3: Message Transformations** - Ability to modify during replay
4. **P.4: SSE Replay Support** - Handle SSE-specific requirements

The replay system should:
- Load tapes from TapeStorage
- Process frames with proper timing
- Support various playback speeds
- Allow message transformation
- Handle both stdio and SSE transports

## Additional Context

### Storage Provider System (Planning Phase)
A new initiative has been documented in @plans/tape-storage-providers/ to create a plugin-based storage backend system. This will allow users to provide custom storage implementations (S3, PostgreSQL, etc.) beyond the built-in filesystem storage. 

**Key Design Decisions**:
- No backward compatibility needed (Shadowcat is pre-release)
- Clean API design without legacy constraints
- Registry-based plugin system
- Runtime provider registration

This is documented but not yet implemented. Focus should remain on Phase 6 (Replay) first.

## Important Notes

- Recording is now fully integrated in both forward and reverse proxies
- Storage system handles persistence automatically
- All architectural refactoring is complete
- No backward compatibility constraints (pre-release advantage)
- Focus on replay implementation for Phase 6
- Maintain test coverage for new features

## Key Files Modified Recently

- `src/recorder/storage.rs` - Enhanced with save/load/delete/export/import methods
- `src/recorder/tape.rs` - Integrated TapeStorage, automatic initialization
- `src/proxy/reverse.rs` - Added tape_recorder to AppState
- `src/proxy/reverse/types.rs` - Added recording configuration fields
- `tests/integration/e2e_framework.rs` - Fixed test compilation

Refer to @plans/proxy-sse-message-tracker.md for the complete task breakdown and dependencies.