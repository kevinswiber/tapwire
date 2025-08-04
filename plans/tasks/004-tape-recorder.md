# Task: Implement Tape Recording Engine

**Status:** Not Started  
**Priority:** Medium  
**Estimated Time:** 1.5 days  
**Dependencies:** Session types, Frame types

---

## Objective

Implement the tape recording system that captures all MCP traffic with timing information, enabling deterministic replay and session analysis. Design a flexible tape format that can evolve with the project.

---

## Tape Format Design

### JSON Structure
```json
{
  "version": "1.0",
  "tape_id": "550e8400-e29b-41d4-a716-446655440000",
  "session_id": "session-123",
  "metadata": {
    "created_at": "2025-08-04T10:00:00Z",
    "duration_ms": 5432,
    "transport": "stdio",
    "protocol_version": "2025-11-05",
    "client_info": {
      "name": "test-client",
      "version": "1.0"
    },
    "server_info": {
      "name": "test-server",
      "version": "2.0"
    },
    "frame_count": 10,
    "error_count": 0
  },
  "frames": [
    {
      "timestamp_ms": 0,
      "direction": "client_to_server",
      "edge": "transport_in",
      "content": {
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {},
        "id": 1
      },
      "metadata": {
        "size_bytes": 128,
        "is_synthetic": false
      }
    }
  ]
}
```

---

## Implementation Steps

### 1. Define Core Types
```rust
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tape {
    pub version: String,
    pub tape_id: Uuid,
    pub session_id: SessionId,
    pub metadata: TapeMetadata,
    pub frames: Vec<RecordedFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeMetadata {
    pub created_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub transport: TransportType,
    pub protocol_version: String,
    pub client_info: Option<ClientInfo>,
    pub server_info: Option<ServerInfo>,
    pub frame_count: usize,
    pub error_count: usize,
    pub compressed: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedFrame {
    pub timestamp_ms: u64,
    pub direction: Direction,
    pub edge: TransportEdge,
    pub content: Value,
    pub metadata: FrameMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMetadata {
    pub size_bytes: usize,
    pub is_synthetic: bool,
    pub error: Option<String>,
    pub latency_ms: Option<u64>,
}
```

### 2. Implement TapeRecorder
```rust
pub struct TapeRecorder {
    tape: Tape,
    start_time: Instant,
    storage: Arc<dyn TapeStorage>,
    buffer: Vec<RecordedFrame>,
    buffer_size: usize,
    auto_save: bool,
}

impl TapeRecorder {
    pub fn new(
        session_id: SessionId, 
        transport: TransportType,
        storage: Arc<dyn TapeStorage>,
    ) -> Self {
        let tape = Tape {
            version: TAPE_VERSION.to_string(),
            tape_id: Uuid::new_v4(),
            session_id,
            metadata: TapeMetadata {
                created_at: Utc::now(),
                duration_ms: 0,
                transport,
                protocol_version: MCP_PROTOCOL_VERSION.to_string(),
                client_info: None,
                server_info: None,
                frame_count: 0,
                error_count: 0,
                compressed: false,
                tags: Vec::new(),
            },
            frames: Vec::new(),
        };
        
        Self {
            tape,
            start_time: Instant::now(),
            storage,
            buffer: Vec::with_capacity(1000),
            buffer_size: 1000,
            auto_save: true,
        }
    }
}
```

### 3. Implement Frame Recording
```rust
impl TapeRecorder {
    pub async fn record_frame(
        &mut self,
        direction: Direction,
        edge: TransportEdge,
        message: &TransportMessage,
    ) -> RecorderResult<()> {
        let timestamp_ms = self.start_time.elapsed().as_millis() as u64;
        
        let content = serde_json::to_value(message)
            .map_err(|e| RecorderError::SerializationFailed(e.to_string()))?;
        
        let size_bytes = content.to_string().len();
        
        let frame = RecordedFrame {
            timestamp_ms,
            direction,
            edge,
            content: content.clone(),
            metadata: FrameMetadata {
                size_bytes,
                is_synthetic: false,
                error: None,
                latency_ms: None,
            },
        };
        
        // Extract protocol information
        self.extract_protocol_info(&message);
        
        // Update metadata
        self.tape.metadata.frame_count += 1;
        self.tape.metadata.duration_ms = timestamp_ms;
        
        // Check for errors
        if message.is_error() {
            self.tape.metadata.error_count += 1;
        }
        
        // Buffer management
        self.buffer.push(frame);
        
        if self.buffer.len() >= self.buffer_size && self.auto_save {
            self.flush_buffer().await?;
        }
        
        Ok(())
    }
    
    async fn flush_buffer(&mut self) -> RecorderResult<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        
        self.tape.frames.extend(self.buffer.drain(..));
        
        // Auto-save to storage
        self.storage.save_partial(&self.tape).await?;
        
        Ok(())
    }
}
```

### 4. Extract Protocol Information
```rust
impl TapeRecorder {
    fn extract_protocol_info(&mut self, message: &TransportMessage) {
        match message {
            TransportMessage::Request { method, params, .. } => {
                if method == "initialize" {
                    if let Some(client_info) = params.get("clientInfo") {
                        self.tape.metadata.client_info = 
                            serde_json::from_value(client_info.clone()).ok();
                    }
                }
            }
            TransportMessage::Response { result, .. } => {
                if let Some(result) = result {
                    if let Some(server_info) = result.get("serverInfo") {
                        self.tape.metadata.server_info = 
                            serde_json::from_value(server_info.clone()).ok();
                    }
                }
            }
            _ => {}
        }
    }
}
```

