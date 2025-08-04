# Task 005: Tape Replay Engine

**File:** `src/recorder/replay.rs`  
**Estimated Effort:** 2 days  
**Priority:** High  
**Dependencies:** Phase 2 TapeRecorder, Session Management

---

## Overview

Implement a comprehensive tape replay engine that can deterministically replay recorded MCP sessions with accurate timing, speed controls, and debugging capabilities.

---

## Requirements

### Core Functionality
1. **TapePlayer Struct**: Main replay engine coordinating playback
2. **Timing Control**: Support variable speed playback (0.1x to 10x)
3. **State Management**: Pause, resume, seek, and step operations
4. **Frame Delivery**: Accurate frame timing and ordering
5. **Progress Tracking**: Monitor replay position and remaining time

### Advanced Features
1. **Frame-by-Frame Stepping**: Manual frame advancement for debugging
2. **Timeline Navigation**: Jump to specific timestamps or frame indices
3. **Replay Validation**: Verify tape integrity before playback
4. **Memory Efficiency**: Stream large tapes without loading entirely
5. **Error Recovery**: Handle corrupted frames gracefully

---

## Technical Specification

### TapePlayer API
```rust
pub struct TapePlayer {
    tape: Tape,
    current_position: usize,
    start_time: Option<SystemTime>,
    speed: f64,
    state: PlaybackState,
    frame_tx: Option<mpsc::Sender<Frame>>,
    control_rx: mpsc::Receiver<PlaybackControl>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Stepping,
    Completed,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum PlaybackControl {
    Play,
    Pause,
    Stop,
    SetSpeed(f64),
    Seek(SeekTarget),
    Step,
}

#[derive(Debug, Clone)]
pub enum SeekTarget {
    Frame(usize),
    Timestamp(u64),
    Percent(f64),
}

impl TapePlayer {
    pub fn new(tape: Tape) -> Self;
    pub async fn start(&mut self) -> Result<(), ReplayError>;
    pub async fn control(&mut self, command: PlaybackControl) -> Result<(), ReplayError>;
    pub fn state(&self) -> &PlaybackState;
    pub fn progress(&self) -> PlaybackProgress;
    pub fn set_frame_sender(&mut self, tx: mpsc::Sender<Frame>);
}

#[derive(Debug, Clone)]
pub struct PlaybackProgress {
    pub current_frame: usize,
    pub total_frames: usize,
    pub current_timestamp: u64,
    pub total_duration: u64,
    pub speed: f64,
    pub elapsed_real_time: Duration,
    pub estimated_remaining: Duration,
}
```

### Timing Strategy
```rust
struct TimingEngine {
    tape_start_time: u64,
    real_start_time: SystemTime,
    speed_multiplier: f64,
    paused_duration: Duration,
}

impl TimingEngine {
    fn calculate_next_delay(&self, current_frame: &Frame, next_frame: &Frame) -> Duration;
    fn adjust_for_speed(&self, original_delay: Duration) -> Duration;
    fn handle_pause(&mut self) -> Duration;
}
```

---

## Implementation Plan

### Day 1: Core Replay Engine

#### Morning: Basic Structure
```rust
// 1. Create basic TapePlayer struct
pub struct TapePlayer {
    tape: Tape,
    current_position: usize,
    state: PlaybackState,
    // ... other fields
}

// 2. Implement constructor and basic state management
impl TapePlayer {
    pub fn new(tape: Tape) -> Self {
        Self {
            tape,
            current_position: 0,
            state: PlaybackState::Stopped,
            // ... initialize other fields
        }
    }
    
    pub fn state(&self) -> &PlaybackState {
        &self.state
    }
}
```

#### Afternoon: Basic Playback
```rust
// 3. Implement basic frame-by-frame playback
impl TapePlayer {
    pub async fn play(&mut self) -> Result<(), ReplayError> {
        self.state = PlaybackState::Playing;
        
        while self.current_position < self.tape.frames.len() {
            let frame = &self.tape.frames[self.current_position];
            
            // Send frame to consumer
            if let Some(tx) = &self.frame_tx {
                tx.send(frame.clone()).await?;
            }
            
            self.current_position += 1;
            
            // Basic timing (no speed control yet)
            if let Some(next_frame) = self.tape.frames.get(self.current_position) {
                let delay = next_frame.timestamp - frame.timestamp;
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }
        
        self.state = PlaybackState::Completed;
        Ok(())
    }
}
```

