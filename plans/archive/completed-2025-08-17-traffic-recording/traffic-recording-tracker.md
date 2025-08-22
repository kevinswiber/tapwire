# Traffic Recording Feature Tracker

## Overview
Implement comprehensive traffic recording functionality for Shadowcat, enabling developers to capture, analyze, and replay MCP sessions for debugging and testing.

## Goals
- [ ] Capture all MCP traffic through the proxy (client↔proxy↔server)
- [ ] Support multiple tape formats (JSONL, binary, compressed)
- [ ] Implement efficient storage with configurable retention
- [ ] Provide replay capabilities for captured sessions
- [ ] Create analysis tools for traffic inspection
- [ ] Ensure minimal performance impact when recording

## Success Criteria
- Recording adds <2% latency overhead to proxy operations
- Support for high-volume traffic (>1000 msg/sec)
- Clean API for starting/stopping recordings
- Tape files compatible with existing MCP tools
- Comprehensive test coverage for recording/replay

## Phases

### Phase A: Analysis & Design (8 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| A.0 | Analyze current recording implementation | 2h | ⬜ | None | |
| A.1 | Research tape format requirements | 2h | ⬜ | A.0 | |
| A.2 | Design storage architecture | 2h | ⬜ | A.1 | |
| A.3 | Create technical specification | 2h | ⬜ | A.0-A.2 | |

### Phase B: Core Implementation (12 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| B.1 | Implement RecordingEngine trait | 3h | ⬜ | A.3 | |
| B.2 | Create JSONL tape writer | 3h | ⬜ | B.1 | |
| B.3 | Add binary format support | 3h | ⬜ | B.1 | |
| B.4 | Implement compression layer | 3h | ⬜ | B.2, B.3 | |

### Phase C: Storage & Management (8 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| C.1 | Create tape storage manager | 3h | ⬜ | B.1 | |
| C.2 | Implement retention policies | 2h | ⬜ | C.1 | |
| C.3 | Add metadata indexing | 3h | ⬜ | C.1 | |

### Phase D: Replay System (10 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| D.1 | Build tape reader/parser | 3h | ⬜ | B.2, B.3 | |
| D.2 | Create replay engine | 4h | ⬜ | D.1 | |
| D.3 | Add timing reconstruction | 3h | ⬜ | D.2 | |

### Phase E: Analysis Tools (8 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| E.1 | Create tape inspection CLI | 3h | ⬜ | D.1 | |
| E.2 | Build traffic statistics analyzer | 2h | ⬜ | E.1 | |
| E.3 | Add filtering and search | 3h | ⬜ | E.1 | |

### Phase F: Integration & Testing (10 hours)
| Task | Description | Duration | Status | Dependencies | Completed |
|------|-------------|----------|--------|--------------|-----------|
| F.1 | Integrate with proxy pipeline | 3h | ⬜ | B.1-B.4 | |
| F.2 | Add CLI commands | 2h | ⬜ | F.1 | |
| F.3 | Create integration tests | 3h | ⬜ | F.1 | |
| F.4 | Performance benchmarking | 2h | ⬜ | F.3 | |

## Total Estimated Time: 56 hours

## Risks & Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| Performance degradation | High | Use async I/O, buffer pooling, optional recording |
| Large tape files | Medium | Implement compression, rotation, retention |
| Format compatibility | Medium | Follow MCP spec, test with official tools |
| Memory overhead | Medium | Stream processing, avoid buffering full sessions |

## Dependencies
- Existing tape module in Shadowcat
- MCP protocol implementation (rmcp)
- Storage backend (SQLite/filesystem)
- Compression libraries (flate2, zstd)

## Key Decisions
- [ ] Primary tape format (JSONL vs binary)
- [ ] Storage backend (filesystem vs database)
- [ ] Compression algorithm (gzip vs zstd vs lz4)
- [ ] Retention strategy (time vs size based)
- [ ] Metadata to capture beyond messages

## Resources
- MCP specification for message formats
- Existing tape.rs implementation
- Performance profiling tools
- Test MCP servers for validation

## Notes
- Current implementation has basic tape recording
- Need to preserve message timing for accurate replay
- Consider privacy/security for sensitive data in recordings
- Potential for distributed recording across multiple proxies

## Progress Log
- 2025-08-22: Plan created, initial structure defined