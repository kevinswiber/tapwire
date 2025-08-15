# Task B.3: Recording Configuration Implementation

**Task ID**: B.3  
**Depends On**: A.1 (CLI Design Proposal)  
**Estimated Duration**: 2 hours  
**Priority**: MEDIUM  
**Status**: Not Started

## Objective

Implement CLI support for session recording configuration in the gateway, enabling capture of MCP sessions for debugging, auditing, and replay purposes.

## Background

The gateway module has built-in recording capabilities via the `TapeRecorder` but these are not exposed through CLI. Recording is essential for debugging MCP interactions, creating test fixtures, and audit compliance.

## Requirements

### Functional Requirements
1. Enable/disable recording via CLI flag
2. Configure recording directory
3. Support recording format selection (tape, jsonl)
4. Implement session filtering for selective recording
5. Add rotation and size management options
6. Provide recording metadata (timestamps, session info)

### Non-Functional Requirements
1. Minimal performance impact when disabled
2. Efficient disk I/O when enabled
3. Thread-safe recording operations
4. Automatic cleanup of old recordings

## Implementation Plan

### Step 1: Extend CLI Arguments (20 min)
```rust
// In src/cli/gateway.rs
#[derive(Debug, Args)]
pub struct ReverseCommand {
    // ... existing fields ...
    
    /// Enable session recording
    #[arg(long)]
    pub enable_recording: bool,
    
    /// Directory for recording files
    #[arg(long, default_value = "./tapes")]
    pub recording_dir: PathBuf,
    
    /// Recording format
    #[arg(long, value_enum, default_value = "tape")]
    pub recording_format: RecordingFormat,
    
    /// Filter pattern for selective recording
    #[arg(long)]
    pub recording_filter: Option<String>,
    
    /// Maximum recording file size in MB
    #[arg(long, default_value = "100")]
    pub recording_max_size_mb: u64,
    
    /// Maximum number of recording files to keep
    #[arg(long, default_value = "100")]
    pub recording_max_files: usize,
    
    /// Enable recording compression
    #[arg(long)]
    pub recording_compress: bool,
    
    /// Recording rotation strategy
    #[arg(long, value_enum, default_value = "size")]
    pub recording_rotation: RotationStrategy,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum RecordingFormat {
    /// Shadowcat tape format
    Tape,
    /// JSON Lines format
    Jsonl,
    /// Raw MCP messages
    Raw,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum RotationStrategy {
    /// Rotate based on file size
    Size,
    /// Rotate daily
    Daily,
    /// Rotate hourly
    Hourly,
    /// Never rotate
    Never,
}
```

### Step 2: Implement Recording Filter (30 min)
```rust
use glob::Pattern;

pub struct RecordingFilter {
    include_patterns: Vec<Pattern>,
    exclude_patterns: Vec<Pattern>,
}

impl RecordingFilter {
    pub fn from_spec(spec: &str) -> Result<Self> {
        // Parse filter specification
        // Format: "+pattern1,-pattern2,+pattern3"
        let mut include = Vec::new();
        let mut exclude = Vec::new();
        
        for part in spec.split(',') {
            let pattern = part.trim_start_matches(&['+', '-']);
            let glob = Pattern::new(pattern)?;
            
            if part.starts_with('-') {
                exclude.push(glob);
            } else {
                include.push(glob);
            }
        }
        
        Ok(Self {
            include_patterns: include,
            exclude_patterns: exclude,
        })
    }
    
    pub fn should_record(&self, message: &ProtocolMessage) -> bool {
        // Check exclusions first
        for pattern in &self.exclude_patterns {
            if self.matches_message(pattern, message) {
                return false;
            }
        }
        
        // If no includes specified, record everything not excluded
        if self.include_patterns.is_empty() {
            return true;
        }
        
        // Check inclusions
        for pattern in &self.include_patterns {
            if self.matches_message(pattern, message) {
                return true;
            }
        }
        
        false
    }
    
    fn matches_message(&self, pattern: &Pattern, msg: &ProtocolMessage) -> bool {
        // Match against method, path, or other message attributes
        match msg {
            ProtocolMessage::Request(req) => {
                pattern.matches(&req.method)
            }
            _ => false
        }
    }
}
```

