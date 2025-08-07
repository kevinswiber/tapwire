# Task 006: Implement Replay Command

## Status
- **Phase**: 2 (Core Features)
- **Priority**: High  
- **Status**: Not Started
- **Depends on**: Task 005 (Record Command) ✅ Complete

## Overview
Implement the `shadowcat replay` command to enable playback of recorded MCP tapes through an HTTP server. This completes the record/replay functionality that is core to Shadowcat's value proposition.

## Context
The [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md) identified replay as a stub. Task 005 successfully implemented the record command, creating test tapes that can be replayed.

## Current State
**File**: `src/main.rs` (post-Task 005)
```rust
Commands::Replay {
    tape_file: _,
    port: _,
} => {
    error!("Replay not yet implemented");
    exit(1);
}
```

### Existing Infrastructure (Ready to Use)
- **TapePlayer**: `src/recorder/replay.rs` - Complete playback engine with speed control, seeking
- **ReplayTransport**: `src/transport/replay.rs` - Transport that can replay tape data  
- **Tape Loading**: `TapeRecorder::load_tape()` - Loads tapes from storage
- **Test Tapes**: Multiple recorded tapes available in `tapes/` directory

### Working Record Command (from Task 005)
```bash
# These commands work and create tapes for replay
shadowcat record stdio --output demo.tape --name "Demo" -- echo '{"jsonrpc":"2.0","method":"ping","id":1}'
shadowcat record http --output http.tape --port 8081
shadowcat tape list  # Shows: 3 tapes available for replay
```

## Requirements

### Core Functionality (Simplified for Task 006)
1. **CLI Integration**: Enhance existing `shadowcat replay <tape-file> --port <port>` command
2. **Tape Loading**: Load tape files from storage directory (by ID or file path)
3. **HTTP Server**: Create HTTP server that serves replayed MCP responses
4. **Basic Playback**: Replay requests/responses with timing preservation
5. **Error Handling**: Robust error handling for missing/corrupt tapes

### CLI Interface (enhance existing)
```bash
# Basic replay
shadowcat replay <tape-id-or-file> --port <port>

# Examples  
shadowcat replay ef510f7f-1de3-426e-b3b6-66f0b16141d6 --port 8080
shadowcat replay ./tapes/demo.json --port 8081
```

### Success Criteria
- [ ] `shadowcat replay --help` shows comprehensive usage information
- [ ] `shadowcat replay <tape-id> --port 8080` starts HTTP server replaying tape
- [ ] `shadowcat replay <file-path> --port 8080` works with file paths
- [ ] HTTP requests receive responses from the replayed tape data
- [ ] Server handles missing/invalid tapes gracefully
- [ ] Integration tests demonstrate end-to-end record -> replay flow
- [ ] All existing tests still pass
- [ ] `cargo fmt` and `cargo clippy -- -D warnings` pass

## Implementation Strategy

### Phase A: CLI Enhancement
1. Update `Commands::Replay` args to match requirements
2. Implement `run_replay_server()` function using existing patterns from Task 005

### Phase B: Core Replay Logic
1. Use existing `TapeRecorder::load_tape()` to load tapes
2. Create HTTP server using axum (same as record command)
3. Use `TapePlayer` for playback control and timing
4. Handle tape ID vs file path resolution

### Phase C: Integration & Testing
1. Test with tapes created by record command
2. Add integration tests demonstrating record -> replay flow
3. Add error handling tests

## Verification Commands
```bash
# Create test tape (using working record command)
shadowcat record stdio --output test-replay.tape --name "Replay Test" -- echo '{"jsonrpc":"2.0","method":"ping","id":1}'

# Get tape ID for testing
shadowcat tape list

# Replay the tape by ID
shadowcat replay <tape-id> --port 8080 &

# Test the replayed endpoint  
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"ping","id":1}' http://localhost:8080/

# Run tests
cargo test --test integration_replay
cargo test
cargo clippy -- -D warnings
```

## Context for Next Session
- Record command is fully working and tested (Task 005 complete)
- Multiple test tapes available in `tapes/` directory
- All 349 tests currently passing
- Clean clippy output
- Existing replay infrastructure needs to be connected to CLI
- This task completes the core record/replay functionality

---

## Advanced Implementation Design (Suggestions from Previous Iteration)

*Note: The following are detailed design suggestions from an earlier iteration. For Task 006, focus on the simplified requirements above, but these can be used as reference for future enhancements.*

### Command Interface (Advanced - Future Enhancement)

```rust
#[derive(Parser)]
pub struct ReplayCommand {
    /// Tape file to replay
    tape_file: PathBuf,
    
    /// Replay speed multiplier (1.0 = realtime, 2.0 = 2x speed)
    #[arg(short, long, default_value = "1.0")]
    speed: f64,
    
    /// Replay mode
    #[arg(short, long, default_value = "both")]
    mode: ReplayMode,
    
    /// Target transport
    #[command(subcommand)]
    transport: TransportType,
    
    /// Interactive mode (step through messages)
    #[arg(short, long)]
    interactive: bool,
    
    /// Start from specific frame index
    #[arg(long)]
    start_frame: Option<usize>,
    
    /// Stop at specific frame index
    #[arg(long)]
    end_frame: Option<usize>,
    
    /// Filter methods to replay
    #[arg(long)]
    filter_methods: Vec<String>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ReplayMode {
    /// Replay all messages
    Both,
    /// Only replay client messages
    Client,
    /// Only replay server messages
    Server,
}
```

