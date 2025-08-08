# MCP-Aware Replay Specification

## Overview

This specification details the intelligent replay system for recorded MCP sessions, enabling debugging, testing, and analysis of MCP interactions.

## Replay Architecture

### Current: Binary Replay
```
[Binary Tape] → [Timestamp Ordering] → [Raw Playback]
```

### Target: Intelligent MCP Replay
```
[MCP Tape] → [Parse & Load] → [Transform] → [Schedule] → [Execute] → [Validate]
                    ↓              ↓           ↓           ↓           ↓
              [Session Setup] [Modify IDs] [Timing]  [Transport]  [Assert]
```

## Core Components

### 1. Replay Engine

**Purpose**: Orchestrate the replay of recorded MCP sessions

```rust
pub struct McpReplayEngine {
    tape: McpTape,
    config: ReplayConfig,
    transformer: MessageTransformer,
    scheduler: ReplayScheduler,
    executor: MessageExecutor,
    validator: ResponseValidator,
    state: ReplayState,
}

#[derive(Debug, Clone)]
pub struct ReplayConfig {
    // Timing control
    pub speed: ReplaySpeed,
    pub start_offset: Option<Duration>,
    pub end_offset: Option<Duration>,
    
    // Message filtering
    pub include_methods: Option<HashSet<String>>,
    pub exclude_methods: Option<HashSet<String>>,
    pub skip_notifications: bool,
    pub skip_errors: bool,
    
    // Transformation
    pub transform_ids: bool,
    pub update_timestamps: bool,
    pub parameter_overrides: HashMap<String, Value>,
    
    // Execution
    pub target: ReplayTarget,
    pub parallel_execution: bool,
    pub max_concurrent: usize,
    
    // Validation
    pub validate_responses: bool,
    pub strict_mode: bool,
    pub ignore_fields: Vec<String>,
    
    // Error handling
    pub on_error: ErrorStrategy,
    pub retry_failed: bool,
    pub max_retries: usize,
}

#[derive(Debug, Clone)]
pub enum ReplaySpeed {
    Realtime,                    // Original timing
    Scaled(f64),                 // 2.0 = 2x speed, 0.5 = half speed
    Fast,                        // As fast as possible
    Stepped,                     // Manual control
    Adaptive { target_rps: f64 }, // Maintain target requests/sec
}

#[derive(Debug, Clone)]
pub enum ReplayTarget {
    Transport(Box<dyn Transport>),
    Endpoint { url: String, headers: HeaderMap },
    MockServer(Arc<MockMcpServer>),
    Recorder(Box<dyn McpRecorder>),  // Re-record with modifications
}

#[derive(Debug, Clone)]
pub struct ReplayState {
    pub status: ReplayStatus,
    pub current_index: usize,
    pub messages_sent: usize,
    pub messages_received: usize,
    pub errors_encountered: Vec<ReplayError>,
    pub start_time: Instant,
    pub elapsed_time: Duration,
    pub pending_responses: HashMap<JsonRpcId, PendingResponse>,
}

impl McpReplayEngine {
    pub async fn load_tape(tape_id: TapeId) -> Result<Self> {
        let tape = load_tape_from_storage(tape_id).await?;
        let config = ReplayConfig::default();
        
        Ok(Self {
            tape,
            config,
            transformer: MessageTransformer::new(),
            scheduler: ReplayScheduler::new(),
            executor: MessageExecutor::new(),
            validator: ResponseValidator::new(),
            state: ReplayState::new(),
        })
    }
    
    pub async fn replay(&mut self) -> Result<ReplayReport> {
        // Initialize session
        self.initialize_session().await?;
        
        // Schedule messages
        let schedule = self.scheduler.create_schedule(&self.tape, &self.config)?;
        
        // Execute replay
        for scheduled_item in schedule {
            // Wait for scheduled time
            self.wait_for_scheduled_time(&scheduled_item).await;
            
            // Transform message
            let transformed = self.transformer
                .transform(scheduled_item.message.clone(), &self.state)
                .await?;
            
            // Execute message
            let result = self.executor
                .execute(transformed, &self.config.target)
                .await;
            
            // Handle result
            match result {
                Ok(response) => {
                    self.handle_response(response, &scheduled_item).await?;
                }
                Err(e) => {
                    self.handle_error(e, &scheduled_item).await?;
                }
            }
            
            // Update state
            self.state.current_index += 1;
            
            // Check for pause/stop
            if self.should_pause() {
                self.pause().await;
            }
            if self.should_stop() {
                break;
            }
        }
        
        // Finalize and generate report
        self.finalize_replay().await
    }
}
```