### Step 3: Configure Recording System (30 min)
```rust
impl ReverseCommand {
    fn setup_recording(&self, config: &mut ReverseProxyConfig) -> Result<()> {
        if !self.enable_recording {
            config.enable_recording = false;
            return Ok(());
        }
        
        // Create recording directory if it doesn't exist
        std::fs::create_dir_all(&self.recording_dir)?;
        
        // Setup recording configuration
        config.enable_recording = true;
        config.recording_dir = Some(self.recording_dir.clone());
        
        // Create recorder with configuration
        let recorder_config = TapeRecorderConfig {
            output_dir: self.recording_dir.clone(),
            format: self.recording_format.into(),
            max_file_size: self.recording_max_size_mb * 1024 * 1024,
            max_files: self.recording_max_files,
            compression: self.recording_compress,
            rotation: self.recording_rotation.into(),
            filter: self.recording_filter
                .as_ref()
                .map(|f| RecordingFilter::from_spec(f))
                .transpose()?,
        };
        
        // Validate configuration
        recorder_config.validate()?;
        
        Ok(())
    }
}
```

### Step 4: Integrate with Proxy Server (20 min)
```rust
// In gateway server initialization
pub async fn start_with_recording(&self) -> Result<()> {
    let recorder = if self.config.enable_recording {
        let recorder = TapeRecorder::new(
            self.config.recording_dir.clone().unwrap(),
            self.config.recording_format,
        )?;
        Some(Arc::new(recorder))
    } else {
        None
    };
    
    // Pass recorder to request handlers
    let app_state = AppState {
        session_manager: self.session_manager.clone(),
        recorder: recorder.clone(),
        // ... other state
    };
    
    // In request handler
    if let Some(recorder) = &state.recorder {
        if should_record(&message) {
            recorder.record_message(session_id, message).await?;
        }
    }
}
```

### Step 5: Implement Rotation Logic (20 min)
```rust
pub struct RecordingRotator {
    strategy: RotationStrategy,
    max_size: u64,
    max_files: usize,
    current_file: Option<PathBuf>,
    current_size: u64,
    last_rotation: Instant,
}

impl RecordingRotator {
    pub fn should_rotate(&self) -> bool {
        match self.strategy {
            RotationStrategy::Size => self.current_size >= self.max_size,
            RotationStrategy::Daily => {
                self.last_rotation.elapsed() >= Duration::from_secs(86400)
            }
            RotationStrategy::Hourly => {
                self.last_rotation.elapsed() >= Duration::from_secs(3600)
            }
            RotationStrategy::Never => false,
        }
    }
    
    pub fn rotate(&mut self) -> Result<PathBuf> {
        // Generate new filename with timestamp
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("recording_{}.tape", timestamp);
        let path = self.recording_dir.join(filename);
        
        // Clean up old files if needed
        self.cleanup_old_files()?;
        
        self.current_file = Some(path.clone());
        self.current_size = 0;
        self.last_rotation = Instant::now();
        
        Ok(path)
    }
    
    fn cleanup_old_files(&self) -> Result<()> {
        let mut files: Vec<_> = std::fs::read_dir(&self.recording_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "tape" || s == "jsonl")
                    .unwrap_or(false)
            })
            .collect();
        
        // Sort by modification time
        files.sort_by_key(|f| f.metadata().unwrap().modified().unwrap());
        
        // Remove oldest files if over limit
        while files.len() > self.max_files {
            if let Some(oldest) = files.first() {
                std::fs::remove_file(oldest.path())?;
                files.remove(0);
            }
        }
        
        Ok(())
    }
}
```

### Step 6: Add Metadata to Recordings (15 min)
```rust
#[derive(Serialize, Deserialize)]
pub struct RecordingMetadata {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub upstream_id: String,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub recording_version: String,
}

impl TapeRecorder {
    pub async fn start_session(&self, metadata: RecordingMetadata) -> Result<()> {
        // Write metadata header
        let header = json!({
            "type": "session_start",
            "metadata": metadata,
            "version": "1.0",
        });
        
        self.write_entry(header).await?;
        Ok(())
    }
}
```