### Replay Engine (Advanced - Future Enhancement)

**File**: `src/recorder/replay_engine.rs` (future enhancement)

```rust
use crate::recorder::tape::{Tape, Frame};
use crate::transport::{Transport, TransportMessage};
use tokio::time::{sleep, Duration, Instant};
use std::io::{self, Write};

pub struct ReplayEngine {
    tape: Tape,
    config: ReplayConfig,
    transport: Box<dyn Transport>,
    current_frame: usize,
}

impl ReplayEngine {
    pub async fn new(
        tape_path: &Path,
        config: ReplayConfig,
        transport: Box<dyn Transport>,
    ) -> Result<Self, ReplayError> {
        // Load and validate tape
        let tape = Tape::load_from_file(tape_path).await?;
        
        // Validate tape integrity
        tape.validate()
            .map_err(|e| ReplayError::InvalidTape(e.to_string()))?;
        
        Ok(Self {
            tape,
            config,
            transport,
            current_frame: config.start_frame.unwrap_or(0),
        })
    }
    
    pub async fn replay(&mut self) -> Result<(), ReplayError> {
        info!("Starting replay of {} frames", self.tape.frames.len());
        
        // Connect transport
        self.transport.connect().await
            .map_err(|e| ReplayError::TransportError(e.to_string()))?;
        
        let start_time = Instant::now();
        let mut last_timestamp = self.tape.frames.first()
            .map(|f| f.timestamp)
            .unwrap_or(0);
        
        // Iterate through frames
        let end_frame = self.config.end_frame
            .unwrap_or(self.tape.frames.len());
        
        while self.current_frame < end_frame {
            let frame = &self.tape.frames[self.current_frame];
            
            // Apply filters
            if !self.should_replay(frame) {
                self.current_frame += 1;
                continue;
            }
            
            // Handle timing
            if !self.config.interactive {
                let delay = self.calculate_delay(frame.timestamp, last_timestamp);
                if delay > Duration::ZERO {
                    sleep(delay).await;
                }
            }
            
            // Interactive mode
            if self.config.interactive {
                self.interactive_prompt(frame).await?;
            }
            
            // Send message
            self.send_frame(frame).await?;
            
            last_timestamp = frame.timestamp;
            self.current_frame += 1;
            
            // Progress reporting
            if self.current_frame % 10 == 0 {
                self.report_progress();
            }
        }
        
        info!("Replay completed in {:?}", start_time.elapsed());
        Ok(())
    }
    
    fn should_replay(&self, frame: &Frame) -> bool {
        // Check replay mode
        match self.config.mode {
            ReplayMode::Client if frame.direction != Direction::FromClient => return false,
            ReplayMode::Server if frame.direction != Direction::FromServer => return false,
            ReplayMode::Both => {},
        }
        
        // Check method filter
        if !self.config.filter_methods.is_empty() {
            if let Some(method) = frame.message.method() {
                if !self.config.filter_methods.contains(&method.to_string()) {
                    return false;
                }
            }
        }
        
        true
    }
    
    fn calculate_delay(&self, current: u64, last: u64) -> Duration {
        let original_delay_ms = current.saturating_sub(last);
        let adjusted_delay_ms = (original_delay_ms as f64 / self.config.speed) as u64;
        Duration::from_millis(adjusted_delay_ms)
    }
    
    async fn send_frame(&mut self, frame: &Frame) -> Result<(), ReplayError> {
        debug!("Replaying frame {}: {:?}", self.current_frame, frame.direction);
        
        self.transport.send(frame.message.clone()).await
            .map_err(|e| ReplayError::TransportError(e.to_string()))?;
        
        // Log the replayed message
        if self.config.verbose {
            println!("[{}] {} {:?}", 
                self.current_frame,
                frame.direction,
                frame.message
            );
        }
        
        Ok(())
    }
    
    async fn interactive_prompt(&self, frame: &Frame) -> Result<(), ReplayError> {
        println!("\n--- Frame {} ---", self.current_frame);
        println!("Direction: {:?}", frame.direction);
        println!("Method: {:?}", frame.message.method());
        println!("Timestamp: {}", frame.timestamp);
        
        print!("Press Enter to continue, 's' to skip, 'q' to quit: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .map_err(|e| ReplayError::IoError(e.to_string()))?;
        
        match input.trim() {
            "q" => return Err(ReplayError::UserAbort),
            "s" => return Ok(()),
            _ => {},
        }
        
        Ok(())
    }
    
    fn report_progress(&self) {
        let percent = (self.current_frame as f64 / self.tape.frames.len() as f64) * 100.0;
        info!("Replay progress: {}/{} frames ({:.1}%)", 
            self.current_frame, 
            self.tape.frames.len(),
            percent
        );
    }
}
```

