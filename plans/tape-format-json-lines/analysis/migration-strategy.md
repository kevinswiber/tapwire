# Migration Strategy: Transitioning to JSON Lines Format

## Overview

Since Shadowcat has not been released yet, we have the opportunity for a **clean transition** to the JSON Lines format without backward compatibility concerns. This document outlines the strategy for migrating the codebase and any existing test data.

## Migration Scope

### What Needs to Change

1. **Core Components**:
   - `recorder/tape.rs` - Tape structure and serialization
   - `recorder/storage.rs` - File I/O operations
   - `recorder/mod.rs` - Recording logic

2. **Storage Backend**:
   - `backend/providers/filesystem.rs` - File system operations
   - `backend/providers/memory.rs` - In-memory operations
   - `backend/mod.rs` - Backend trait definitions

3. **CLI Commands**:
   - `record` command - Write JSON Lines format
   - `replay` command - Read JSON Lines format
   - `inspect` command - Display tape information

4. **Tests**:
   - Update test fixtures to JSON Lines format
   - Modify test assertions for streaming behavior
   - Add streaming-specific test cases

## Implementation Strategy

### Phase 1: Core Infrastructure (Current)
- Design JSON Lines format specification ✅
- Analyze performance implications ✅
- Design new streaming APIs (next)

### Phase 2: Implementation
1. **New Modules** (parallel development):
   - `recorder/streaming/writer.rs` - Streaming tape writer
   - `recorder/streaming/reader.rs` - Streaming tape reader
   - `recorder/streaming/metadata.rs` - Metadata file management

2. **Incremental Replacement**:
   ```rust
   // Start with new modules alongside old
   mod tape;  // Existing
   mod streaming;  // New
   
   // Gradually migrate functionality
   // Then remove old modules
   ```

3. **Testing Strategy**:
   - Write tests for new streaming modules first
   - Ensure feature parity with existing tests
   - Add streaming-specific tests (concurrent read, live tailing)

### Phase 3: Cutover
1. Replace old implementation with new
2. Update all CLI commands
3. Convert test fixtures
4. Remove old code

## Development Workflow

### Feature Branch Strategy
```bash
# Current worktree for development
shadowcat-tape-format-json-lines/  # feat/tape-format-json-lines branch

# Development workflow
1. Implement streaming modules
2. Test thoroughly in isolation
3. Integrate with CLI commands
4. Full system testing
5. Merge to main
```

### Testing During Development

```rust
// Parallel testing approach
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_streaming_write() {
        let writer = StreamingTapeWriter::new("test.jsonl");
        // Test streaming writes
    }
    
    #[tokio::test]
    async fn test_concurrent_read_write() {
        // Test reading while writing
    }
    
    #[tokio::test]
    async fn test_live_tailing() {
        // Test following a growing tape
    }
}
```

## Risk Mitigation

### Development Risks

| Risk | Mitigation |
|------|------------|
| Breaking existing tests | Run tests continuously during development |
| Performance regression | Benchmark each component |
| Incomplete feature parity | Checklist of all current features |
| Complex merge conflicts | Small, incremental commits |

### No Production Risk
- No deployed systems to migrate
- No user data to convert
- No backward compatibility needed
- Clean slate advantage

## Conversion Tools (Development Only)

### Test Data Converter
For any existing test recordings in JSON format:

```rust
// Simple converter for test data
pub fn convert_test_fixtures() -> Result<()> {
    for entry in fs::read_dir("tests/fixtures")? {
        let path = entry?.path();
        if path.extension() == Some("json") {
            convert_json_to_jsonl(&path)?;
        }
    }
    Ok(())
}
```

## Implementation Checklist

### Streaming Writer
- [ ] Append-only file operations
- [ ] Init record on first write
- [ ] Frame serialization
- [ ] Correlation tracking
- [ ] Checkpoint generation
- [ ] Metadata file updates
- [ ] Atomic write guarantees
- [ ] Error recovery

### Streaming Reader  
- [ ] Line-by-line parsing
- [ ] Init record validation
- [ ] Frame deserialization
- [ ] Correlation matching
- [ ] Checkpoint processing
- [ ] Live tailing support
- [ ] Concurrent read safety
- [ ] Error tolerance

### Metadata Management
- [ ] Create `.meta.json` files
- [ ] Atomic update strategy
- [ ] Statistics aggregation
- [ ] Status tracking
- [ ] Index maintenance

### CLI Integration
- [ ] Update record command
- [ ] Update replay command
- [ ] Update inspect command
- [ ] Add tail command (new)
- [ ] Progress indicators

### Testing
- [ ] Unit tests for each module
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Stress tests
- [ ] Concurrent access tests

## Success Criteria

### Functional
- ✅ All existing functionality preserved
- ✅ Streaming read/write working
- ✅ Concurrent access supported
- ✅ Live tailing functional
- ✅ Tests passing

### Performance
- ✅ Constant memory usage verified
- ✅ O(1) append time confirmed
- ✅ Instant playback start
- ✅ No performance regressions

### Code Quality
- ✅ No clippy warnings
- ✅ Full documentation
- ✅ 95% test coverage
- ✅ Clean module structure

## Timeline

### Week 1 (Current)
- Day 1-2: Design and analysis ✅
- Day 3-4: Streaming writer implementation
- Day 5: Streaming reader implementation

### Week 2
- Day 1-2: Metadata management
- Day 3: CLI integration
- Day 4: Testing and benchmarks
- Day 5: Code cleanup and documentation

## Rollback Plan

Not needed since:
- No production systems
- No user data
- Development can continue on feature branch
- Main branch unaffected until ready

## Next Steps

1. Complete API design (Task 1.4)
2. Begin streaming writer implementation
3. Set up continuous benchmarking
4. Create test fixtures in JSON Lines format

## Conclusion

The migration to JSON Lines format is straightforward since Shadowcat is pre-release. We can make a clean transition without compatibility concerns, focusing entirely on building the best streaming implementation. The modular approach allows parallel development and thorough testing before replacing the existing code.