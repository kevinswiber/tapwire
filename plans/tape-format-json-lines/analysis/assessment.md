# Tape Format Assessment: JSON vs JSON Lines

## Executive Summary

After analyzing Shadowcat's tape recording mechanism, we recommend migrating from monolithic JSON files to JSON Lines format for improved streaming, memory efficiency, and resilience. This document provides a detailed assessment of both formats and proposes a hybrid implementation approach.

## Current Implementation Analysis

### Format Overview
- **Storage**: Single JSON file per tape (`{tape_id}.json`)
- **Structure**: Monolithic `Tape` object containing all frames in memory
- **Operations**: Full file read/write for all operations
- **Index**: Separate `index.json` maintains tape metadata

### Code Structure
```rust
// Current tape structure (src/recorder/tape.rs)
pub struct Tape {
    pub id: TapeId,
    pub session_id: SessionId,
    pub created_at: DateTime<Utc>,
    pub protocol_version: ProtocolVersion,
    pub frames: Vec<TapeFrame>,        // All frames in memory
    pub metadata: TapeMetadata,
    pub correlations: Vec<CorrelationRecord>,
    pub compressed: bool,
}
```

### Current Storage Flow
```rust
// Save operation (src/recorder/storage.rs:335-362)
pub async fn save_tape(&mut self, tape: &Tape) -> RecorderResult<PathBuf> {
    // Serialize entire tape to JSON
    let json_data = serde_json::to_string_pretty(tape)?;
    // Write complete file
    fs::write(&file_path, &json_data).await?;
}

// Load operation (src/recorder/storage.rs:687-697)
pub async fn load_tape_from_file(&self, path: &Path) -> RecorderResult<Tape> {
    let json_data = fs::read_to_string(path).await?;
    let tape: Tape = serde_json::from_str(&json_data)?;
    Ok(tape)
}
```

## JSON Lines Format Benefits

### 1. Streaming & Memory Efficiency
- **Current**: Entire tape must fit in memory
- **JSON Lines**: Process frames one at a time
- **Impact**: Can handle unlimited tape sizes with constant memory usage

### 2. Append Performance
- **Current**: O(n) - Must rewrite entire file for each frame
- **JSON Lines**: O(1) - Simply append new line
- **Impact**: Dramatically faster recording for long sessions

### 3. Resilience & Recovery
- **Current**: Single corrupt byte can invalidate entire tape
- **JSON Lines**: Only affected lines are lost
- **Impact**: Better data recovery in crash scenarios

### 4. Processing Flexibility
- **Current**: Custom parsers required
- **JSON Lines**: Standard UNIX tools work (`grep`, `tail`, `wc -l`)
- **Impact**: Easier debugging and analysis

### 5. Real-time Streaming
- **Current**: Must wait for complete tape
- **JSON Lines**: Can tail and process in real-time
- **Impact**: Live monitoring of active recordings

## Current JSON Format Benefits

### 1. Atomic Operations
- Single file read/write is inherently atomic
- Easier transactional guarantees
- Simpler file management

### 2. Rich Structure
- Metadata, frames, and correlations in one cohesive structure
- Easy to validate completeness
- Self-contained format

### 3. Implementation Simplicity
- Standard serde serialization
- No custom streaming logic needed
- Well-understood by developers

## Recommended Approach: Hybrid JSON Lines

### Format Design

```jsonl
// Line 1: Header with metadata
{"type": "header", "version": "2.0", "tape_id": "uuid", "session_id": "uuid", "created_at": "2024-01-01T00:00:00Z", "protocol_version": "2025-11-05"}

// Lines 2-N: Frame data
{"type": "frame", "sequence": 0, "timestamp": 0, "envelope": {...}, "interceptor_action": null, "transport_metadata": {...}}
{"type": "frame", "sequence": 1, "timestamp": 100, "envelope": {...}, "interceptor_action": null, "transport_metadata": {...}}

// Correlation records (can appear anywhere)
{"type": "correlation", "id": "corr-1", "request_seq": 0, "response_seq": 1, "rtt_ms": 100}

// Final line: Footer with statistics
{"type": "footer", "stats": {...}, "correlations_count": 1, "finalized_at": "2024-01-01T00:01:00Z"}
```

