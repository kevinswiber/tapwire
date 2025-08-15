# JSON Lines Tape Format Specification

## Overview

This document defines the JSON Lines (`.jsonl`) format specification for Shadowcat tape recordings. The format is designed for **streaming append and streaming read operations**, enabling:
- Real-time append of messages without buffering entire tape in memory
- Streaming read/playback without loading entire file
- Resilience to partial corruption
- Concurrent read while writing is in progress

## Core Design Principles

### Streaming-First Architecture
- **Append-Only**: New frames are appended as they arrive, no rewriting needed
- **Stateless Lines**: Each line is self-contained and independently parseable
- **No Required Footer**: Tapes are valid and readable even while recording is in progress
- **Separate Metadata**: Metadata stored in companion file to avoid locking issues

## Format Structure

### File Organization
```
tapes/
├── {tape_id}.jsonl         # The tape data (append-only)
├── {tape_id}.meta.json     # Metadata file (updated periodically)
├── {tape_id}.index         # Optional: byte offset index for seeking
└── index.json              # Directory-level index of all tapes
```

### File Extensions
- `.jsonl` - Tape data file (JSON Lines format)
- `.meta.json` - Metadata companion file
- `.index` - Optional binary index for fast seeking

### Tape Data Structure (.jsonl file)

A tape file consists of sequential JSON objects, one per line:

1. **Initialization Line** (required, first line) - Minimal tape start marker
2. **Frame Lines** (0 or more) - Message frames appended as they arrive
3. **Correlation Lines** (0 or more) - Can be interleaved with frames
4. **Checkpoint Lines** (optional) - Periodic statistics snapshots

## Line Type Specifications

### 1. Initialization Line (init)

The first line of every tape file is a minimal marker that allows immediate streaming append.

```json
{
  "type": "init",
  "version": "2.0",
  "tape_id": "550e8400-e29b-41d4-a716-446655440000",
  "session_id": "7d3a4b8c-9e5f-4a2b-8c1d-3e5f6a7b8c9d",
  "created_at": "2025-08-14T10:30:00.000Z",
  "protocol_version": "2025-11-05"
}
```

**Fields:**
- `type`: Always `"init"` for initialization lines
- `version`: Tape format version (currently "2.0")
- `tape_id`: UUID v4 identifying this tape
- `session_id`: Session identifier from MCP protocol
- `created_at`: ISO 8601 timestamp of tape creation
- `protocol_version`: MCP protocol version used

**Note**: Additional metadata (name, description, tags, etc.) is stored in the companion `.meta.json` file to avoid blocking on writes.

### 2. Frame Line

Each frame represents a single message in the MCP session.

```json
{
  "type": "frame",
  "seq": 0,
  "ts": 0,
  "dir": "client_to_server",
  "env": {
    "message": {
      "jsonrpc": "2.0",
      "method": "initialize",
      "params": {...},
      "id": 1
    },
    "direction": "client_to_server",
    "timestamp": "2025-08-14T10:30:00.123Z",
    "session_id": "7d3a4b8c-9e5f-4a2b-8c1d-3e5f6a7b8c9d"
  },
  "action": null,
  "transport": {
    "stdio": {
      "process_id": 12345,
      "command": "mcp-server"
    }
  },
  "correlation_id": "req-1",
  "flags": {
    "is_error": false,
    "is_notification": false,
    "requires_response": true
  }
}
```

**Fields:**
- `type`: Always `"frame"` for frame lines
- `seq`: Sequence number (monotonically increasing)
- `ts`: Timestamp in milliseconds since tape start
- `dir`: Direction ("client_to_server" or "server_to_client")
- `env`: Message envelope containing the actual MCP message
- `action`: Interceptor action taken (if any)
- `transport`: Transport-specific metadata
- `correlation_id`: Optional ID for request-response correlation
- `flags`: Frame processing flags

**Compressed Format:**
When `compressed: true` in header, the `env.message` field may be base64-encoded compressed data:
```json
{
  "type": "frame",
  "seq": 1,
  "ts": 100,
  "env_compressed": "eJzT0yMAAGTvBe8=",
  ...
}
```

### 3. Correlation Line

Records request-response correlations for performance analysis.

```json
{
  "type": "correlation",
  "id": "corr-1",
  "request_seq": 0,
  "response_seq": 1,
  "request_ts": 0,
  "response_ts": 100,
  "rtt_ms": 100,
  "status": "success"
}
```

**Fields:**
- `type`: Always `"correlation"` for correlation lines
- `id`: Unique correlation identifier
- `request_seq`: Sequence number of request frame
- `response_seq`: Sequence number of response frame
- `request_ts`: Request timestamp (ms)
- `response_ts`: Response timestamp (ms)
- `rtt_ms`: Round-trip time in milliseconds
- `status`: Correlation status ("success", "timeout", "error")

### 4. Checkpoint Line

Optional periodic statistics snapshots that can be written during recording.

```json
{
  "type": "checkpoint",
  "checkpoint_at": "2025-08-14T10:35:00.000Z",
  "seq": 150,
  "stats": {
    "frame_count": 150,
    "duration_ms": 300000,
    "message_counts": {
      "client_to_server": 75,
      "server_to_client": 75
    },
    "error_count": 0,
    "correlation_count": 70
  }
}
```

