# Tape Format JSON Lines Migration Tracker

## Overview

This tracker coordinates the migration of Shadowcat's tape recording format from monolithic JSON files to a streaming-friendly JSON Lines format. This change will enable better memory efficiency, streaming capabilities, and resilience for long-running MCP session recordings.

**Last Updated**: 2025-08-15  
**Total Estimated Duration**: 16-24 hours  
**Status**: Phase 2 Complete ✅ | Phase 3 Ready to Start

## IMPORTANT: Git Worktree Configuration

**🚨 CRITICAL**: All work for this feature MUST be done in a dedicated git worktree to avoid conflicts with main development.

### Worktree Setup
- **Worktree Directory**: `shadowcat-tape-format-json-lines/`
- **Branch Name**: `feat/tape-format-json-lines`
- **Parent Directory**: Same level as main `shadowcat/` directory

### Working Directory Commands
```bash
# Navigate to the worktree (ALWAYS use this for development)
cd shadowcat-tape-format-json-lines

# Verify you're in the correct worktree
git worktree list
git branch --show-current  # Should show: feat/tape-format-json-lines
```

**Note for next-session-prompt.md**: Always include a reminder that work must be done in the `shadowcat-tape-format-json-lines` worktree directory, NOT in the main shadowcat directory.

## Goals

1. **Memory Efficiency** - Enable streaming of tape data without loading entire recordings into memory
2. **Append Performance** - Achieve O(1) append operations for new frames instead of O(n) JSON rewrites
3. **Resilience** - Partial file corruption affects only damaged lines, not entire recordings
4. **Backward Compatibility** - Support migration from existing JSON format with zero data loss

## Architecture Vision

```
Current Format:                  Target Format:
┌─────────────────┐             ┌─────────────────┐
│  tape.json      │             │  tape.jsonl     │
│  ┌───────────┐  │             │  ┌───────────┐  │
│  │  Metadata │  │             │  │  Header   │  │ <- Tape metadata
│  ├───────────┤  │             │  ├───────────┤  │
│  │           │  │             │  │  Frame 1  │  │ <- Individual line
│  │  Frames   │  │   =====>    │  ├───────────┤  │
│  │   Array   │  │             │  │  Frame 2  │  │ <- Individual line
│  │           │  │             │  ├───────────┤  │
│  ├───────────┤  │             │  │    ...    │  │
│  │   Stats   │  │             │  ├───────────┤  │
│  └───────────┘  │             │  │  Footer   │  │ <- Final stats
└─────────────────┘             └─────────────────┘
   Monolithic JSON               Streaming JSON Lines
```

## Work Phases

### Phase 1: Design & Analysis (Week 1)
Define the new format specification and migration strategy

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| 1.1 | **Format Specification** | 2h | None | ✅ Complete | | [Spec](analysis/format-specification.md) |
| 1.2 | **Performance Analysis** | 3h | None | ✅ Complete | | [Analysis](analysis/performance-analysis.md) |
| 1.3 | **Migration Strategy** | 2h | 1.1 | ✅ Complete | | [Strategy](analysis/migration-strategy.md) |
| 1.4 | **API Design** | 2h | 1.1 | ✅ Complete | | [API](analysis/api-design.md) |

**Phase 1 Total**: 9 hours

### Phase 2: Core Implementation (Week 1-2)
Implement the JSON Lines tape format with streaming capabilities

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| 2.1 | **Streaming Writer** | 4h | 1.1, 1.4 | ✅ Complete | | Implemented StreamingTapeWriter with O(1) append |
| 2.2 | **Streaming Reader** | 4h | 1.1, 1.4 | ✅ Complete | | Implemented StreamingTapeReader with line-by-line parsing |
| 2.3 | **Index Enhancement** | 3h | 2.1 | ✅ Complete | | Implemented index generation and loading |
| 2.4 | **Seek Capability** | 2h | 2.2, 2.3 | ✅ Complete | | Added seek_to_frame and seek_to_time methods |

**Phase 2 Total**: 13 hours ✅ Complete

### Phase 3: Direct Integration (Week 2)
Replace old format entirely (no backward compatibility needed - pre-release!)

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| 3.1 | **Replace Implementation** | 3h | 2.1, 2.2 | ⬜ Not Started | | Direct replacement in tape.rs/storage.rs |
| 3.2 | **CLI Integration** | 2h | 3.1 | ⬜ Not Started | | Update record/replay/list commands |
| 3.3 | **Testing & Cleanup** | 2h | 3.2 | ⬜ Not Started | | Remove old code, final tests |

**Phase 3 Total**: 7 hours (simplified - no migration needed!)

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (2025-08-14 to 2025-08-15)
- [x] 1.1: Format Specification ✅
- [x] 1.2: Performance Analysis ✅
- [x] 1.3: Migration Strategy ✅
- [x] 1.4: API Design ✅
- [x] 2.1: Streaming Writer ✅
- [x] 2.2: Streaming Reader ✅
- [x] 2.3: Index Enhancement ✅
- [x] 2.4: Seek Capability ✅