### Tape Validation (Advanced - Future Enhancement)

**File**: `src/recorder/tape.rs` (future enhancement)

```rust
impl Tape {
    pub async fn load_from_file(path: &Path) -> Result<Self, RecorderError> {
        let contents = tokio::fs::read_to_string(path).await
            .map_err(|e| RecorderError::IoError(e.to_string()))?;
        
        serde_json::from_str(&contents)
            .map_err(|e| RecorderError::DeserializationError(e.to_string()))
    }
    
    pub fn validate(&self) -> Result<(), String> {
        // Check tape has frames
        if self.frames.is_empty() {
            return Err("Tape contains no frames".to_string());
        }
        
        // Check timestamps are monotonic
        let mut last_timestamp = 0;
        for (i, frame) in self.frames.iter().enumerate() {
            if frame.timestamp < last_timestamp {
                return Err(format!(
                    "Non-monotonic timestamp at frame {}: {} < {}",
                    i, frame.timestamp, last_timestamp
                ));
            }
            last_timestamp = frame.timestamp;
        }
        
        // Validate session IDs are consistent
        let session_id = &self.session_id;
        for (i, frame) in self.frames.iter().enumerate() {
            if frame.session_id != *session_id {
                return Err(format!(
                    "Inconsistent session ID at frame {}",
                    i
                ));
            }
        }
        
        // Check for required metadata
        if !self.metadata.contains_key("version") {
            return Err("Missing version in metadata".to_string());
        }
        
        Ok(())
    }
}
```

### CLI Implementation (Advanced - Future Enhancement)

**File**: `src/main.rs` (future enhancement)

```rust
Commands::Replay {
    tape_file,
    speed,
    mode,
    transport,
    interactive,
    start_frame,
    end_frame,
    filter_methods,
} => {
    let config = ReplayConfig {
        speed,
        mode,
        interactive,
        start_frame,
        end_frame,
        filter_methods,
        verbose: args.verbose,
    };
    
    // Create transport
    let transport: Box<dyn Transport> = match transport {
        TransportType::Stdio => {
            Box::new(StdioTransport::new(Default::default()))
        },
        TransportType::Http { port } => {
            Box::new(HttpTransport::new(port))
        },
    };
    
    // Create and run replay engine
    let mut engine = ReplayEngine::new(&tape_file, config, transport).await?;
    engine.replay().await?;
    
    info!("Replay completed successfully");
}
```

## Usage Examples (Advanced - Future Enhancement)

```bash
# Basic replay
shadowcat replay session.tape stdio

# Replay at 2x speed
shadowcat replay -s 2.0 session.tape stdio

# Interactive stepping
shadowcat replay -i session.tape stdio

# Replay only client messages
shadowcat replay -m client session.tape stdio

# Replay specific range
shadowcat replay --start-frame 10 --end-frame 50 session.tape stdio

# Filter specific methods
shadowcat replay --filter-methods initialize,execute session.tape stdio

# Replay to HTTP endpoint
shadowcat replay session.tape http --port 8080
```

## Testing (Advanced - Future Enhancement)

```rust
#[tokio::test]
async fn test_replay_engine() {
    // Create test tape
    let tape = create_test_tape();
    let tape_path = save_test_tape(&tape).await;
    
    // Create mock transport
    let mut mock_transport = MockTransport::new();
    mock_transport.expect_send()
        .times(tape.frames.len())
        .returning(|_| Ok(()));
    
    // Replay
    let config = ReplayConfig::default();
    let mut engine = ReplayEngine::new(
        &tape_path,
        config,
        Box::new(mock_transport)
    ).await.unwrap();
    
    engine.replay().await.unwrap();
}

#[tokio::test]
async fn test_replay_with_filter() {
    let tape = create_test_tape_with_methods();
    let config = ReplayConfig {
        filter_methods: vec!["initialize".to_string()],
        ..Default::default()
    };
    
    // Should only replay initialize methods
    // ...
}

#[tokio::test]
async fn test_replay_timing() {
    // Test that replay respects timing
    let start = Instant::now();
    // ... replay with known delays ...
    let elapsed = start.elapsed();
    
    // Verify timing is correct
    assert!(elapsed >= expected_duration);
}
```

## Advanced Validation (Future Enhancement)

- [ ] Replay command works without "not implemented" error
- [ ] Tapes load and validate correctly
- [ ] Messages replay in correct order
- [ ] Timing is preserved (adjustable by speed)
- [ ] Interactive mode allows stepping
- [ ] Filtering works correctly
- [ ] Progress reporting is accurate

## Advanced Success Criteria (Future Enhancement)

- [ ] Can replay any valid tape file
- [ ] Replay timing matches original (with speed adjustment)
- [ ] All replay modes work (client/server/both)
- [ ] Interactive stepping works
- [ ] No message corruption during replay
- [ ] Memory usage stays bounded for large tapes
- [ ] Integration with record command verified