**Fields:**
- `type`: Always `"checkpoint"` for checkpoint lines
- `checkpoint_at`: ISO 8601 timestamp of checkpoint
- `seq`: Sequence number at checkpoint
- `stats`: Current statistics snapshot

**Note**: Checkpoints are optional and can be written periodically (e.g., every 1000 frames or every 60 seconds) to provide progress information without requiring a footer.

## Companion Metadata File (.meta.json)

Stored separately to avoid lock contention and enable concurrent read/write.

```json
{
  "tape_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Production Debug Session",
  "description": "Debugging authentication flow",
  "tags": ["production", "auth", "debug"],
  "created_at": "2025-08-14T10:30:00.000Z",
  "updated_at": "2025-08-14T10:35:00.000Z",
  "finalized_at": null,
  "status": "recording",
  "transport": {
    "type": "stdio",
    "command": "mcp-server",
    "args": ["--debug"]
  },
  "environment": {
    "platform": "darwin",
    "shadowcat_version": "0.1.0",
    "hostname": "prod-server-1"
  },
  "stats": {
    "frame_count": 150,
    "duration_ms": 300000,
    "file_size_bytes": 524288,
    "last_sequence": 149,
    "message_counts": {
      "client_to_server": 75,
      "server_to_client": 75
    },
    "error_count": 0,
    "correlation_count": 70
  },
  "checksum": null
}
```

### Update Strategy
- Updated periodically (e.g., every 10 seconds or 100 frames)
- Written atomically using write-and-rename
- Can be read while tape is being written
- Contains aggregated statistics from checkpoints

## Writing Guidelines

### Streaming Append Pattern
```rust
// Pseudo-code for streaming writer
let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(tape_path)?;

// Write init line once
if file.is_empty() {
    writeln!(file, "{}", serde_json::to_string(&init_record)?)?;
    file.flush()?;
}

// Append frames as they arrive - NO BUFFERING
while let Some(message) = transport.recv().await {
    let frame = create_frame(message);
    writeln!(file, "{}", serde_json::to_string(&frame)?)?;
    file.flush()?;  // Ensure durability
    
    // Periodically update metadata file (non-blocking)
    if should_update_metadata() {
        tokio::spawn(update_metadata_file(stats));
    }
}
```

### Atomicity Requirements
- Each line MUST be written atomically
- Use O_APPEND flag for true append-only behavior
- Flush after each write for durability
- NO rewriting or seeking backward in the tape file

### Line Validation
- Each line MUST be valid JSON independently
- Lines MUST NOT contain newline characters within JSON
- Maximum line length: 10MB (configurable)

### Ordering
- Init line MUST be first line
- Frames MUST be in sequence order
- Correlations MAY be written out of order
- Checkpoints MAY appear anywhere after init

## Reading Guidelines

### Streaming Read Pattern
```rust
// Pseudo-code for streaming reader - NO BUFFERING ENTIRE FILE
let file = File::open(tape_path)?;
let reader = BufReader::new(file);

// Process line by line without loading entire file
for line in reader.lines() {
    let line = line?;
    let record: TapeRecord = serde_json::from_str(&line)?;
    
    match record.record_type() {
        "init" => process_init(record),
        "frame" => process_frame(record),  // Process immediately
        "correlation" => process_correlation(record),
        "checkpoint" => update_stats(record),
        _ => log_unknown_record(record),
    }
}
```

### Concurrent Read While Writing
```rust
// Can read a tape that's still being written
let mut reader = BufReader::new(File::open(tape_path)?);
loop {
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) => {
            // No more data yet, wait or poll
            if tape_is_finalized() { break; }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Ok(_) => process_line(&line)?,
        Err(e) => handle_error(e)?,
    }
}
```

### Recovery Mode
- Skip invalid JSON lines with warning
- Continue reading after encountering errors
- Validate sequence numbers for gap detection
- No footer required - tape is valid even if incomplete
- Use metadata file for latest statistics

## Index Format

Optional index file (`{tape_id}.index`) for fast seeking:

```json
{
  "version": "1.0",
  "tape_id": "550e8400-e29b-41d4-a716-446655440000",
  "offsets": {
    "header": 0,
    "first_frame": 512,
    "frames": [
      {"seq": 0, "offset": 512, "length": 256},
      {"seq": 1, "offset": 768, "length": 300}
    ],
    "footer": 1048576
  },
  "frame_count": 150,
  "byte_size": 1049088
}
```

## Directory Index Format

The directory-level `index.json` file maintains a searchable index of all tapes:

```json
{
  "version": "2.0",
  "updated_at": "2025-08-14T10:35:00.000Z",
  "tapes": [
    {
      "tape_id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Production Debug Session",
      "created_at": "2025-08-14T10:30:00.000Z",
      "status": "recording",
      "frame_count": 150,
      "duration_ms": 300000,
      "size_bytes": 524288,
      "tags": ["production", "auth"]
    }
  ]
}
```