### 2. Message Transformer

**Purpose**: Modify messages during replay for testing scenarios

```rust
pub struct MessageTransformer {
    rules: Vec<TransformRule>,
    id_mapper: IdMapper,
    timestamp_adjuster: TimestampAdjuster,
}

#[derive(Debug, Clone)]
pub struct TransformRule {
    pub name: String,
    pub condition: TransformCondition,
    pub action: TransformAction,
}

#[derive(Debug, Clone)]
pub enum TransformCondition {
    Always,
    MethodEquals(String),
    MessageType(MessageType),
    Index(usize),
    Custom(Box<dyn Fn(&McpMessage) -> bool>),
}

#[derive(Debug, Clone)]
pub enum TransformAction {
    // ID management
    RegenerateId,
    MapId { from: JsonRpcId, to: JsonRpcId },
    
    // Parameter modification
    SetParam { path: String, value: Value },
    RemoveParam { path: String },
    ReplaceParam { path: String, pattern: Regex, replacement: String },
    
    // Result modification
    SetResult { path: String, value: Value },
    InjectError { code: i32, message: String },
    
    // Method changes
    RenameMethod { to: String },
    
    // Timing
    AddDelay(Duration),
    SetTimestamp(DateTime<Utc>),
}

pub struct IdMapper {
    mappings: HashMap<JsonRpcId, JsonRpcId>,
    generator: IdGenerator,
}

impl MessageTransformer {
    pub async fn transform(
        &mut self,
        mut message: McpMessage,
        state: &ReplayState,
    ) -> Result<McpMessage> {
        // Apply transformation rules
        for rule in &self.rules {
            if self.evaluate_condition(&rule.condition, &message) {
                message = self.apply_action(rule.action.clone(), message)?;
            }
        }
        
        // Update IDs if configured
        if self.should_transform_ids() {
            message = self.transform_ids(message)?;
        }
        
        // Adjust timestamps
        message = self.timestamp_adjuster.adjust(message, state.elapsed_time)?;
        
        Ok(message)
    }
    
    fn transform_ids(&mut self, message: McpMessage) -> Result<McpMessage> {
        match message {
            McpMessage::Single(JsonRpcMessage::V2(msg)) => {
                match msg {
                    JsonRpcV2Message::Request { id, method, params } => {
                        let new_id = self.id_mapper.get_or_create(id);
                        Ok(McpMessage::Single(JsonRpcMessage::V2(
                            JsonRpcV2Message::Request {
                                id: new_id,
                                method,
                                params,
                            }
                        )))
                    }
                    JsonRpcV2Message::Response { id, result, error } => {
                        let new_id = self.id_mapper.get_mapped(id)
                            .unwrap_or(id);
                        Ok(McpMessage::Single(JsonRpcMessage::V2(
                            JsonRpcV2Message::Response {
                                id: new_id,
                                result,
                                error,
                            }
                        )))
                    }
                    _ => Ok(McpMessage::Single(JsonRpcMessage::V2(msg)))
                }
            }
            McpMessage::Batch(messages) => {
                // Transform each message in batch
                let transformed = messages.into_iter()
                    .map(|m| self.transform_single(m))
                    .collect::<Result<Vec<_>>>()?;
                Ok(McpMessage::Batch(transformed))
            }
        }
    }
}
```

### 3. Replay Scheduler

**Purpose**: Control timing and ordering of replayed messages