### Day 2: Advanced Controls & Timing

#### Morning: Speed Control & Timing Engine
```rust
// 4. Implement sophisticated timing engine
struct TimingEngine {
    tape_start_time: u64,
    real_start_time: SystemTime,
    speed_multiplier: f64,
    paused_duration: Duration,
}

impl TimingEngine {
    fn calculate_next_delay(&self, current_frame: &Frame, next_frame: &Frame) -> Duration {
        let tape_delay = Duration::from_millis(next_frame.timestamp - current_frame.timestamp);
        Duration::from_secs_f64(tape_delay.as_secs_f64() / self.speed_multiplier)
    }
}

// 5. Add speed control to TapePlayer
impl TapePlayer {
    pub fn set_speed(&mut self, speed: f64) -> Result<(), ReplayError> {
        if speed <= 0.0 || speed > 10.0 {
            return Err(ReplayError::InvalidSpeed(speed));
        }
        self.speed = speed;
        Ok(())
    }
}
```

#### Afternoon: Control Commands & State Management
```rust
// 6. Implement pause/resume/seek functionality
impl TapePlayer {
    pub async fn pause(&mut self) -> Result<(), ReplayError> {
        match self.state {
            PlaybackState::Playing => {
                self.state = PlaybackState::Paused;
                // Record pause time for timing adjustments
                Ok(())
            }
            _ => Err(ReplayError::InvalidStateTransition)
        }
    }
    
    pub async fn seek(&mut self, target: SeekTarget) -> Result<(), ReplayError> {
        let new_position = match target {
            SeekTarget::Frame(idx) => idx,
            SeekTarget::Timestamp(ts) => self.find_frame_by_timestamp(ts)?,
            SeekTarget::Percent(pct) => ((self.tape.frames.len() as f64) * pct) as usize,
        };
        
        if new_position >= self.tape.frames.len() {
            return Err(ReplayError::SeekOutOfBounds(new_position));
        }
        
        self.current_position = new_position;
        Ok(())
    }
}
```

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Frame, Session};
    use crate::transport::{Direction, SessionId, TransportMessage, TransportType};
    use serde_json::json;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_tape_player_creation() {
        let tape = create_test_tape();
        let player = TapePlayer::new(tape);
        assert_eq!(player.state(), &PlaybackState::Stopped);
        assert_eq!(player.current_position, 0);
    }

    #[tokio::test]
    async fn test_basic_playback() {
        let tape = create_test_tape_with_timing();
        let mut player = TapePlayer::new(tape);
        
        let (tx, mut rx) = mpsc::channel(100);
        player.set_frame_sender(tx);
        
        let start_time = std::time::Instant::now();
        player.play().await.unwrap();
        let elapsed = start_time.elapsed();
        
        // Verify timing accuracy (within 10ms tolerance)
        assert!(elapsed.as_millis() >= 90 && elapsed.as_millis() <= 110);
        assert_eq!(player.state(), &PlaybackState::Completed);
    }

    #[tokio::test]
    async fn test_speed_control() {
        let tape = create_test_tape_with_timing();
        let mut player = TapePlayer::new(tape);
        player.set_speed(2.0).unwrap(); // 2x speed
        
        let start_time = std::time::Instant::now();
        player.play().await.unwrap();
        let elapsed = start_time.elapsed();
        
        // Should complete in ~50ms at 2x speed
        assert!(elapsed.as_millis() >= 40 && elapsed.as_millis() <= 60);
    }

    #[tokio::test]
    async fn test_pause_resume() {
        let tape = create_test_tape_with_timing();
        let mut player = TapePlayer::new(tape);
        
        // Start playback
        let play_task = tokio::spawn(async move {
            player.play().await
        });
        
        // Pause after 25ms
        sleep(Duration::from_millis(25)).await;
        player.pause().await.unwrap();
        assert_eq!(player.state(), &PlaybackState::Paused);
        
        // Resume and complete
        sleep(Duration::from_millis(50)).await;
        player.resume().await.unwrap();
        
        play_task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_seek_functionality() {
        let tape = create_test_tape_with_10_frames();
        let mut player = TapePlayer::new(tape);
        
        // Seek to frame 5
        player.seek(SeekTarget::Frame(5)).await.unwrap();
        assert_eq!(player.current_position, 5);
        
        // Seek to 50% (frame 5)
        player.seek(SeekTarget::Percent(0.5)).await.unwrap();
        assert_eq!(player.current_position, 5);
        
        // Seek to specific timestamp
        player.seek(SeekTarget::Timestamp(1500)).await.unwrap();
        // Verify position is correct for timestamp
    }

    fn create_test_tape() -> crate::recorder::Tape {
        // Create a simple tape with 3 frames for testing
    }

    fn create_test_tape_with_timing() -> crate::recorder::Tape {
        // Create tape with frames at 0ms, 50ms, 100ms timestamps
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_replay_integration_with_transport() {
    // Test that replayed frames can be consumed by a Transport
    let tape = create_recorded_tape();
    let mut player = TapePlayer::new(tape);
    
    let (frame_tx, frame_rx) = mpsc::channel(100);
    player.set_frame_sender(frame_tx);
    
    // Create a consumer that validates frame order and timing
    let consumer_task = tokio::spawn(async move {
        // Consume frames and validate
    });
    
    player.play().await.unwrap();
    consumer_task.await.unwrap();
}
```

---

## Error Handling

### Error Types
```rust
#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("Invalid speed: {0}. Must be between 0.1 and 10.0")]
    InvalidSpeed(f64),
    
    #[error("Seek position {0} is out of bounds")]
    SeekOutOfBounds(usize),
    
    #[error("Invalid state transition from {current:?} to {requested:?}")]
    InvalidStateTransition { current: PlaybackState, requested: PlaybackState },
    
    #[error("Tape validation failed: {0}")]
    TapeValidation(String),
    
    #[error("Frame delivery failed: {0}")]
    FrameDelivery(String),
    
    #[error("Timing synchronization lost: {0}")]
    TimingSync(String),
}
```

### Error Recovery
- Validate tape integrity before starting playback
- Handle missing or corrupted frames gracefully
- Provide fallback timing when timestamps are invalid
- Allow playback to continue with warnings for non-critical errors

---

## Performance Considerations

### Memory Optimization
- Stream large tapes instead of loading entirely into memory
- Use frame references instead of cloning when possible
- Implement frame buffering for smooth playback
- Clean up completed frames promptly

### Timing Accuracy
- Use high-resolution timing for sub-millisecond accuracy
- Compensate for system scheduling delays
- Implement timing drift detection and correction
- Provide timing statistics for validation

### Scalability
- Support tapes with 10,000+ frames efficiently
- Implement lazy loading for tape metadata
- Use indexes for fast seeking operations
- Minimize allocation during playback

---

## Success Criteria

### Functional Requirements
- [x] TapePlayer can replay recorded tapes accurately
- [x] Speed control works from 0.1x to 10x without timing issues
- [x] Pause/resume maintains timing accuracy
- [x] Seek operations work for frame, timestamp, and percentage targets
- [x] Frame-by-frame stepping works for debugging

### Performance Requirements
- [x] Timing accuracy within ±10ms for normal speeds (0.5x to 2x)
- [x] Memory usage under 50MB for tapes with 1000 frames
- [x] Seek operations complete within 100ms
- [x] Can handle tapes with 10,000+ frames without performance degradation

### Quality Requirements
- [x] Comprehensive test coverage (>90%)
- [x] Error handling provides clear, actionable messages
- [x] API is intuitive and well-documented
- [x] Integration tests validate end-to-end functionality

---

## Future Enhancements

### Advanced Features
- Multiple tape synchronization for complex scenarios
- Real-time tape modification during playback
- Interactive debugging with breakpoints
- Performance profiling and optimization suggestions

### Integration Points
- Hook points for interceptor integration (Phase 4)
- Plugin system for custom frame processors
- Integration with external debugging tools
- Export replay statistics and performance metrics

This task establishes the foundation for all replay functionality in Shadowcat, enabling powerful debugging and testing capabilities for MCP applications.