# Task 005: Implement Record Command

## Overview
Implement the `shadowcat record` command that is currently a stub, enabling recording of MCP sessions to tape files.

## Context
The [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md) identified that the record command exits immediately with "not yet implemented". This is a core advertised feature.

## Current State

**File**: `src/cli.rs:217-220`
```rust
Commands::Record { .. } => {
    eprintln!("Record command not yet implemented");
    std::process::exit(1);
}
```

## Requirements

1. Record all MCP messages passing through the proxy
2. Save to tape format (already defined in `src/recorder/tape.rs`)
3. Support both stdio and HTTP transports
4. Include timestamps and direction metadata
5. Allow filtering/sampling options
6. Provide tape file management (rotation, size limits)

## Implementation Design

### Command Interface

```rust
#[derive(Parser)]
pub struct RecordCommand {
    /// Output tape file path
    #[arg(short, long, default_value = "./recordings/tape-{timestamp}.json")]
    output: PathBuf,
    
    /// Transport type
    #[command(subcommand)]
    transport: TransportType,
    
    /// Maximum tape file size in MB
    #[arg(long, default_value = "100")]
    max_size: usize,
    
    /// Enable tape rotation
    #[arg(long)]
    rotate: bool,
    
    /// Filter: only record matching methods
    #[arg(long)]
    filter_methods: Vec<String>,
    
    /// Sampling rate (0.0-1.0, 1.0 = record all)
    #[arg(long, default_value = "1.0")]
    sample_rate: f64,
    
    /// Command to execute
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}
```

### Core Recording Logic

**File**: `src/recorder/engine.rs` (new)

```rust
use crate::recorder::tape::{Tape, Frame};
use crate::session::{SessionId, Direction};
use crate::transport::TransportMessage;
use tokio::sync::mpsc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct RecordingEngine {
    tape: Tape,
    output_path: PathBuf,
    config: RecordConfig,
    writer: Option<File>,
    frame_rx: mpsc::Receiver<Frame>,
    frame_tx: mpsc::Sender<Frame>,
}

impl RecordingEngine {
    pub async fn new(config: RecordConfig) -> Result<Self, RecorderError> {
        let (frame_tx, frame_rx) = mpsc::channel(1000);
        
        let tape = Tape::new(
            config.session_id.clone(),
            config.metadata.clone(),
        );
        
        let writer = File::create(&config.output_path).await
            .map_err(|e| RecorderError::IoError(e.to_string()))?;
        
        Ok(Self {
            tape,
            output_path: config.output_path.clone(),
            config,
            writer: Some(writer),
            frame_rx,
            frame_tx,
        })
    }
    
    pub fn get_frame_sender(&self) -> mpsc::Sender<Frame> {
        self.frame_tx.clone()
    }
    
    pub async fn start_recording(&mut self) -> Result<(), RecorderError> {
        info!("Starting recording to {:?}", self.output_path);
        
        // Write initial tape header
        self.write_header().await?;
        
        // Process frames
        while let Some(frame) = self.frame_rx.recv().await {
            if self.should_record(&frame) {
                self.record_frame(frame).await?;
                
                // Check rotation
                if self.should_rotate().await? {
                    self.rotate_tape().await?;
                }
            }
        }
        
        // Write tape footer
        self.finalize().await?;
        
        Ok(())
    }
    
    fn should_record(&self, frame: &Frame) -> bool {
        // Apply filtering
        if !self.config.filter_methods.is_empty() {
            if let Some(method) = frame.message.method() {
                if !self.config.filter_methods.contains(&method.to_string()) {
                    return false;
                }
            }
        }
        
        // Apply sampling
        if self.config.sample_rate < 1.0 {
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() > self.config.sample_rate {
                return false;
            }
        }
        
        true
    }
    
    async fn record_frame(&mut self, frame: Frame) -> Result<(), RecorderError> {
        self.tape.add_frame(frame.clone());
        
        // Write incrementally to avoid memory buildup
        if self.tape.frames.len() % 100 == 0 {
            self.flush_to_disk().await?;
        }
        
        Ok(())
    }
    
    async fn flush_to_disk(&mut self) -> Result<(), RecorderError> {
        if let Some(writer) = &mut self.writer {
            let json = serde_json::to_string_pretty(&self.tape)
                .map_err(|e| RecorderError::SerializationError(e.to_string()))?;
            
            writer.write_all(json.as_bytes()).await
                .map_err(|e| RecorderError::IoError(e.to_string()))?;
            
            writer.flush().await
                .map_err(|e| RecorderError::IoError(e.to_string()))?;
        }
        
        Ok(())
    }
    
    async fn should_rotate(&self) -> Result<bool, RecorderError> {
        if !self.config.rotate {
            return Ok(false);
        }
        
        let metadata = tokio::fs::metadata(&self.output_path).await
            .map_err(|e| RecorderError::IoError(e.to_string()))?;
        
        Ok(metadata.len() > self.config.max_size_bytes)
    }
    
    async fn rotate_tape(&mut self) -> Result<(), RecorderError> {
        // Finalize current tape
        self.finalize().await?;
        
        // Create new tape file
        let new_path = self.generate_rotation_path();
        self.writer = Some(File::create(&new_path).await
            .map_err(|e| RecorderError::IoError(e.to_string()))?);
        
        // Reset tape
        self.tape = Tape::new(
            SessionId::new(),
            self.config.metadata.clone(),
        );
        
        info!("Rotated tape to {:?}", new_path);
        
        Ok(())
    }
}
```