```rust
pub struct ReplayScheduler {
    timing_strategy: TimingStrategy,
    ordering_strategy: OrderingStrategy,
}

#[derive(Debug, Clone)]
pub enum TimingStrategy {
    // Preserve original timing
    Original,
    
    // Scale all delays by factor
    Scaled { factor: f64 },
    
    // Fixed delay between messages
    Fixed { delay: Duration },
    
    // Adaptive based on response times
    Adaptive { target_rps: f64 },
    
    // Custom timing function
    Custom(Box<dyn Fn(&TapeEntry, &TapeEntry) -> Duration>),
}

#[derive(Debug, Clone)]
pub enum OrderingStrategy {
    // Original order from tape
    Sequential,
    
    // Group by correlation ID
    Correlated,
    
    // Prioritize by method
    Priority { method_priority: HashMap<String, i32> },
    
    // Random order (for chaos testing)
    Random { seed: Option<u64> },
}

#[derive(Debug, Clone)]
pub struct ScheduledMessage {
    pub index: usize,
    pub message: McpMessage,
    pub scheduled_time: Instant,
    pub original_timestamp: DateTime<Utc>,
    pub correlation_id: Option<CorrelationId>,
    pub dependencies: Vec<usize>,  // Indices of messages that must complete first
}

impl ReplayScheduler {
    pub fn create_schedule(
        &self,
        tape: &McpTape,
        config: &ReplayConfig,
    ) -> Result<Vec<ScheduledMessage>> {
        // Filter messages
        let filtered = self.filter_messages(&tape.entries, config)?;
        
        // Order messages
        let ordered = self.order_messages(filtered, &self.ordering_strategy)?;
        
        // Calculate timing
        let mut schedule = Vec::new();
        let mut last_time = Instant::now();
        
        for (i, entry) in ordered.iter().enumerate() {
            let delay = self.calculate_delay(
                i,
                entry,
                ordered.get(i.saturating_sub(1)),
                &self.timing_strategy,
            )?;
            
            last_time += delay;
            
            schedule.push(ScheduledMessage {
                index: i,
                message: entry.message.raw.clone(),
                scheduled_time: last_time,
                original_timestamp: entry.timestamp,
                correlation_id: entry.correlation_id.clone(),
                dependencies: self.find_dependencies(entry, &ordered[..i]),
            });
        }
        
        Ok(schedule)
    }
    
    fn find_dependencies(
        &self,
        entry: &TapeEntry,
        previous: &[TapeEntry],
    ) -> Vec<usize> {
        let mut deps = Vec::new();
        
        // Response depends on its request
        if let Some(correlation_id) = &entry.correlation_id {
            for (i, prev) in previous.iter().enumerate() {
                if prev.correlation_id.as_ref() == Some(correlation_id) {
                    deps.push(i);
                    break;
                }
            }
        }
        
        deps
    }
}
```

### 4. Response Validator

**Purpose**: Validate replayed responses against original recordings

```rust
pub struct ResponseValidator {
    rules: Vec<ValidationRule>,
    comparator: MessageComparator,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub name: String,
    pub applies_to: ValidationScope,
    pub validation_type: ValidationType,
    pub on_failure: FailureAction,
}

#[derive(Debug, Clone)]
pub enum ValidationScope {
    All,
    Method(String),
    MessageType(MessageType),
    Custom(Box<dyn Fn(&McpMessage) -> bool>),
}

#[derive(Debug, Clone)]
pub enum ValidationType {
    // Exact match
    Exact,
    
    // Structural match (same fields, different values ok)
    Structural,
    
    // Partial match (subset of fields)
    Partial { required_fields: Vec<String> },
    
    // Custom validation
    Custom(Box<dyn Fn(&McpMessage, &McpMessage) -> bool>),
}

#[derive(Debug, Clone)]
pub enum FailureAction {
    Ignore,
    Warn,
    Error,
    Retry,
    Skip,
}

pub struct ValidationResult {
    pub passed: bool,
    pub differences: Vec<Difference>,
    pub applied_rules: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Difference {
    pub path: String,
    pub expected: Value,
    pub actual: Value,
    pub difference_type: DifferenceType,
}

impl ResponseValidator {
    pub async fn validate(
        &self,
        expected: &McpMessage,
        actual: &McpMessage,
        context: &ValidationContext,
    ) -> ValidationResult {
        let mut result = ValidationResult {
            passed: true,
            differences: Vec::new(),
            applied_rules: Vec::new(),
        };
        
        // Apply validation rules
        for rule in &self.rules {
            if self.applies(&rule.applies_to, actual) {
                let validation = self.validate_with_rule(expected, actual, rule);
                
                if !validation.passed {
                    match rule.on_failure {
                        FailureAction::Ignore => {},
                        FailureAction::Warn => {
                            warn!("Validation warning: {}", rule.name);
                        }
                        FailureAction::Error => {
                            result.passed = false;
                        }
                        _ => {}
                    }
                }
                
                result.differences.extend(validation.differences);
                result.applied_rules.push(rule.name.clone());
            }
        }
        
        result
    }
}
```

