# Task 006: Implement Replay Command

## Overview
Implement the `shadowcat replay` command to replay recorded MCP session tapes.

## Context
The [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md) identified that replay is a stub. This feature is essential for debugging and testing.

## Current State

**File**: `src/cli.rs:221-224`
```rust
Commands::Replay { .. } => {
    eprintln!("Replay command not yet implemented");
    std::process::exit(1);
}
```

## Requirements

1. Read and parse tape files
2. Replay messages with original timing or custom speed
3. Support different replay modes (client-only, server-only, both)
4. Allow message filtering during replay
5. Support interactive stepping through messages
6. Validate tape integrity before replay

## Implementation Design

### Command Interface

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

### Replay Engine

**File**: `src/recorder/replay_engine.rs` (new)

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

### Tape Validation

**File**: `src/recorder/tape.rs` (update)

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

### CLI Implementation

**File**: `src/cli.rs` (update)

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

## Usage Examples

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

## Testing

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

## Validation

- [ ] Replay command works without "not implemented" error
- [ ] Tapes load and validate correctly
- [ ] Messages replay in correct order
- [ ] Timing is preserved (adjustable by speed)
- [ ] Interactive mode allows stepping
- [ ] Filtering works correctly
- [ ] Progress reporting is accurate

## Success Criteria

- [ ] Can replay any valid tape file
- [ ] Replay timing matches original (with speed adjustment)
- [ ] All replay modes work (client/server/both)
- [ ] Interactive stepping works
- [ ] No message corruption during replay
- [ ] Memory usage stays bounded for large tapes
- [ ] Integration with record command verified