### Step 7: Add Tests (15 min)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_recording_filter_parsing() {
        let filter = RecordingFilter::from_spec("+tools/*,-system/*").unwrap();
        
        let tools_msg = create_test_message("tools/list");
        assert!(filter.should_record(&tools_msg));
        
        let system_msg = create_test_message("system/info");
        assert!(!filter.should_record(&system_msg));
    }
    
    #[tokio::test]
    async fn test_rotation_by_size() {
        let rotator = RecordingRotator::new(
            RotationStrategy::Size,
            1024 * 1024, // 1MB
            10,
        );
        
        // Write data until rotation triggers
        // Assert new file created
    }
    
    #[tokio::test]
    async fn test_cleanup_old_files() {
        // Create more than max_files recordings
        // Assert oldest are deleted
    }
}
```

## Testing Checklist

- [ ] Recording enable/disable works correctly
- [ ] Recording directory is created if missing
- [ ] Filter patterns work as expected
- [ ] Rotation triggers at correct thresholds
- [ ] Old files are cleaned up properly
- [ ] Compression works when enabled
- [ ] Different formats produce valid output
- [ ] Metadata is included in recordings
- [ ] Performance impact is minimal

## Documentation Updates

### CLI Help Text
```
--enable-recording
    Enable session recording for debugging and audit
    
--recording-dir <DIR>
    Directory to store recording files [default: ./tapes]
    
--recording-format <FORMAT>
    Recording file format
    
    Possible values:
    - tape:  Shadowcat tape format (recommended)
    - jsonl: JSON Lines format
    - raw:   Raw MCP messages
    
    Default: tape
    
--recording-filter <PATTERN>
    Filter pattern for selective recording
    Format: "+include,-exclude" (e.g., "+tools/*,-system/*")
    
--recording-max-size-mb <SIZE>
    Maximum size per recording file in MB [default: 100]
    
--recording-max-files <COUNT>
    Maximum number of recording files to keep [default: 100]
    
--recording-compress
    Enable compression for recording files
    
--recording-rotation <STRATEGY>
    Recording file rotation strategy
    
    Possible values:
    - size:   Rotate when file reaches max size
    - daily:  Rotate daily at midnight
    - hourly: Rotate every hour
    - never:  Never rotate (single file)
    
    Default: size
```

### Example Commands
```bash
# Basic recording
shadowcat reverse --upstream http://server \
  --enable-recording

# Custom directory with compression
shadowcat reverse --upstream http://server \
  --enable-recording \
  --recording-dir /var/log/shadowcat/tapes \
  --recording-compress

# Selective recording with filter
shadowcat reverse --upstream http://server \
  --enable-recording \
  --recording-filter "+tools/*,+resources/*,-system/*"

# Daily rotation with cleanup
shadowcat reverse --upstream http://server \
  --enable-recording \
  --recording-rotation daily \
  --recording-max-files 30
```

## Configuration File Support

```yaml
recording:
  enabled: true
  directory: "./tapes"
  format: tape
  filter:
    include:
      - "tools/*"
      - "resources/*"
    exclude:
      - "system/*"
  max_size_mb: 100
  max_files: 100
  rotation: daily
  compression: true
```

## Success Criteria

1. Recording can be enabled/disabled via CLI
2. Recording directory is configurable
3. Multiple recording formats work correctly
4. Filter patterns correctly include/exclude messages
5. Rotation works for all strategies
6. Old files are cleaned up automatically
7. Compression reduces file size by >50%
8. Metadata is included in recordings
9. No performance impact when disabled
10. < 5% performance impact when enabled

## Error Handling

```
Error: Cannot create recording directory '/restricted/path'
Hint: Check permissions or specify a different directory with --recording-dir

Warning: Recording filter pattern 'invalid[' is malformed
Info: Recording all sessions without filtering

Error: Recording directory full (100 files maximum reached)
Hint: Increase --recording-max-files or enable rotation

Warning: Large recording file (500MB) may impact performance
Info: Consider enabling rotation or compression
```

## Performance Considerations

1. Use buffered writes to reduce I/O overhead
2. Compress in background thread to avoid blocking
3. Implement write-ahead buffer for high throughput
4. Consider async file I/O for better concurrency
5. Add metrics for recording performance

## Future Enhancements

1. Support for S3/cloud storage backends
2. Real-time streaming of recordings
3. Recording replay via CLI
4. Recording analysis tools
5. Encryption for sensitive recordings
6. Recording index for fast searching