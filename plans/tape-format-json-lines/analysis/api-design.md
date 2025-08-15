# API Design: Streaming Tape Interfaces

## Design Principles

1. **Zero-Buffer Architecture**: Never hold entire tape in memory
2. **Async-First**: All I/O operations are async
3. **Fail-Safe**: Partial failures don't corrupt recordings
4. **Concurrent-Friendly**: Multiple readers don't block writer
5. **Progressive Enhancement**: Start simple, add features incrementally

## Core Types

### Record Types

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Discriminated union for all tape record types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TapeRecord {
    Init(InitRecord),
    Frame(FrameRecord),
    Correlation(CorrelationRecord),
    Checkpoint(CheckpointRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitRecord {
    pub version: String,
    pub tape_id: TapeId,
    pub session_id: SessionId,
    pub created_at: DateTime<Utc>,
    pub protocol_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameRecord {
    pub seq: u64,
    pub ts: u64,  // milliseconds since start
    pub dir: MessageDirection,
    pub env: MessageEnvelope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<InterceptAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<FrameFlags>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationRecord {
    pub id: String,
    pub request_seq: u64,
    pub response_seq: u64,
    pub request_ts: u64,
    pub response_ts: u64,
    pub rtt_ms: u64,
    pub status: CorrelationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointRecord {
    pub checkpoint_at: DateTime<Utc>,
    pub seq: u64,
    pub stats: TapeStats,
}
```

### Metadata Types

```rust
/// Tape metadata stored in companion .meta.json file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeMetadata {
    pub tape_id: TapeId,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub finalized_at: Option<DateTime<Utc>>,
    pub status: TapeStatus,
    pub transport: TransportInfo,
    pub environment: Environment,
    pub stats: TapeStats,
    pub performance: Option<PerformanceStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TapeStatus {
    Recording,
    Paused,
    Finalized,
    Error(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TapeStats {
    pub frame_count: u64,
    pub duration_ms: u64,
    pub file_size_bytes: u64,
    pub last_sequence: u64,
    pub message_counts: MessageCounts,
    pub error_count: u64,
    pub correlation_count: u64,
    pub checkpoint_count: u64,
}
```

## Streaming Writer API

```rust
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Streaming tape writer - zero buffering, immediate append
pub struct StreamingTapeWriter {
    // Core state
    tape_id: TapeId,
    file: BufWriter<File>,
    meta_path: PathBuf,
    
    // Runtime state
    start_time: Instant,
    last_seq: u64,
    stats: TapeStats,
    
    // Configuration
    checkpoint_interval: Option<Duration>,
    metadata_update_interval: Duration,
    last_checkpoint: Instant,
    last_metadata_update: Instant,
}

impl StreamingTapeWriter {
    /// Create a new streaming tape writer
    pub async fn create(
        dir: impl AsRef<Path>,
        session_id: SessionId,
        metadata: TapeMetadata,
    ) -> Result<Self> {
        let tape_id = TapeId::new();
        let tape_path = dir.as_ref().join(format!("{}.jsonl", tape_id));
        let meta_path = dir.as_ref().join(format!("{}.meta.json", tape_id));
        
        // Open file in append mode
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&tape_path)
            .await?;
        
        let mut writer = Self {
            tape_id,
            file: BufWriter::new(file),
            meta_path,
            start_time: Instant::now(),
            last_seq: 0,
            stats: TapeStats::default(),
            checkpoint_interval: Some(Duration::from_secs(60)),
            metadata_update_interval: Duration::from_secs(10),
            last_checkpoint: Instant::now(),
            last_metadata_update: Instant::now(),
        };
        
        // Write init record
        writer.write_init(session_id).await?;
        
        // Create initial metadata file
        writer.update_metadata(metadata).await?;
        
        Ok(writer)
    }
    
    /// Append a frame to the tape - O(1) operation
    pub async fn write_frame(
        &mut self,
        envelope: MessageEnvelope,
        metadata: Option<TransportMetadata>,
    ) -> Result<u64> {
        let seq = self.last_seq;
        self.last_seq += 1;
        
        let frame = FrameRecord {
            seq,
            ts: self.start_time.elapsed().as_millis() as u64,
            dir: envelope.direction,
            env: envelope,
            action: None,
            transport: metadata,
            correlation_id: None,
            flags: None,
        };
        
        // Write line atomically
        self.write_record(TapeRecord::Frame(frame)).await?;
        
        // Update statistics
        self.stats.frame_count += 1;
        self.stats.last_sequence = seq;
        
        // Check if we should write checkpoint or update metadata
        self.maybe_checkpoint().await?;
        self.maybe_update_metadata().await?;
        
        Ok(seq)
    }
    
    /// Write a correlation record
    pub async fn write_correlation(
        &mut self,
        request_seq: u64,
        response_seq: u64,
    ) -> Result<()> {
        let correlation = CorrelationRecord {
            id: format!("corr-{}-{}", request_seq, response_seq),
            request_seq,
            response_seq,
            request_ts: 0,  // Would be tracked from frames
            response_ts: 0,
            rtt_ms: 0,
            status: CorrelationStatus::Success,
        };
        
        self.write_record(TapeRecord::Correlation(correlation)).await?;
        self.stats.correlation_count += 1;
        Ok(())
    }
    
    /// Finalize the tape
    pub async fn finalize(mut self) -> Result<TapeMetadata> {
        // Write final checkpoint
        self.write_checkpoint().await?;
        
        // Update metadata with final status
        let mut metadata = self.load_metadata().await?;
        metadata.finalized_at = Some(Utc::now());
        metadata.status = TapeStatus::Finalized;
        metadata.stats = self.stats;
        self.save_metadata(&metadata).await?;
        
        // Ensure all data is flushed
        self.file.flush().await?;
        
        Ok(metadata)
    }
    
    // Private helper methods
    
    async fn write_record(&mut self, record: TapeRecord) -> Result<()> {
        let line = serde_json::to_string(&record)?;
        self.file.write_all(line.as_bytes()).await?;
        self.file.write_all(b"\n").await?;
        self.file.flush().await?;  // Ensure durability
        Ok(())
    }
    
    async fn write_init(&mut self, session_id: SessionId) -> Result<()> {
        let init = InitRecord {
            version: "2.0".to_string(),
            tape_id: self.tape_id.clone(),
            session_id,
            created_at: Utc::now(),
            protocol_version: "2025-11-05".to_string(),
        };
        self.write_record(TapeRecord::Init(init)).await
    }
    
    async fn maybe_checkpoint(&mut self) -> Result<()> {
        if let Some(interval) = self.checkpoint_interval {
            if self.last_checkpoint.elapsed() >= interval {
                self.write_checkpoint().await?;
                self.last_checkpoint = Instant::now();
            }
        }
        Ok(())
    }
    
    async fn write_checkpoint(&mut self) -> Result<()> {
        let checkpoint = CheckpointRecord {
            checkpoint_at: Utc::now(),
            seq: self.last_seq,
            stats: self.stats.clone(),
        };
        self.write_record(TapeRecord::Checkpoint(checkpoint)).await
    }
    
    async fn maybe_update_metadata(&mut self) -> Result<()> {
        if self.last_metadata_update.elapsed() >= self.metadata_update_interval {
            self.update_metadata_stats().await?;
            self.last_metadata_update = Instant::now();
        }
        Ok(())
    }
}
```

## Streaming Reader API

```rust
use tokio::io::{AsyncBufReadExt, BufReader};
use futures::stream::Stream;

/// Streaming tape reader - zero buffering, progressive parsing
pub struct StreamingTapeReader {
    reader: BufReader<File>,
    init: Option<InitRecord>,
    current_line: u64,
}

impl StreamingTapeReader {
    /// Open a tape for streaming read
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path).await?;
        let mut reader = Self {
            reader: BufReader::new(file),
            init: None,
            current_line: 0,
        };
        
        // Read and validate init record
        reader.read_init().await?;
        
        Ok(reader)
    }
    
    /// Read the next record from the tape
    pub async fn next_record(&mut self) -> Result<Option<TapeRecord>> {
        let mut line = String::new();
        match self.reader.read_line(&mut line).await? {
            0 => Ok(None),  // EOF
            _ => {
                self.current_line += 1;
                match serde_json::from_str(&line) {
                    Ok(record) => Ok(Some(record)),
                    Err(e) => {
                        warn!("Skipping invalid line {}: {}", self.current_line, e);
                        self.next_record().await  // Skip and continue
                    }
                }
            }
        }
    }
    
    /// Stream all frames (skip other record types)
    pub fn frames(&mut self) -> impl Stream<Item = Result<FrameRecord>> + '_ {
        async_stream::stream! {
            while let Some(record) = self.next_record().await? {
                if let TapeRecord::Frame(frame) = record {
                    yield Ok(frame);
                }
            }
        }
    }
    
    /// Follow a tape that's still being written (tail -f behavior)
    pub fn follow(&mut self) -> impl Stream<Item = Result<TapeRecord>> + '_ {
        async_stream::stream! {
            loop {
                match self.next_record().await {
                    Ok(Some(record)) => yield Ok(record),
                    Ok(None) => {
                        // No more data, wait and retry
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    Err(e) => yield Err(e),
                }
            }
        }
    }
    
    /// Get tape metadata
    pub async fn metadata(&self) -> Result<TapeMetadata> {
        let meta_path = self.meta_path()?;
        let content = tokio::fs::read_to_string(meta_path).await?;
        Ok(serde_json::from_str(&content)?)
    }
    
    /// Get init record
    pub fn init(&self) -> Option<&InitRecord> {
        self.init.as_ref()
    }
    
    // Private helpers
    
    async fn read_init(&mut self) -> Result<()> {
        match self.next_record().await? {
            Some(TapeRecord::Init(init)) => {
                self.init = Some(init);
                Ok(())
            }
            _ => Err(Error::InvalidTape("Missing or invalid init record")),
        }
    }
}
```

## Concurrent Read/Write Support

```rust
/// Manager for concurrent tape access
pub struct TapeManager {
    tapes: Arc<RwLock<HashMap<TapeId, TapeHandle>>>,
}

pub struct TapeHandle {
    writer: Option<Arc<Mutex<StreamingTapeWriter>>>,
    reader_count: usize,
    path: PathBuf,
}

impl TapeManager {
    /// Start recording a new tape
    pub async fn start_recording(
        &self,
        session_id: SessionId,
        metadata: TapeMetadata,
    ) -> Result<Arc<Mutex<StreamingTapeWriter>>> {
        let writer = StreamingTapeWriter::create(
            &self.storage_dir,
            session_id,
            metadata,
        ).await?;
        
        let writer = Arc::new(Mutex::new(writer));
        
        let mut tapes = self.tapes.write().await;
        tapes.insert(writer.tape_id.clone(), TapeHandle {
            writer: Some(writer.clone()),
            reader_count: 0,
            path: self.storage_dir.join(format!("{}.jsonl", writer.tape_id)),
        });
        
        Ok(writer)
    }
    
    /// Open a tape for reading (doesn't block writer)
    pub async fn open_for_reading(&self, tape_id: &TapeId) -> Result<StreamingTapeReader> {
        let tapes = self.tapes.read().await;
        let handle = tapes.get(tape_id)
            .ok_or_else(|| Error::TapeNotFound(tape_id.to_string()))?;
        
        let reader = StreamingTapeReader::open(&handle.path).await?;
        
        // Increment reader count (for statistics)
        drop(tapes);
        let mut tapes = self.tapes.write().await;
        if let Some(handle) = tapes.get_mut(tape_id) {
            handle.reader_count += 1;
        }
        
        Ok(reader)
    }
    
    /// Follow a tape that's being recorded
    pub async fn follow_tape(
        &self,
        tape_id: &TapeId,
    ) -> Result<impl Stream<Item = Result<TapeRecord>>> {
        let reader = self.open_for_reading(tape_id).await?;
        Ok(reader.follow())
    }
}
```

## CLI Integration Examples

```rust
/// Record command using streaming writer
pub async fn record_command(args: RecordArgs) -> Result<()> {
    let writer = StreamingTapeWriter::create(
        &args.output_dir,
        args.session_id,
        create_metadata(&args),
    ).await?;
    
    let writer = Arc::new(Mutex::new(writer));
    
    // Process messages without buffering
    while let Some(msg) = transport.recv().await? {
        let mut w = writer.lock().await;
        w.write_frame(msg, None).await?;
    }
    
    // Finalize when done
    let w = Arc::try_unwrap(writer)
        .map_err(|_| Error::Internal("Writer still in use"))?
        .into_inner();
    w.finalize().await?;
    
    Ok(())
}

/// Replay command using streaming reader
pub async fn replay_command(args: ReplayArgs) -> Result<()> {
    let mut reader = StreamingTapeReader::open(&args.tape_path).await?;
    
    // Stream frames without loading entire tape
    let mut frames = reader.frames();
    while let Some(frame) = frames.next().await {
        let frame = frame?;
        process_frame(frame).await?;
        
        // Optional delay for realistic playback
        if let Some(delay) = args.playback_delay {
            tokio::time::sleep(delay).await;
        }
    }
    
    Ok(())
}

/// Tail command for live monitoring
pub async fn tail_command(args: TailArgs) -> Result<()> {
    let mut reader = StreamingTapeReader::open(&args.tape_path).await?;
    
    // Follow the tape like tail -f
    let mut stream = reader.follow();
    while let Some(record) = stream.next().await {
        match record? {
            TapeRecord::Frame(frame) => {
                println!("{}: {:?}", frame.seq, frame.env.message);
            }
            TapeRecord::Checkpoint(cp) => {
                println!("--- Checkpoint: {} frames, {} ms ---", 
                    cp.stats.frame_count, cp.stats.duration_ms);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum TapeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid tape: {0}")]
    InvalidTape(String),
    
    #[error("Tape not found: {0}")]
    TapeNotFound(String),
    
    #[error("Corrupted record at line {line}: {error}")]
    CorruptedRecord { line: u64, error: String },
    
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, TapeError>;
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_streaming_write_read() {
        let dir = TempDir::new().unwrap();
        
        // Write frames without buffering
        let mut writer = StreamingTapeWriter::create(
            dir.path(),
            SessionId::new(),
            test_metadata(),
        ).await.unwrap();
        
        for i in 0..1000 {
            writer.write_frame(test_envelope(i), None).await.unwrap();
        }
        
        let metadata = writer.finalize().await.unwrap();
        assert_eq!(metadata.stats.frame_count, 1000);
        
        // Read frames without loading entire tape
        let mut reader = StreamingTapeReader::open(
            dir.path().join(format!("{}.jsonl", metadata.tape_id))
        ).await.unwrap();
        
        let mut count = 0;
        while let Some(record) = reader.next_record().await.unwrap() {
            if matches!(record, TapeRecord::Frame(_)) {
                count += 1;
            }
        }
        assert_eq!(count, 1000);
    }
    
    #[tokio::test]
    async fn test_concurrent_read_write() {
        let dir = TempDir::new().unwrap();
        let manager = TapeManager::new(dir.path());
        
        // Start recording
        let writer = manager.start_recording(
            SessionId::new(),
            test_metadata(),
        ).await.unwrap();
        
        let tape_id = {
            let w = writer.lock().await;
            w.tape_id.clone()
        };
        
        // Start reading while writing
        let reader_handle = tokio::spawn(async move {
            let reader = manager.open_for_reading(&tape_id).await.unwrap();
            let mut stream = reader.follow();
            let mut count = 0;
            while let Some(record) = stream.next().await {
                if matches!(record.unwrap(), TapeRecord::Frame(_)) {
                    count += 1;
                    if count >= 100 {
                        break;
                    }
                }
            }
            count
        });
        
        // Write frames
        for i in 0..100 {
            let mut w = writer.lock().await;
            w.write_frame(test_envelope(i), None).await.unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        // Verify reader got all frames
        let count = reader_handle.await.unwrap();
        assert_eq!(count, 100);
    }
}
```

## Performance Considerations

1. **Buffer Sizes**: Use 8KB buffers for optimal disk I/O
2. **Flush Strategy**: Flush after each write for durability
3. **Checkpoint Frequency**: Every 60 seconds or 1000 frames
4. **Metadata Updates**: Every 10 seconds to avoid lock contention
5. **Line Length Limit**: 10MB max to prevent memory issues

## Migration Path

1. Implement new streaming modules in parallel
2. Add feature flag for JSON Lines format
3. Test thoroughly with benchmarks
4. Switch default format to JSON Lines
5. Remove old implementation

## Future Enhancements

1. **Compression**: Per-line zstd compression
2. **Encryption**: Line-level encryption for sensitive data
3. **Remote Storage**: Stream directly to S3/GCS
4. **Filtering**: Reader-side filtering by criteria
5. **Indexing**: Binary index for fast seeking