### 5. Replay Controller

**Purpose**: Interactive control over replay execution

```rust
pub struct ReplayController {
    engine: Arc<Mutex<McpReplayEngine>>,
    control_rx: mpsc::Receiver<ControlCommand>,
    status_tx: mpsc::Sender<StatusUpdate>,
}

#[derive(Debug, Clone)]
pub enum ControlCommand {
    Start,
    Pause,
    Resume,
    Stop,
    Step,
    Seek { to_index: usize },
    SetSpeed(ReplaySpeed),
    AddBreakpoint { at_index: usize },
    RemoveBreakpoint { at_index: usize },
    InjectMessage(McpMessage),
    ModifyNext(TransformAction),
}

#[derive(Debug, Clone)]
pub struct StatusUpdate {
    pub status: ReplayStatus,
    pub progress: ReplayProgress,
    pub current_message: Option<McpMessage>,
    pub metrics: ReplayMetrics,
}

#[derive(Debug, Clone)]
pub struct ReplayProgress {
    pub current_index: usize,
    pub total_messages: usize,
    pub percent_complete: f64,
    pub elapsed_time: Duration,
    pub estimated_remaining: Duration,
}

impl ReplayController {
    pub async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                // Handle control commands
                Some(cmd) = self.control_rx.recv() => {
                    self.handle_command(cmd).await?;
                }
                
                // Process next message if running
                _ = self.process_next_message() => {
                    self.send_status_update().await?;
                }
                
                // Check for completion
                _ = self.check_completion() => {
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_command(&mut self, cmd: ControlCommand) -> Result<()> {
        let mut engine = self.engine.lock().await;
        
        match cmd {
            ControlCommand::Pause => {
                engine.state.status = ReplayStatus::Paused;
            }
            ControlCommand::Resume => {
                engine.state.status = ReplayStatus::Running;
            }
            ControlCommand::Stop => {
                engine.state.status = ReplayStatus::Stopped;
            }
            ControlCommand::Step => {
                if engine.state.status == ReplayStatus::Paused {
                    // Process one message
                    self.process_single_message(&mut engine).await?;
                }
            }
            ControlCommand::Seek { to_index } => {
                engine.state.current_index = to_index;
            }
            ControlCommand::SetSpeed(speed) => {
                engine.config.speed = speed;
            }
            // ... other commands
        }
        
        Ok(())
    }
}
```

## Replay Scenarios

### 1. Debugging Replay
```rust
let replay = McpReplayEngine::load_tape(tape_id).await?
    .with_config(ReplayConfig {
        speed: ReplaySpeed::Stepped,
        validate_responses: true,
        on_error: ErrorStrategy::Pause,
        ..Default::default()
    });

// Step through messages
controller.add_breakpoint(problematic_index);
controller.start().await;
```

### 2. Load Testing Replay
```rust
let replay = McpReplayEngine::load_tape(tape_id).await?
    .with_config(ReplayConfig {
        speed: ReplaySpeed::Scaled(10.0),  // 10x speed
        parallel_execution: true,
        max_concurrent: 100,
        validate_responses: false,
        ..Default::default()
    });

replay.replay().await?;
```

