# Shadowcat Phase 3 Implementation Plan

**Project:** Shadowcat Phase 3 - Recording & Replay Engine  
**Timeline:** Weeks 3-4 (August 5-18, 2025)  
**Status:** Planning

---

## Overview

Phase 3 focuses on building a comprehensive recording and replay system that enables deterministic playback of MCP sessions. This includes a powerful tape replay engine, CLI management tools, enhanced tape formats, and storage optimizations.

---

## High Priority Tasks

### 1. Tape Replay Engine
**File:** `src/recorder/replay.rs`  
**Details:** [tasks/005-tape-replay.md](tasks/005-tape-replay.md)  
**Estimated Effort:** 2 days

**Core Features:**
- Implement `TapePlayer` struct for deterministic replay
- Support variable speed control (0.1x to 10x speed)
- Add pause/resume functionality with state preservation
- Frame-by-frame stepping for debugging
- Replay position tracking and seeking

**Technical Requirements:**
- Accurate timing reproduction using frame timestamps
- State management for pause/resume operations
- Event callbacks for replay progress monitoring
- Memory-efficient streaming for large tapes
- Error handling for corrupted or incompatible tapes

### 2. CLI Tape Management
**File:** `src/cli/tape.rs`  
**Details:** [tasks/006-tape-cli.md](tasks/006-tape-cli.md)  
**Estimated Effort:** 1.5 days

**Commands to Implement:**
```bash
shadowcat tape list [--format=table|json] [--filter=<criteria>]
shadowcat tape show <tape-id> [--frames] [--stats]
shadowcat tape replay <tape-id> [--speed=1.0] [--from=<frame>] [--to=<frame>]
shadowcat tape delete <tape-id> [--confirm]
shadowcat tape export <tape-id> [--format=har|postman|curl]
shadowcat tape validate <tape-id>
shadowcat tape compress <tape-id>
```

**Features:**
- Rich output formatting with colored tables
- Interactive confirmation for destructive operations
- Progress bars for long-running operations
- Comprehensive error messages and help text
- Auto-completion support preparation

### 3. Enhanced Tape Format
**File:** `src/recorder/format.rs`  
**Details:** [tasks/007-tape-format.md](tasks/007-tape-format.md)  
**Estimated Effort:** 2 days

**Format Enhancements:**
```json
{
  "version": "1.0",
  "format_version": 1,
  "checksum": "sha256-hash",
  "compression": "gzip|none",
  "environment": {
    "shadowcat_version": "0.1.0",
    "platform": "darwin-arm64",
    "mcp_version": "2025-11-05",
    "recorded_at": "2025-08-04T10:30:00Z"
  },
  "metadata": {
    "id": "uuid",
    "name": "string",
    "description": "optional_string",
    "tags": ["array"],
    "session_info": {
      "client_transport": "stdio|http",
      "server_endpoint": "optional_string",
      "duration_ms": "number"
    }
  },
  "frames": [...],
  "index": {
    "frame_count": "number",
    "request_count": "number", 
    "response_count": "number",
    "notification_count": "number",
    "timeline": [
      {"timestamp": "number", "frame_index": "number"}
    ]
  }
}
```

**Migration & Validation:**
- Automatic migration from v0 format (current) to v1
- Schema validation with detailed error reporting
- Checksum verification for integrity checking
- Compression support for storage efficiency

---

## Medium Priority Tasks

### 4. Replay Transport
**File:** `src/transport/replay.rs`  
**Details:** [tasks/008-replay-transport.md](tasks/008-replay-transport.md)  
**Estimated Effort:** 1.5 days

**Transport Implementation:**
- Create `ReplayTransport` implementing the `Transport` trait
- Support both real-time and stepped replay
- Integrate with `TapePlayer` for frame delivery
- Handle transport state consistency during replay
- Enable replay pause/resume at transport level

**Integration Points:**
- Works with existing `ForwardProxy` without modifications
- Supports all existing Transport trait methods
- Maintains timing accuracy for realistic replay
- Handles connection state simulation

### 5. Storage Optimization
**Files:** `src/recorder/storage.rs`, `src/recorder/index.rs`  
**Details:** [tasks/009-storage-optimization.md](tasks/009-storage-optimization.md)  
**Estimated Effort:** 1.5 days

**Indexing System:**
- Fast tape discovery and metadata extraction
- Search by date range, session type, tags
- Statistics aggregation (total sessions, data size, etc.)
- Tape dependency tracking for related sessions

**Storage Backend Options:**
- File-based storage (current, enhanced)
- SQLite backend for metadata and indexing
- Configurable storage policies (TTL, size limits)
- Background cleanup and maintenance tasks

**Performance Features:**
- Lazy loading for large tape files
- Streaming read/write for memory efficiency
- Concurrent access with proper locking
- Cache frequently accessed metadata

---

## Low Priority Tasks

### 6. Advanced Replay Features
**Files:** Various  
**Estimated Effort:** 1 day

**Enhanced Capabilities:**
- Tape comparison and diff utilities
- Replay with modifications (interceptor integration prep)
- Batch replay operations
- Replay performance metrics and analysis

### 7. Export & Integration
**File:** `src/recorder/export.rs`  
**Estimated Effort:** 0.5 days

**Export Formats:**
- HAR (HTTP Archive) format for web tools
- Postman collection format
- cURL command generation
- Custom JSON schema for external tools

---

## Week-by-Week Breakdown

### Week 3: Core Replay Engine
**Days 1-2:** Tape Replay Engine
- Implement `TapePlayer` with basic replay functionality
- Add timing controls and state management
- Write comprehensive tests with mock tapes