### 5. Implement Tape Storage
```rust
#[async_trait]
pub trait TapeStorage: Send + Sync {
    async fn save(&self, tape: &Tape) -> StorageResult<()>;
    async fn save_partial(&self, tape: &Tape) -> StorageResult<()>;
    async fn load(&self, tape_id: &Uuid) -> StorageResult<Tape>;
    async fn list(&self, filter: TapeFilter) -> StorageResult<Vec<TapeInfo>>;
    async fn delete(&self, tape_id: &Uuid) -> StorageResult<()>;
}

pub struct FileTapeStorage {
    base_dir: PathBuf,
    compression: bool,
}

#[async_trait]
impl TapeStorage for FileTapeStorage {
    async fn save(&self, tape: &Tape) -> StorageResult<()> {
        let file_name = format!("{}.json", tape.tape_id);
        let file_path = self.base_dir.join(&file_name);
        
        let json = serde_json::to_string_pretty(tape)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        if self.compression {
            // Compress with gzip
            let compressed = compress_json(&json)?;
            tokio::fs::write(file_path.with_extension("json.gz"), compressed)
                .await
                .map_err(|e| StorageError::Io(e.to_string()))?;
        } else {
            tokio::fs::write(file_path, json)
                .await
                .map_err(|e| StorageError::Io(e.to_string()))?;
        }
        
        Ok(())
    }
}
```

### 6. Implement Finalization
```rust
impl TapeRecorder {
    pub async fn finalize(&mut self) -> RecorderResult<()> {
        // Flush any remaining buffered frames
        self.flush_buffer().await?;
        
        // Update final metadata
        self.tape.metadata.duration_ms = self.start_time.elapsed().as_millis() as u64;
        
        // Calculate statistics
        self.calculate_statistics();
        
        // Save complete tape
        self.storage.save(&self.tape).await?;
        
        info!(
            "Tape finalized: {} frames, {} ms duration",
            self.tape.metadata.frame_count,
            self.tape.metadata.duration_ms
        );
        
        Ok(())
    }
    
    fn calculate_statistics(&mut self) {
        // Calculate average latencies
        let mut request_times: HashMap<String, u64> = HashMap::new();
        
        for frame in &self.tape.frames {
            if let Some(id) = frame.content.get("id").and_then(|v| v.as_str()) {
                match frame.direction {
                    Direction::ClientToServer => {
                        request_times.insert(id.to_string(), frame.timestamp_ms);
                    }
                    Direction::ServerToClient => {
                        if let Some(request_time) = request_times.get(id) {
                            let latency = frame.timestamp_ms - request_time;
                            // Add latency to frame metadata (would need mutable access)
                        }
                    }
                }
            }
        }
    }
}
```

### 7. Add Synthetic Frame Support
```rust
impl TapeRecorder {
    pub async fn add_synthetic_frame(
        &mut self,
        direction: Direction,
        content: Value,
        reason: &str,
    ) -> RecorderResult<()> {
        let timestamp_ms = self.start_time.elapsed().as_millis() as u64;
        
        let frame = RecordedFrame {
            timestamp_ms,
            direction,
            edge: TransportEdge::ProxyIn, // Synthetic frames originate from proxy
            content,
            metadata: FrameMetadata {
                size_bytes: 0,
                is_synthetic: true,
                error: Some(format!("Synthetic: {}", reason)),
                latency_ms: None,
            },
        };
        
        self.buffer.push(frame);
        self.tape.metadata.frame_count += 1;
        
        if self.buffer.len() >= self.buffer_size && self.auto_save {
            self.flush_buffer().await?;
        }
        
        Ok(())
    }
}
```

---

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_tape_recording() {
    let storage = Arc::new(MemoryTapeStorage::new());
    let mut recorder = TapeRecorder::new(
        SessionId::new(),
        TransportType::Stdio,
        storage.clone(),
    );
    
    // Record frames
    recorder.record_frame(
        Direction::ClientToServer,
        TransportEdge::TransportIn,
        &TransportMessage::new_request("1", "test", json!({}))
    ).await.unwrap();
    
    recorder.record_frame(
        Direction::ServerToClient,
        TransportEdge::TransportOut,
        &TransportMessage::new_response("1", Some(json!({"ok": true})), None)
    ).await.unwrap();
    
    // Finalize
    recorder.finalize().await.unwrap();
    
    // Verify
    let tape = storage.load(&recorder.tape.tape_id).await.unwrap();
    assert_eq!(tape.frames.len(), 2);
    assert_eq!(tape.metadata.frame_count, 2);
    assert!(tape.metadata.duration_ms > 0);
}
```

---

## Performance Considerations

- Buffer frames in memory before writing
- Use compression for large tapes
- Consider splitting very large tapes
- Index tapes for fast searching
- Stream frames during replay instead of loading all

---

## Future Enhancements

- Tape editing capabilities
- Tape merging for multi-session analysis
- Real-time streaming to remote storage
- Encryption for sensitive data
- Tape validation and repair tools
- Export to other formats (HAR, etc.)