### Update Strategy
- Updated when tapes are created/finalized
- Can be rebuilt from `.meta.json` files
- Used for tape discovery and searching
- Not required for tape playback

## Performance Characteristics

### Memory Usage
- **Streaming Write**: O(1) - Only current frame in memory, no buffering
- **Streaming Read**: O(1) - Only current line in memory
- **Concurrent Read/Write**: O(1) - Multiple readers don't affect writer
- **Metadata Updates**: O(1) - Separate file, no tape rewrite

### Append Performance
- **Monolithic JSON**: O(n) - Must rewrite entire file for each append
- **JSON Lines**: O(1) - True append-only, no seeking or rewriting
- **Latency**: < 1ms per frame append (disk flush included)

### Streaming Benefits
- **Start reading immediately**: Don't wait for recording to finish
- **Live monitoring**: Tail the tape file like a log
- **No memory limits**: Can record indefinitely without OOM
- **Parallel processing**: Multiple consumers can read simultaneously

### Corruption Recovery
- **Monolithic JSON**: Total loss if structure corrupted
- **JSON Lines**: Only affected lines lost, rest remains readable
- **Partial writes**: Last incomplete line ignored, previous data intact

### File Size
- **Overhead**: ~2-5% larger due to repeated field names
- **Mitigation**: Can use shorter field names or compression
- **Trade-off**: Small size increase for massive memory savings

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 2.0 | 2025-08-14 | JSON Lines streaming format |

## Examples

### Minimal Valid Tape
```jsonl
{"type":"init","version":"2.0","tape_id":"550e8400-e29b-41d4-a716-446655440000","session_id":"test","created_at":"2025-08-14T10:30:00Z","protocol_version":"2025-11-05"}
```

### Active Recording (Still Being Written)
```jsonl
{"type":"init","version":"2.0","tape_id":"550e8400-e29b-41d4-a716-446655440000","session_id":"test","created_at":"2025-08-14T10:30:00Z","protocol_version":"2025-11-05"}
{"type":"frame","seq":0,"ts":0,"dir":"client_to_server","env":{"message":{"jsonrpc":"2.0","method":"initialize","id":1}}}
{"type":"frame","seq":1,"ts":100,"dir":"server_to_client","env":{"message":{"jsonrpc":"2.0","result":{"protocol_version":"2025-11-05"},"id":1}}}
{"type":"correlation","id":"c1","request_seq":0,"response_seq":1,"rtt_ms":100}
{"type":"frame","seq":2,"ts":200,"dir":"client_to_server","env":{"message":{"jsonrpc":"2.0","method":"tools/list","id":2}}}
```
Note: No footer needed - can be read while still recording!

### With Checkpoint
```jsonl
{"type":"init","version":"2.0","tape_id":"550e8400-e29b-41d4-a716-446655440000","session_id":"test","created_at":"2025-08-14T10:30:00Z","protocol_version":"2025-11-05"}
{"type":"frame","seq":0,"ts":0,"dir":"client_to_server","env":{"message":{"jsonrpc":"2.0","method":"initialize","id":1}}}
{"type":"frame","seq":1,"ts":100,"dir":"server_to_client","env":{"message":{"jsonrpc":"2.0","result":{"protocol_version":"2025-11-05"},"id":1}}}
{"type":"checkpoint","checkpoint_at":"2025-08-14T10:31:00Z","seq":1,"stats":{"frame_count":2,"duration_ms":60000}}
{"type":"frame","seq":2,"ts":60100,"dir":"client_to_server","env":{"message":{"jsonrpc":"2.0","method":"tools/list","id":2}}}
```

## Implementation Notes

### Rust Type Definitions
```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TapeRecord {
    Init(InitRecord),
    Frame(FrameRecord),
    Correlation(CorrelationRecord),
    Checkpoint(CheckpointRecord),
}

// Streaming writer holds minimal state
pub struct TapeWriter {
    file: File,           // Opened with O_APPEND
    meta_path: PathBuf,   // Path to .meta.json
    stats: TapeStats,     // Running statistics
    last_seq: u64,        // Last sequence number
}

// Streaming reader requires no state beyond file position
pub struct TapeReader {
    reader: BufReader<File>,
    init: Option<InitRecord>,
}
```

### File Locking Strategy
- **Tape file (.jsonl)**: Single writer, multiple readers
- **Metadata file (.meta.json)**: Atomic write-and-rename
- **Index file (.index)**: Updated after tape finalized
- **No exclusive locks** required for reading

## Security Considerations

1. **Line Length Limits**: Enforce maximum line length to prevent DoS
2. **JSON Parsing**: Use streaming parser to avoid memory exhaustion
3. **File Permissions**: Tapes may contain sensitive data, secure appropriately
4. **Checksum Validation**: Verify footer checksum when present
5. **Schema Validation**: Validate against schema to prevent injection

## Future Enhancements

1. **Compression**: Per-line compression with zstd
2. **Encryption**: Line-level encryption for sensitive data
3. **Streaming Index**: Periodic index markers for large files
4. **Binary Format**: MessagePack variant for size optimization
5. **Partitioning**: Split large recordings across multiple files