### 3. Chaos Testing Replay
```rust
let replay = McpReplayEngine::load_tape(tape_id).await?
    .with_config(ReplayConfig {
        speed: ReplaySpeed::Fast,
        // Inject random errors
        error_injection: Some(ErrorInjection {
            probability: 0.1,
            error_types: vec![
                ErrorType::Timeout,
                ErrorType::NetworkError,
                ErrorType::InvalidResponse,
            ],
        }),
        // Random ordering
        ordering: OrderingStrategy::Random { seed: Some(42) },
        ..Default::default()
    });
```

### 4. Regression Testing Replay
```rust
let replay = McpReplayEngine::load_tape(baseline_tape).await?
    .with_config(ReplayConfig {
        speed: ReplaySpeed::Fast,
        validate_responses: true,
        strict_mode: true,
        // Ignore timestamps and IDs in comparison
        ignore_fields: vec!["timestamp", "id"],
        ..Default::default()
    });

let report = replay.replay().await?;
assert!(report.all_validations_passed());
```

## Configuration

```yaml
replay:
  # Default configuration
  defaults:
    speed: realtime
    validate_responses: true
    transform_ids: true
    update_timestamps: true
  
  # Validation settings
  validation:
    enabled: true
    strict_mode: false
    ignore_fields:
      - timestamp
      - id
      - session_id
    tolerance:
      numeric: 0.01  # 1% tolerance for numbers
      timing: 100ms   # 100ms tolerance for durations
  
  # Transformation rules
  transformations:
    - name: "Update API endpoint"
      condition:
        method_equals: "tools/call"
      action:
        set_param:
          path: "$.endpoint"
          value: "https://new-api.example.com"
    
    - name: "Inject auth token"
      condition: always
      action:
        set_param:
          path: "$.auth"
          value: "${env.AUTH_TOKEN}"
  
  # Error handling
  error_handling:
    strategy: continue  # continue, pause, stop, retry
    max_retries: 3
    retry_delay: 1s
    
  # Performance
  performance:
    parallel_execution: false
    max_concurrent: 10
    buffer_size: 100
```

## Export and Import

### Export for External Tools
```rust
pub trait ReplayExporter {
    async fn export(&self, tape: &McpTape, format: ExportFormat) -> Result<Vec<u8>>;
}

pub enum ExportFormat {
    Postman,      // Postman collection
    Insomnia,     // Insomnia workspace
    K6Script,     // k6 load testing script
    Playwright,   // Playwright test
    OpenAPI,      // OpenAPI specification
    Custom(String),
}
```

### Import from External Sources
```rust
pub trait ReplayImporter {
    async fn import(&self, data: &[u8], format: ImportFormat) -> Result<McpTape>;
}

pub enum ImportFormat {
    HarFile,      // HTTP Archive
    PcapFile,     // Packet capture
    JsonLines,    // JSON Lines format
    Custom(String),
}
```

## Metrics and Reporting

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ReplayReport {
    pub summary: ReplaySummary,
    pub performance: PerformanceMetrics,
    pub validation: ValidationSummary,
    pub errors: Vec<ReplayError>,
    pub timeline: Vec<TimelineEvent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplaySummary {
    pub tape_id: TapeId,
    pub total_messages: usize,
    pub messages_sent: usize,
    pub messages_skipped: usize,
    pub duration: Duration,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceMetrics {
    pub avg_response_time: Duration,
    pub p50_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
    pub throughput: f64,  // messages/sec
    pub error_rate: f64,
}
```

## Future Enhancements

1. **Distributed Replay**
   - Coordinate replay across multiple nodes
   - Simulate realistic load patterns

2. **AI-Powered Replay**
   - Generate variations of recorded sessions
   - Intelligent test case generation

3. **Visual Replay**
   - Timeline visualization
   - Real-time replay monitoring
   - Diff visualization for validations

4. **Contract Testing**
   - Generate contracts from recordings
   - Validate API compatibility