### Integration with Proxy

**File**: `src/proxy/recording_proxy.rs` (new)

```rust
use crate::proxy::{ForwardProxy, ReverseProxy};
use crate::recorder::RecordingEngine;

pub struct RecordingProxy<P> {
    inner_proxy: P,
    recorder: RecordingEngine,
}

impl<P: Proxy> RecordingProxy<P> {
    pub async fn new(inner_proxy: P, config: RecordConfig) -> Result<Self, Error> {
        let recorder = RecordingEngine::new(config).await?;
        Ok(Self {
            inner_proxy,
            recorder,
        })
    }
    
    pub async fn run(mut self) -> Result<(), Error> {
        let frame_sender = self.recorder.get_frame_sender();
        
        // Start recorder in background
        let recorder_handle = tokio::spawn(async move {
            self.recorder.start_recording().await
        });
        
        // Run proxy with frame capture
        self.inner_proxy.run_with_observer(move |message, direction| {
            let frame = Frame::new(
                message.clone(),
                direction,
            );
            
            // Non-blocking send
            if let Err(e) = frame_sender.try_send(frame) {
                warn!("Failed to record frame: {}", e);
            }
        }).await?;
        
        // Wait for recorder to finish
        recorder_handle.await??;
        
        Ok(())
    }
}
```

### CLI Implementation

**File**: `src/cli.rs` (update)

```rust
Commands::Record { 
    output,
    transport,
    max_size,
    rotate,
    filter_methods,
    sample_rate,
    command,
} => {
    let config = RecordConfig {
        output_path: expand_path(&output)?,
        max_size_bytes: max_size * 1024 * 1024,
        rotate,
        filter_methods,
        sample_rate,
        session_id: SessionId::new(),
        metadata: HashMap::from([
            ("command".to_string(), command.join(" ")),
            ("started_at".to_string(), Utc::now().to_rfc3339()),
        ]),
    };
    
    match transport {
        TransportType::Stdio => {
            let proxy = ForwardProxy::stdio(command)?;
            let recording_proxy = RecordingProxy::new(proxy, config).await?;
            recording_proxy.run().await?;
        },
        TransportType::Http { port } => {
            let proxy = ForwardProxy::http(port, command)?;
            let recording_proxy = RecordingProxy::new(proxy, config).await?;
            recording_proxy.run().await?;
        },
    }
    
    info!("Recording completed successfully");
}
```

## Usage Examples

```bash
# Basic recording
shadowcat record stdio -- mcp-server

# Record to specific file
shadowcat record -o session.tape stdio -- mcp-server

# Record with filtering
shadowcat record --filter-methods initialize,execute stdio -- mcp-server

# Record with rotation
shadowcat record --rotate --max-size 50 stdio -- mcp-server

# Record with sampling (10% of messages)
shadowcat record --sample-rate 0.1 stdio -- mcp-server

# Record HTTP transport
shadowcat record http --port 8080 -- mcp-server
```

## Testing

```rust
#[tokio::test]
async fn test_recording_engine() {
    let config = RecordConfig::default();
    let mut engine = RecordingEngine::new(config).await.unwrap();
    
    let sender = engine.get_frame_sender();
    
    // Send test frames
    let frame = Frame::new_test();
    sender.send(frame).await.unwrap();
    
    // Verify recording
    // ...
}

#[tokio::test]
async fn test_recording_with_filter() {
    let mut config = RecordConfig::default();
    config.filter_methods = vec!["initialize".to_string()];
    
    // Test that only initialize methods are recorded
    // ...
}

#[tokio::test]
async fn test_tape_rotation() {
    let mut config = RecordConfig::default();
    config.rotate = true;
    config.max_size_bytes = 1024; // Small size for testing
    
    // Test that tapes rotate when size exceeded
    // ...
}
```

## Integration Tests

```bash
#!/bin/bash
# Test recording with echo server
./target/debug/shadowcat record -o test.tape stdio -- echo '{"jsonrpc":"2.0","method":"test","id":1}'

# Verify tape file created
test -f test.tape || exit 1

# Verify tape contains expected content
grep '"method":"test"' test.tape || exit 1

# Test replay of recorded tape
./target/debug/shadowcat replay test.tape
```

## Validation

- [ ] Record command no longer shows "not implemented"
- [ ] Tape files are created successfully
- [ ] All messages are captured with correct metadata
- [ ] Filtering works as expected
- [ ] Rotation works when size limit reached
- [ ] Sampling reduces recorded messages appropriately
- [ ] Can replay recorded tapes

## Success Criteria

- [ ] End-to-end recording works for stdio transport
- [ ] End-to-end recording works for HTTP transport
- [ ] Tape format matches specification
- [ ] No message loss during recording
- [ ] Memory usage remains bounded
- [ ] Performance overhead <5%
- [ ] Integration tests pass