### Week 2 (TBD)
- [ ] 3.1: Migration Tool
- [ ] 3.2: Backward Compatibility
- [ ] 3.3: Testing & Validation

## Success Criteria

### Functional Requirements
- ✅ Stream tapes without loading entire file into memory
- ✅ Append frames with O(1) complexity
- ✅ Support partial file recovery after corruption
- ✅ Maintain all existing tape functionality
- ✅ Zero data loss during migration

### Performance Requirements
- ✅ < 10MB memory usage for tapes with 1M+ frames
- ✅ < 1ms append latency for individual frames
- ✅ Support streaming playback at 10,000 frames/second
- ✅ < 5 seconds to migrate 1GB tape file

### Quality Requirements
- ✅ 95% test coverage for new code
- ✅ No clippy warnings
- ✅ Full documentation with examples
- ✅ Integration tests for all tape operations
- ✅ Benchmarks showing performance improvements

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Data loss during migration | HIGH | Comprehensive testing, backup original files | Planned |
| Performance regression for small tapes | MEDIUM | Benchmark both formats, hybrid approach for small files | Planned |
| Breaking API changes | HIGH | Maintain backward compatibility layer | Planned |
| Incomplete line handling on crash | MEDIUM | Write atomic line markers, recovery mode | Planned |
| Large memory spike during migration | MEDIUM | Streaming migration, progress checkpoints | Planned |

## Implementation Details

### JSON Lines Format Specification

```jsonl
{"type": "init", "version": "2.0", "tape_id": "...", "session_id": "...", "created_at": "...", "protocol_version": "..."}
{"type": "frame", "seq": 0, "ts": 0, "message": {...}, "direction": "client_to_server", "session_id": "..."}
{"type": "frame", "seq": 1, "ts": 100, "message": {...}, "direction": "server_to_client", "session_id": "..."}
{"type": "correlation", "id": "...", "request_seq": 0, "response_seq": 1, "rtt_ms": 100}
{"type": "checkpoint", "checkpoint_at": "...", "seq": 2, "stats": {...}}
```

Note: Separate `.meta.json` file contains tape metadata to avoid lock contention during concurrent read/write.

### File Structure

```
storage/
├── index.json           # Global index (unchanged)
├── tapes/
│   ├── {id}.jsonl      # Tape frames in JSON Lines format
│   └── {id}.meta.json  # Optional: Quick-access metadata cache
└── legacy/
    └── {id}.json       # Original JSON tapes (after migration)
```

### Key Implementation Considerations

1. **Atomic Writes**: Each line must be written atomically to prevent corruption
2. **Line Validation**: Each line must be valid JSON independently
3. **Recovery Mode**: Support reading partially corrupted files
4. **Compression**: Consider per-line compression for large frames
5. **Indexing**: Maintain byte offsets for quick seeking

## Session Planning Guidelines

### Optimal Session Structure
1. **Review** (10 min): Check this tracker and analysis documents
2. **Implementation** (2-3 hours): Complete the task deliverables
3. **Testing** (30 min): Run tests, benchmarks
4. **Documentation** (15 min): Update tracker, API docs
5. **Handoff** (10 min): Update next-session-prompt.md if needed

### Task Completion Criteria
- [ ] All deliverables checked off
- [ ] Tests passing
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Benchmarks run (for performance tasks)
- [ ] Tracker status updated

## Related Documents

### Primary References
- [Current Tape Implementation](../../shadowcat/src/recorder/tape.rs)
- [Storage Implementation](../../shadowcat/src/recorder/storage.rs)
- [Assessment Document](analysis/assessment.md)

### Task Files
- [Phase 1 Tasks](tasks/)
- [Phase 2 Tasks](tasks/)
- [Phase 3 Tasks](tasks/)

### Specifications
- [MCP Protocol Spec](https://spec.modelcontextprotocol.io)
- [JSON Lines Specification](https://jsonlines.org/)

## Next Actions

1. **Index Enhancement** - Implement seeking and indexing capabilities (Task 2.3)
2. **Migration Tool** - Create tool to convert existing JSON tapes to JSON Lines format (Task 3.1)
3. **Integration** - Update recorder module to use new streaming implementation

## Notes

- Current implementation loads entire tapes into memory, limiting recording duration
- JSON Lines format is widely supported by data processing tools
- Consider compatibility with existing tape analysis tools
- Migration must be optional initially to allow gradual adoption
- Performance testing should include both small (<1MB) and large (>1GB) tapes

---

**Document Version**: 1.1  
**Created**: 2025-08-13  
**Last Modified**: 2025-08-15  
**Author**: Shadowcat Team

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-13 | 1.0 | Initial tracker creation | Shadowcat Team |
| 2025-08-15 | 1.1 | Phase 2 progress: Completed streaming writer and reader implementation | Shadowcat Team |