**Days 3-4:** CLI Tape Management
- Create tape CLI commands with clap integration
- Implement list, show, and basic replay commands
- Add rich formatting and user experience features

**Day 5:** Enhanced Tape Format
- Design and implement v1 tape format
- Add migration utilities from v0 to v1
- Implement checksum and validation

### Week 4: Integration & Optimization
**Days 1-2:** Replay Transport
- Implement `ReplayTransport` with Transport trait
- Integrate with `TapePlayer` for frame delivery
- Test integration with `ForwardProxy`

**Days 3-4:** Storage Optimization
- Add indexing and search capabilities
- Implement storage policies and cleanup
- Performance testing and optimization

**Day 5:** Polish & Testing
- Integration testing across all components
- Performance benchmarking
- Documentation and examples

---

## Success Criteria

### Functional Requirements
- [ ] Can replay recorded tapes with accurate timing
- [ ] CLI provides comprehensive tape management
- [ ] Enhanced tape format supports versioning and validation
- [ ] ReplayTransport integrates seamlessly with existing proxy
- [ ] Storage system handles large tape collections efficiently

### Performance Requirements
- [ ] Replay accuracy within ±10ms of original timing
- [ ] Can handle tapes with 10,000+ frames
- [ ] CLI operations complete within 500ms for metadata
- [ ] Memory usage stays under 100MB for typical operations
- [ ] Storage indexing supports 1,000+ tapes efficiently

### Quality Requirements
- [ ] Comprehensive test coverage for all new components
- [ ] Integration tests for end-to-end replay scenarios
- [ ] Error handling with helpful user messages
- [ ] Documentation for all public APIs
- [ ] Performance benchmarks established

---

## Technical Decisions

### Replay Timing Strategy
- Use frame timestamps as authoritative timing source
- Support both real-time and accelerated/decelerated playback
- Handle timing drift with periodic synchronization
- Allow manual timing adjustments for debugging

### Storage Architecture
- Hybrid approach: JSON files + SQLite index
- Lazy loading to minimize memory usage
- Separate metadata cache for fast operations
- Configurable storage backends via trait abstraction

### CLI Design Philosophy
- Follow Unix philosophy: do one thing well
- Rich output formatting with sensible defaults
- Interactive confirmations for destructive operations
- Machine-readable output options (JSON, CSV)

---

## Risks and Mitigations

### Risk: Timing Accuracy Challenges
**Impact:** Replay may not accurately represent original session  
**Mitigation:** 
- Implement multiple timing strategies (strict, relaxed, manual)
- Add timing validation and drift detection
- Provide timing adjustment tools for debugging

### Risk: Large Tape Performance
**Impact:** Memory usage and slow operations with large tapes  
**Mitigation:**
- Implement streaming read/write operations
- Add pagination for large tape browsing
- Implement tape compression and indexing
- Set reasonable default limits with override options

### Risk: Storage Complexity
**Impact:** Complex storage logic may introduce bugs  
**Mitigation:**
- Start with simple file-based storage
- Add SQLite backend as optional enhancement
- Comprehensive testing with various tape sizes
- Clear separation between storage interface and implementation

---

## Dependencies

### New Dependencies Needed
```toml
# CLI enhancements
clap_complete = "4.0"  # Auto-completion support
indicatif = "0.17"     # Progress bars
comfy-table = "7.0"    # Rich table formatting

# Compression and validation
flate2 = "1.0"         # Gzip compression
sha2 = "0.10"          # Checksum calculation

# Optional SQLite backend
sqlx = { version = "0.8", features = ["sqlite", "json"] }  # Already present
```

### Internal Dependencies
- All Phase 2 components (proxy, session, transport, recorder)
- CLI framework from Phase 1
- Error handling and logging infrastructure

---

## Definition of Done

Phase 3 is complete when:

1. **Tape Replay Engine**
   - `TapePlayer` can replay any recorded tape with timing accuracy
   - Speed controls (0.1x to 10x) work correctly
   - Pause/resume functionality maintains state properly
   - Frame-by-frame stepping is implemented

2. **CLI Tape Management**
   - All planned tape commands are implemented and tested
   - Rich output formatting provides excellent user experience
   - Error handling gives clear, actionable feedback
   - Help documentation is comprehensive

3. **Enhanced Tape Format**
   - Version 1 format is fully specified and implemented
   - Migration from v0 format works seamlessly
   - Validation and checksum verification are reliable
   - Compression reduces storage requirements significantly

4. **Integration & Testing**
   - ReplayTransport integrates with ForwardProxy
   - All components work together in integration tests
   - Performance meets established criteria
   - Documentation is complete and accurate

5. **Quality Assurance**
   - Test suite covers all new functionality
   - No regressions in existing functionality
   - Code passes all quality checks (fmt, clippy, tests)
   - Performance benchmarks are established and met

---

## Future Considerations

### Phase 4 Preparation
- Replay engine should support interceptor integration
- CLI framework should be extensible for intercept commands
- Storage system should support rule and policy persistence
- Transport abstraction should support modification hooks

### Long-term Scalability
- Consider tape format evolution strategy
- Plan for distributed storage options
- Design plugin architecture for custom exporters
- Consider real-time collaboration features

---

## Conclusion

Phase 3 builds upon the solid foundation of Phase 2 to create a powerful recording and replay system. The focus on user experience through the CLI, combined with robust technical implementation, will make Shadowcat a valuable tool for MCP development and debugging.

The deliverables will enable developers to:
- Record MCP sessions during development or production
- Replay sessions for debugging and testing
- Manage large collections of recorded sessions
- Export sessions to other tools and formats
- Build reliable, reproducible MCP applications

This phase establishes Shadowcat as a comprehensive MCP development platform, setting the stage for advanced interception capabilities in Phase 4.