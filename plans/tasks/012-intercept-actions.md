# Task 012: Intercept Actions Implementation

**Phase:** 4 - Interception & Rule Engine  
**Priority:** High  
**Estimated Effort:** 2 days  
**Assignee:** Development Team  
**Status:** Not Started

---

## Overview

Implement the comprehensive action system that enables powerful manipulation of intercepted MCP messages. This includes message modification, mock responses, fault injection, conditional execution, and action composition. The action system integrates with the rule engine to provide automated interception capabilities.

## Objectives

- Implement all InterceptAction types from the interceptor engine
- Create message modification framework with JSON path editing
- Add mock response generation with templating support
- Support fault injection for testing resilience
- Enable conditional action execution based on runtime context
- Provide action composition and chaining capabilities

## Technical Requirements

### Core Components

#### 1. Action Specification
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSpec {
    pub id: String,
    pub action_type: ActionType,
    pub description: Option<String>,
    pub enabled: bool,
    pub timeout_ms: Option<u64>,
    pub retry_config: Option<RetryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    // Message Flow Control
    Continue,
    Block { 
        reason: String, 
        error_code: Option<i32>,
        include_original: bool,
    },
    Pause { 
        message: Option<String>,
        timeout_ms: Option<u64>,
        auto_resume: bool,
    },
    
    // Message Modification
    Modify { 
        changes: Vec<MessageChange>,
        preserve_id: bool,
    },
    Transform { 
        transformer: TransformSpec,
    },
    
    // Response Generation  
    Mock { 
        response: MockResponse,
        delay_ms: Option<u64>,
    },
    Redirect {
        target: RedirectTarget,
        preserve_context: bool,
    },
    
    // Testing & Debugging
    Delay { 
        duration_ms: u64,
        jitter_percent: Option<u8>,
    },
    Fault { 
        fault_type: FaultType,
        probability: f32,
    },
    Log { 
        level: LogLevel,
        template: String,
        include_message: bool,
    },
    
    // Advanced Composition
    Chain { 
        actions: Vec<ActionSpec>,
        stop_on_error: bool,
    },
    Conditional { 
        condition: RuntimeCondition,
        then_action: Box<ActionSpec>,
        else_action: Option<Box<ActionSpec>>,
    },
    Parallel {
        actions: Vec<ActionSpec>,
        wait_for_all: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageChange {
    SetField { 
        path: String, 
        value: serde_json::Value,
        create_path: bool,
    },
    RemoveField { 
        path: String,
        ignore_missing: bool,
    },
    AddField { 
        path: String, 
        key: String,
        value: serde_json::Value,
    },
    Transform { 
        path: String, 
        transform: FieldTransform,
    },
    ReplaceMessage {
        new_message: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldTransform {
    StringReplace { pattern: String, replacement: String },
    StringFormat { template: String },
    NumberAdd { value: f64 },
    NumberMultiply { factor: f64 },
    ArrayAppend { value: serde_json::Value },
    ArrayPrepend { value: serde_json::Value },
    ArrayRemoveIndex { index: usize },
    ObjectMerge { merge_with: serde_json::Value },
}
```

#### 2. Mock Response System
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MockResponse {
    Static {
        response: serde_json::Value,
    },
    Template {
        template: String,
        context: HashMap<String, serde_json::Value>,
    },
    File {
        file_path: String,
        template_vars: HashMap<String, String>,
    },
    Generate {
        generator: ResponseGenerator,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseGenerator {
    Success {
        result_template: Option<String>,
    },
    Error {
        code: i32,
        message: String,
        data: Option<serde_json::Value>,
    },
    Random {
        responses: Vec<serde_json::Value>,
        weights: Option<Vec<f32>>,
    },
    Sequence {
        responses: Vec<serde_json::Value>,
        repeat: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay_ms: u64,
    pub backoff_multiplier: f32,
    pub max_delay_ms: u64,
}
```

#### 3. Fault Injection
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultType {
    NetworkTimeout {
        timeout_ms: u64,
    },
    NetworkError {
        error_type: NetworkErrorType,
    },
    MalformedResponse {
        corruption_type: CorruptionType,
    },
    SlowResponse {
        delay_ms: u64,
        delay_type: DelayType,
    },
    MemoryExhaustion {
        allocation_mb: usize,
    },
    ProcessCrash {
        signal: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkErrorType {
    ConnectionRefused,
    ConnectionReset,
    UnreachableHost,
    DnsFailure,
    SslError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorruptionType {
    TruncateResponse { at_byte: usize },
    InvalidJson { error_type: String },
    WrongContentType,
    MissingHeaders { headers: Vec<String> },
    InvalidEncoding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelayType {
    Fixed,
    Random { min_ms: u64, max_ms: u64 },
    Exponential { base_ms: u64 },
}
```

#### 4. Action Executor
```rust
pub struct ActionExecutor {
    template_engine: HandlebarsEngine,
    json_path_engine: JsonPathEngine,
    fault_injector: FaultInjector,
    metrics: ActionMetrics,
    config: ActionExecutorConfig,
}

impl ActionExecutor {
    pub fn new(config: ActionExecutorConfig) -> Self;
    
    pub async fn execute_action(
        &self,
        action: &ActionSpec,
        context: &ActionContext,
    ) -> ActionResult<InterceptAction>;
    
    pub async fn apply_message_changes(
        &self,
        message: &TransportMessage,
        changes: &[MessageChange],
    ) -> ActionResult<TransportMessage>;
    
    pub async fn generate_mock_response(
        &self,
        mock: &MockResponse,
        context: &ActionContext,
    ) -> ActionResult<TransportMessage>;
    
    pub async fn inject_fault(
        &self,
        fault: &FaultType,
        probability: f32,
    ) -> ActionResult<Option<InterceptAction>>;
    
    pub fn get_metrics(&self) -> ActionMetrics;
}

#[derive(Debug, Clone)]
pub struct ActionContext {
    pub message: TransportMessage,
    pub direction: Direction,
    pub session_id: SessionId,
    pub transport_type: TransportType,
    pub timestamp: u64,
    pub metadata: HashMap<String, serde_json::Value>,
    pub execution_count: u32,
    pub rule_context: Option<RuleContext>,
}

#[derive(Debug, Clone)]
pub struct RuleContext {
    pub rule_id: String,
    pub rule_name: String,
    pub matched_conditions: Vec<String>,
    pub execution_history: Vec<ActionExecution>,
}

#[derive(Debug, Clone)]
pub struct ActionExecution {
    pub action_id: String,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub status: ExecutionStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Timeout,
    Cancelled,
}
```

#### 5. Template Engine Integration
```rust
impl ActionExecutor {
    fn setup_template_engine() -> HandlebarsEngine {
        let mut engine = Handlebars::new();
        
        // Register helper functions
        engine.register_helper("json_path", Box::new(json_path_helper));
        engine.register_helper("uuid", Box::new(uuid_helper));
        engine.register_helper("timestamp", Box::new(timestamp_helper));
        engine.register_helper("random", Box::new(random_helper));
        engine.register_helper("base64_encode", Box::new(base64_encode_helper));
        engine.register_helper("base64_decode", Box::new(base64_decode_helper));
        engine.register_helper("hash", Box::new(hash_helper));
        
        engine
    }
    
    async fn render_template(
        &self,
        template: &str,
        context: &ActionContext,
    ) -> ActionResult<String> {
        let template_context = self.build_template_context(context);
        
        self.template_engine
            .render_template(template, &template_context)
            .map_err(|e| ActionError::TemplateError(e.to_string()))
    }
    
    fn build_template_context(&self, context: &ActionContext) -> serde_json::Value {
        json!({
            "message": context.message,
            "direction": context.direction,
            "session_id": context.session_id.to_string(),
            "transport_type": context.transport_type,
            "timestamp": context.timestamp,
            "metadata": context.metadata,
            "execution_count": context.execution_count,
            "rule": context.rule_context,
            "env": std::env::vars().collect::<HashMap<String, String>>(),
        })
    }
}

// Template helper functions
fn json_path_helper(
    h: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    // Extract value using JSON path from template context
    // Implementation details...
    Ok(())
}

fn uuid_helper(/* ... */) -> handlebars::HelperResult {
    let uuid = uuid::Uuid::new_v4();
    out.write(&uuid.to_string())?;
    Ok(())
}

fn timestamp_helper(/* ... */) -> handlebars::HelperResult {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    out.write(&timestamp.to_string())?;
    Ok(())
}
```

## Implementation Details

### Phase 1: Core Action Types (Day 1)
1. Implement ActionSpec and ActionType enums
2. Create ActionExecutor with basic action execution
3. Add Continue, Block, and Pause actions
4. Implement action context and metrics collection
5. Write unit tests for basic actions

### Phase 2: Message Modification (Day 1-2)
1. Implement MessageChange types and JSON path editing
2. Add field transformation capabilities
3. Create message validation after modification
4. Add preserve_id and integrity checking
5. Test message modification with complex JSON structures

### Phase 3: Mock Responses and Templates (Day 2)
1. Implement MockResponse with template engine
2. Add response generators (static, template, file, generate)
3. Create template helper functions
4. Add response validation and error handling
5. Test template rendering with various contexts

### Phase 4: Advanced Features (Day 2)
1. Implement fault injection with probability controls
2. Add action composition (Chain, Conditional, Parallel)
3. Create retry mechanisms and timeout handling
4. Add comprehensive logging and metrics
5. Integration testing with interceptor engine

## Acceptance Criteria

### Functional Requirements
- [ ] All ActionType variants work correctly
- [ ] Message modification preserves JSON-RPC structure
- [ ] Mock responses generate valid MCP messages
- [ ] Template engine renders with context variables
- [ ] Fault injection works with specified probabilities
- [ ] Action composition executes in correct order
- [ ] Conditional actions evaluate runtime conditions
- [ ] Error handling provides detailed error messages

### Performance Requirements
- [ ] Action execution completes in < 1ms for simple actions
- [ ] Message modification adds < 100μs overhead
- [ ] Template rendering completes in < 5ms
- [ ] Memory usage scales linearly with action complexity
- [ ] Fault injection has negligible overhead when disabled

### Quality Requirements
- [ ] 100% test coverage for action execution logic
- [ ] Integration tests with all action types
- [ ] Performance benchmarks for action execution
- [ ] Template syntax validation and error handling
- [ ] Thread safety for concurrent action execution

## Test Plan

### Unit Tests
```rust
#[tokio::test]
async fn test_message_modification() {
    let executor = ActionExecutor::new(ActionExecutorConfig::default());
    
    let original_message = TransportMessage::new_request(
        "1".to_string(),
        "test".to_string(),
        json!({"param": "original_value"})
    );
    
    let changes = vec![
        MessageChange::SetField {
            path: "$.param".to_string(),
            value: json!("modified_value"),
            create_path: false,
        }
    ];
    
    let modified = executor.apply_message_changes(&original_message, &changes).await.unwrap();
    
    // Verify modification was applied correctly
    if let TransportMessage::Request { params, .. } = modified {
        assert_eq!(params["param"], "modified_value");
    }
}

#[tokio::test]
async fn test_mock_response_generation() {
    let executor = ActionExecutor::new(ActionExecutorConfig::default());
    
    let mock = MockResponse::Template {
        template: r#"{"result": "Hello {{message.method}}!"}"#.to_string(),
        context: HashMap::new(),
    };
    
    let context = ActionContext {
        message: TransportMessage::new_request("1".to_string(), "greet".to_string(), json!({})),
        // ... other fields
    };
    
    let response = executor.generate_mock_response(&mock, &context).await.unwrap();
    
    if let TransportMessage::Response { result, .. } = response {
        assert_eq!(result.unwrap()["result"], "Hello greet!");
    }
}

#[tokio::test]  
async fn test_fault_injection() {
    let executor = ActionExecutor::new(ActionExecutorConfig::default());
    
    let fault = FaultType::NetworkTimeout { timeout_ms: 1000 };
    
    // Test with 100% probability
    let result = executor.inject_fault(&fault, 1.0).await.unwrap();
    assert!(result.is_some());
    
    // Test with 0% probability
    let result = executor.inject_fault(&fault, 0.0).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_action_composition() {
    let executor = ActionExecutor::new(ActionExecutorConfig::default());
    
    let chain_action = ActionSpec {
        id: "test-chain".to_string(),
        action_type: ActionType::Chain {
            actions: vec![
                ActionSpec {
                    id: "log-1".to_string(),
                    action_type: ActionType::Log {
                        level: LogLevel::Info,
                        template: "First action".to_string(),
                        include_message: false,
                    },
                    // ... other fields
                },
                ActionSpec {
                    id: "modify-1".to_string(),
                    action_type: ActionType::Modify {
                        changes: vec![/* ... */],
                        preserve_id: true,
                    },
                    // ... other fields
                },
            ],
            stop_on_error: true,
        },
        // ... other fields
    };
    
    let context = create_test_context();
    let result = executor.execute_action(&chain_action, &context).await.unwrap();
    
    // Verify that chain executed successfully
    match result {
        InterceptAction::Modify(modified_message) => {
            // Verify message was modified by the chain
        }
        _ => panic!("Expected modified message from chain"),
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_action_executor_integration() {
    let executor = ActionExecutor::new(ActionExecutorConfig::default());
    
    // Test integration with interceptor engine
    let action = ActionSpec {
        id: "integration-test".to_string(),
        action_type: ActionType::Mock {
            response: MockResponse::Static {
                response: json!({
                    "id": "1",
                    "result": {"status": "success"}
                })
            },
            delay_ms: Some(10),
        },
        // ... other fields
    };
    
    let context = create_test_context();
    let result = executor.execute_action(&action, &context).await.unwrap();
    
    match result {
        InterceptAction::Mock { response } => {
            assert_eq!(response.id(), Some("1"));
        }
        _ => panic!("Expected mock response"),
    }
}
```

### Performance Tests
```rust
#[tokio::test]
async fn test_action_execution_performance() {
    let executor = ActionExecutor::new(ActionExecutorConfig::default());
    
    let simple_action = ActionSpec {
        id: "perf-test".to_string(),
        action_type: ActionType::Continue,
        // ... other fields
    };
    
    let context = create_test_context();
    let start = std::time::Instant::now();
    
    for _ in 0..10000 {
        let _ = executor.execute_action(&simple_action, &context).await.unwrap();
    }
    
    let elapsed = start.elapsed();
    let avg_per_action = elapsed / 10000;
    
    // Should complete in under 1ms per action
    assert!(avg_per_action.as_micros() < 1000);
}
```

## Dependencies

### Internal Dependencies
- InterceptAction from interceptor engine (Task 010)
- RuleContext from rule engine (Task 011)
- TransportMessage from transport layer
- Error handling framework

### External Dependencies
- `handlebars` crate for template rendering
- `jsonpath-lib` crate for JSON path operations
- `serde_json` for JSON manipulation
- `uuid` crate for UUID generation
- `base64` crate for encoding/decoding
- `sha2` crate for hashing

## Risks and Mitigations

### Risk: Template Injection
**Impact:** Malicious templates could execute arbitrary code  
**Mitigation:**
- Sandbox template execution environment
- Validate template syntax before execution
- Limit available helper functions
- Implement template complexity limits

### Risk: Message Corruption
**Impact:** Message modification could create invalid JSON-RPC  
**Mitigation:**
- Validate message structure after modification
- Preserve required JSON-RPC fields
- Add rollback capability for failed modifications
- Comprehensive schema validation

### Risk: Performance Impact
**Impact:** Complex actions could slow down message processing  
**Mitigation:**
- Implement action timeout handling
- Add performance monitoring and alerts
- Optimize template rendering and JSON operations
- Provide action complexity estimation

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Tests passing with > 95% coverage
- [ ] Performance benchmarks meet requirements
- [ ] Integration with interceptor engine working
- [ ] Template engine secure and performant
- [ ] Code review completed and approved
- [ ] Documentation complete with examples
- [ ] Error handling comprehensive

## Follow-up Tasks

- **Task 013:** CLI Intercept Management
- **Task 014:** Persistent Rule Storage
- Integration with session recording for action audit logs
- Advanced template functions and custom helpers
- Action library and sharing system