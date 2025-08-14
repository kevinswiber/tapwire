# Tape Format JSON Lines Migration Tracker

## Overview

This tracker coordinates the migration of Shadowcat's tape recording format from monolithic JSON files to a streaming-friendly JSON Lines format. This change will enable better memory efficiency, streaming capabilities, and resilience for long-running MCP session recordings.

**Last Updated**: 2025-08-14  
**Total Estimated Duration**: 16-24 hours  
**Status**: Planning

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

**Note for NEXT_SESSION_PROMPT.md**: Always include a reminder that work must be done in the `shadowcat-tape-format-json-lines` worktree directory, NOT in the main shadowcat directory.

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
| 1.1 | **Format Specification** | 2h | None | ⬜ Not Started | | [Details](tasks/1.1-format-specification.md) |
| 1.2 | **Performance Analysis** | 3h | None | ⬜ Not Started | | [Details](tasks/1.2-performance-analysis.md) |
| 1.3 | **Migration Strategy** | 2h | 1.1 | ⬜ Not Started | | [Details](tasks/1.3-migration-strategy.md) |
| 1.4 | **API Design** | 2h | 1.1 | ⬜ Not Started | | [Details](tasks/1.4-api-design.md) |

**Phase 1 Total**: 9 hours

### Phase 2: Core Implementation (Week 1-2)
Implement the JSON Lines tape format with streaming capabilities

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| 2.1 | **Streaming Writer** | 4h | 1.1, 1.4 | ⬜ Not Started | | [Details](tasks/2.1-streaming-writer.md) |
| 2.2 | **Streaming Reader** | 4h | 1.1, 1.4 | ⬜ Not Started | | [Details](tasks/2.2-streaming-reader.md) |
| 2.3 | **Index Enhancement** | 3h | 2.1 | ⬜ Not Started | | [Details](tasks/2.3-index-enhancement.md) |
| 2.4 | **Seek Capability** | 2h | 2.2, 2.3 | ⬜ Not Started | | [Details](tasks/2.4-seek-capability.md) |

**Phase 2 Total**: 13 hours

### Phase 3: Migration & Compatibility (Week 2)
Ensure smooth transition from existing format

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| 3.1 | **Migration Tool** | 3h | 2.1, 2.2 | ⬜ Not Started | | [Details](tasks/3.1-migration-tool.md) |
| 3.2 | **Backward Compatibility** | 2h | 3.1 | ⬜ Not Started | | [Details](tasks/3.2-backward-compatibility.md) |
| 3.3 | **Testing & Validation** | 3h | All Phase 2 | ⬜ Not Started | | [Details](tasks/3.3-testing-validation.md) |

**Phase 3 Total**: 8 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (TBD)
- [ ] 1.1: Format Specification
- [ ] 1.2: Performance Analysis
- [ ] 1.3: Migration Strategy
- [ ] 1.4: API Design
- [ ] 2.1: Streaming Writer (start)

### Week 2 (TBD)
- [ ] 2.1: Streaming Writer (complete)
- [ ] 2.2: Streaming Reader
- [ ] 2.3: Index Enhancement
- [ ] 2.4: Seek Capability
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
{"type": "header", "version": "2.0", "tape_id": "...", "session_id": "...", "created_at": "...", "protocol_version": "..."}
{"type": "frame", "sequence": 0, "timestamp": 0, "envelope": {...}, "metadata": {...}}
{"type": "frame", "sequence": 1, "timestamp": 100, "envelope": {...}, "metadata": {...}}
{"type": "correlation", "id": "...", "request_seq": 0, "response_seq": 1, "rtt_ms": 100}
{"type": "footer", "stats": {...}, "finalized_at": "...", "frame_count": 2, "duration_ms": 100}
```

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
5. **Handoff** (10 min): Update NEXT_SESSION_PROMPT.md if needed

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

1. **Create format specification document** - Define exact JSON Lines schema
2. **Benchmark current implementation** - Establish performance baseline
3. **Design streaming API** - Define new TapeWriter/TapeReader interfaces

## Notes

- Current implementation loads entire tapes into memory, limiting recording duration
- JSON Lines format is widely supported by data processing tools
- Consider compatibility with existing tape analysis tools
- Migration must be optional initially to allow gradual adoption
- Performance testing should include both small (<1MB) and large (>1GB) tapes

---

**Document Version**: 1.0  
**Created**: 2025-08-13  
**Last Modified**: 2025-08-13  
**Author**: Shadowcat Team

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-13 | 1.0 | Initial tracker creation | Shadowcat Team |