### Implementation Strategy

#### Phase 1: Core Components
1. **TapeJsonLinesWriter**: Streaming writer with buffering
2. **TapeJsonLinesReader**: Iterator-based reader
3. **Format converter**: JSON ↔ JSON Lines migration

#### Phase 2: Enhanced Features
1. **Index with byte offsets**: Enable seeking to specific frames
2. **Compression per line**: Optional zstd/lz4 compression
3. **Recovery mode**: Skip corrupted lines

#### Phase 3: Integration
1. **Backward compatibility**: Support both formats
2. **Gradual migration**: Convert on access
3. **Performance validation**: Benchmark both formats

## Performance Comparison

### Memory Usage
| Scenario | Current JSON | JSON Lines |
|----------|-------------|------------|
| 1K frames | ~10MB | ~1MB |
| 100K frames | ~1GB | ~1MB |
| 1M frames | ~10GB | ~1MB |

### Operation Performance
| Operation | Current JSON | JSON Lines |
|-----------|-------------|------------|
| Append frame | O(n) | O(1) |
| Load tape | O(n) | O(1) for header |
| Seek to frame | O(1) in memory | O(log n) with index |
| Full replay | O(n) | O(n) streaming |

## Migration Path

### Step 1: Dual Format Support
```rust
enum TapeFormat {
    Json(Tape),
    JsonLines(TapeJsonLines),
}
```

### Step 2: Automatic Migration
- Detect format on load
- Convert to JSON Lines on next save
- Keep original as backup

### Step 3: Tooling Updates
- Update CLI commands
- Provide conversion utilities
- Update documentation

## Risk Analysis

### Potential Issues
1. **Line size limits**: Very large frames might exceed line buffers
2. **Atomic writes**: Need careful handling for consistency
3. **Index synchronization**: Index must stay in sync with tape file
4. **Tool compatibility**: Some tools may expect monolithic JSON

### Mitigations
1. **Chunked frames**: Split large frames across multiple lines
2. **Write-ahead logging**: Use WAL pattern for atomicity
3. **Index validation**: Rebuild index from tape if needed
4. **Format detection**: Auto-detect and handle both formats

## Recommendation

**Implement JSON Lines format with the following priorities:**

1. **Immediate**: Design and implement streaming writer/reader
2. **Short-term**: Add backward compatibility and migration tools
3. **Long-term**: Optimize with indexing and compression

This approach provides:
- ✅ Unlimited tape sizes with constant memory
- ✅ Real-time streaming capabilities
- ✅ Better crash resilience
- ✅ Backward compatibility
- ✅ Standard tooling support

The investment is justified by the significant improvements in scalability, performance, and operational flexibility for long-running MCP proxy sessions.

## Appendix: Size Estimates

### Typical Frame Sizes
Based on analysis of the `TapeFrame` structure:

```rust
pub struct TapeFrame {
    pub sequence: u64,                    // 8 bytes
    pub timestamp: Duration,               // 16 bytes
    pub envelope: MessageEnvelope,         // Variable (typically 500-5000 bytes)
    pub interceptor_action: Option<...>,   // 0-200 bytes
    pub transport_metadata: ...,           // 50-500 bytes
    pub correlation_id: Option<String>,    // 0-50 bytes
    pub flags: FrameFlags,                 // 20-100 bytes
}
```

**Average frame size**: 1-5 KB serialized to JSON

### Storage Projections
| Recording Duration | Frame Rate | Frame Count | Current JSON | JSON Lines |
|-------------------|------------|-------------|--------------|------------|
| 1 hour | 10/sec | 36,000 | ~180MB | ~180MB |
| 8 hours | 10/sec | 288,000 | ~1.4GB | ~1.4GB |
| 24 hours | 10/sec | 864,000 | ~4.3GB | ~4.3GB |

**Key difference**: JSON Lines can be processed without loading the full 4.3GB into memory.

## References

- [JSON Lines Specification](https://jsonlines.org/)
- [Shadowcat Tape Implementation](../../shadowcat/src/recorder/tape.rs)
- [MCP Protocol Specification](https://spec.modelcontextprotocol.io)