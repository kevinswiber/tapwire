### Recorder overhead and memory usage (Delta: shadowcat-delta@b793fd1)

Findings

- TapeRecorder buffers frames in memory per session with coarse-grained RwLocks
```150:171:shadowcat-delta/src/recorder/tape.rs
pub struct TapeRecorder {
    storage_dir: PathBuf,
    active_tapes: Arc<RwLock<HashMap<SessionId, Tape>>>,
    frame_buffer: Arc<RwLock<HashMap<SessionId, Vec<MessageEnvelope>>>>,
    buffer_limit: usize,
}
impl TapeRecorder {
    pub fn new<P: AsRef<Path>>(storage_dir: P) -> Self { … buffer_limit: 1000 }
    pub fn with_buffer_limit(mut self, limit: usize) -> Self { … }
```
- Recording path clones every envelope and performs lock, push, length check
```207:233:shadowcat-delta/src/recorder/tape.rs
pub async fn record_frame(&self, envelope: MessageEnvelope) -> RecorderResult<()> {
    let session_id = envelope.context.session_id.clone();
    let should_flush = {
        let mut frame_buffer = self.frame_buffer.write().await;
        if let Some(buffer) = frame_buffer.get_mut(&session_id) {
            buffer.push(envelope);
            buffer.len() >= self.buffer_limit
        } else { return Err(..); }
    };
    if should_flush { self.flush_frames(&session_id).await?; }
    Ok(())
}
```
- Flush moves vector under write lock, then appends into active tape under another write lock
```325:344:shadowcat-delta/src/recorder/tape.rs
let frames = { let mut frame_buffer = self.frame_buffer.write().await; frame_buffer.get_mut(session_id).map(std::mem::take).unwrap_or_default() };
if !frames.is_empty() {
    let mut active_tapes = self.active_tapes.write().await;
    if let Some(tape) = active_tapes.get_mut(session_id) {
        for frame in frames { tape.add_frame(frame); }
    }
}
```
- Total bytes computed by serializing all messages on stop
```116:126:shadowcat-delta/src/recorder/tape.rs
self.metadata.total_bytes = self.frames.iter().map(|envelope| serde_json::to_string(&envelope.message).map(|s| s.len()).unwrap_or(0)).sum();
```
- Forward proxy records to both session store and tape recorder separately, cloning twice
```521:531:shadowcat-delta/src/proxy/forward.rs
if let Some(session_manager) = &processors.session_manager { session_manager.record_frame(envelope.clone()).await?; }
if let Some(tape_recorder) = &processors.tape_recorder { tape_recorder.record_frame(envelope.clone()).await?; }
```

Risks

- Memory spikes with large `buffer_limit` and many sessions; copying full `MessageEnvelope` per sink.
- Lock contention on `frame_buffer` and `active_tapes` under high throughput.
- Stop-time cost scales with frames via `calculate_total_bytes` serialization walk.

Recommendations

- Streaming and zero-copy paths
  - Record by reference with an immutable `Arc<MessageEnvelopeInner>` to avoid per-sink clones.
  - Consider a single fan-out recorder that writes to both session store and tape sinks.
- Buffered IO and batching
  - Use a per-session bounded channel to accumulate frames; a background task drains and flushes to tape in batches.
  - Tune `buffer_limit` adaptively based on throughput or time-based flush (e.g., 50ms or 1MB).
- Lock granularity
  - Replace HashMap<RwLock<...>> with DashMap or per-session sharded locks to reduce contention.
  - In `flush_frames`, avoid holding the `active_tapes` write lock while iterating; collect and extend in one call.
- Precompute sizes
  - Maintain running `total_bytes` incrementally when buffering; avoid full serialization at stop.
- Backpressure
  - If buffer is full, drop non-essential frames or apply backpressure to producers to protect memory.
