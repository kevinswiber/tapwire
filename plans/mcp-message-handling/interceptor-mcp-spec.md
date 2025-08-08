# MCP-Aware Interceptor Specification

## Overview

This specification details how to upgrade Shadowcat's interceptor from transport-level frame interception to intelligent MCP JSON-RPC message interception.

## Current vs Target Architecture

### Current: Transport Frame Interception
```
[Transport] → [Raw Bytes] → [Interceptor] → [Raw Bytes] → [Destination]
```

### Target: MCP Message Interception
```
[Transport] → [Parse] → [MCP Message] → [Interceptor] → [MCP Message] → [Serialize] → [Destination]
                              ↓                                ↑
                        [Correlation]                    [Modification]
```

## Core Components

### 1. Message Stream Processor

**Purpose**: Convert transport streams into MCP message streams

```rust
pub struct McpStreamProcessor {
    parser: McpParser,
    correlator: MessageCorrelator,
    interceptor_chain: InterceptorChain,
    metrics: McpMetrics,
}

impl McpStreamProcessor {
    pub async fn process_inbound(&mut self, bytes: &[u8]) -> Result<Vec<McpMessage>> {
        // 1. Parse bytes into MCP messages
        let messages = self.parser.parse(bytes)?;
        
        // 2. Correlate with pending requests
        for msg in &messages {
            if let McpMessage::Single(JsonRpcMessage::V2(JsonRpcV2Message::Response { id, .. })) = msg {
                self.correlator.match_response(id, msg).await;
            }
        }
        
        // 3. Apply interceptors
        let mut processed = Vec::new();
        for msg in messages {
            match self.interceptor_chain.process(msg).await {
                InterceptResult::Allow(msg) => processed.push(msg),
                InterceptResult::Block(reason) => {
                    self.metrics.record_blocked(reason);
                }
                InterceptResult::Modify(modified) => processed.push(modified),
            }
        }
        
        Ok(processed)
    }
}
```

### 2. MCP Interceptor Rules

**Purpose**: Define conditions and actions for MCP messages

```rust
pub struct McpInterceptorRule {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub conditions: Vec<McpCondition>,
    pub actions: Vec<McpAction>,
    pub metadata: RuleMetadata,
}

pub enum McpCondition {
    // Method matching
    MethodEquals(String),
    MethodStartsWith(String),
    MethodEndsWith(String),
    MethodMatches(Regex),
    MethodIn(HashSet<String>),
    
    // Parameter inspection
    ParamExists(String),
    ParamEquals(String, Value),
    ParamMatches(String, Regex),
    ParamGreaterThan(String, f64),
    ParamLessThan(String, f64),
    ParamIsNull(String),
    ParamIsArray(String),
    
    // Response inspection
    ResultExists(String),
    ResultEquals(String, Value),
    ErrorCodeEquals(i32),
    ErrorMessageContains(String),
    
    // Session context
    SessionIdEquals(String),
    SessionAgeGreaterThan(Duration),
    
    // Logical operators
    All(Vec<McpCondition>),
    Any(Vec<McpCondition>),
    Not(Box<McpCondition>),
}

pub enum McpAction {
    // Control flow
    Allow,
    Block(String),
    Delay(Duration),
    
    // Modification
    SetParam(String, Value),
    RemoveParam(String),
    SetResult(String, Value),
    InjectError { code: i32, message: String },
    
    // Transformation
    Transform(Box<dyn MessageTransformer>),
    
    // Side effects
    Log { level: LogLevel, message: String },
    Metric { name: String, value: f64 },
    Webhook { url: String, include_message: bool },
    RunScript { path: String, timeout: Duration },
    
    // Advanced
    Fork { copies: usize },
    Cache { key: String, ttl: Duration },
    RateLimit { key: String, limit: usize, window: Duration },
}
```

### 3. Stateful Interception

**Purpose**: Maintain state across related messages

```rust
pub struct StatefulInterceptor {
    sessions: HashMap<SessionId, SessionState>,
    conversations: HashMap<JsonRpcId, Conversation>,
}

pub struct SessionState {
    pub session_id: SessionId,
    pub mcp_session_id: Option<String>,
    pub started_at: Instant,
    pub message_count: usize,
    pub method_stats: HashMap<String, MethodStats>,
    pub custom_state: HashMap<String, Value>,
}

pub struct Conversation {
    pub request: JsonRpcV2Message,
    pub request_time: Instant,
    pub response: Option<JsonRpcV2Message>,
    pub response_time: Option<Instant>,
    pub intercepted: bool,
    pub modifications: Vec<Modification>,
}

impl StatefulInterceptor {
    pub async fn intercept_with_context(
        &mut self,
        message: McpMessage,
        session_id: SessionId,
    ) -> InterceptResult {
        let session = self.sessions.entry(session_id).or_default();
        
        match message {
            McpMessage::Single(JsonRpcMessage::V2(msg)) => {
                match msg {
                    JsonRpcV2Message::Request { id, method, .. } => {
                        // Track conversation start
                        self.conversations.insert(id.clone(), Conversation {
                            request: msg.clone(),
                            request_time: Instant::now(),
                            response: None,
                            response_time: None,
                            intercepted: false,
                            modifications: Vec::new(),
                        });
                        
                        // Update session stats
                        session.method_stats
                            .entry(method.clone())
                            .or_default()
                            .increment_request();
                    }
                    JsonRpcV2Message::Response { id, .. } => {
                        // Complete conversation
                        if let Some(conv) = self.conversations.get_mut(&id) {
                            conv.response = Some(msg.clone());
                            conv.response_time = Some(Instant::now());
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        
        // Apply stateful rules
        self.apply_stateful_rules(message, session).await
    }
}
```

### 4. Method-Specific Interceptors

**Purpose**: Specialized handling for common MCP methods

```rust
pub trait MethodInterceptor: Send + Sync {
    fn method(&self) -> &str;
    async fn intercept_request(&self, params: Option<&Value>) -> InterceptResult;
    async fn intercept_response(&self, result: Option<&Value>) -> InterceptResult;
}

// Example: Tool call interceptor
pub struct ToolCallInterceptor {
    allowed_tools: HashSet<String>,
    rate_limiter: RateLimiter,
}

impl MethodInterceptor for ToolCallInterceptor {
    fn method(&self) -> &str {
        "tools/call"
    }
    
    async fn intercept_request(&self, params: Option<&Value>) -> InterceptResult {
        if let Some(params) = params {
            if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
                // Check if tool is allowed
                if !self.allowed_tools.contains(tool_name) {
                    return InterceptResult::Block(format!("Tool '{}' not allowed", tool_name));
                }
                
                // Apply rate limiting
                if !self.rate_limiter.check(tool_name).await {
                    return InterceptResult::Block("Rate limit exceeded".into());
                }
            }
        }
        InterceptResult::Allow
    }
    
    async fn intercept_response(&self, result: Option<&Value>) -> InterceptResult {
        // Could modify or validate tool responses
        InterceptResult::Allow
    }
}
```

### 5. Interceptor Chain Manager

**Purpose**: Coordinate multiple interceptors with priorities

```rust
pub struct InterceptorChainManager {
    interceptors: Vec<Box<dyn McpInterceptor>>,
    rules: Vec<McpInterceptorRule>,
    method_interceptors: HashMap<String, Box<dyn MethodInterceptor>>,
    config: ChainConfig,
}

pub struct ChainConfig {
    pub fail_fast: bool,  // Stop on first block
    pub parallel_execution: bool,  // Run independent interceptors in parallel
    pub timeout: Duration,  // Max time for chain execution
    pub default_action: InterceptAction,  // Action if no rules match
}

impl InterceptorChainManager {
    pub async fn process(&self, message: McpMessage) -> InterceptResult {
        // 1. Extract method if present
        let method = extract_method(&message);
        
        // 2. Apply method-specific interceptor if exists
        if let Some(method) = method {
            if let Some(interceptor) = self.method_interceptors.get(method) {
                let result = apply_method_interceptor(interceptor, &message).await;
                if !result.is_allow() && self.config.fail_fast {
                    return result;
                }
            }
        }
        
        // 3. Apply rules in priority order
        let mut rules = self.rules.clone();
        rules.sort_by_key(|r| -r.priority);
        
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            
            if evaluate_conditions(&rule.conditions, &message).await {
                let result = apply_actions(&rule.actions, message.clone()).await;
                if !result.is_allow() && self.config.fail_fast {
                    return result;
                }
            }
        }
        
        // 4. Apply general interceptors
        if self.config.parallel_execution {
            self.process_parallel(message).await
        } else {
            self.process_sequential(message).await
        }
    }
}
```

## Integration Points

### 1. Forward Proxy Integration

```rust
// In src/proxy/forward.rs
impl ForwardProxy {
    async fn handle_client_message(&mut self, raw_message: Vec<u8>) -> Result<()> {
        // Parse to MCP
        let mcp_messages = self.mcp_processor.parse(&raw_message)?;
        
        // Apply interceptors
        let processed = self.interceptor_manager.process_batch(mcp_messages).await?;
        
        // Record if enabled
        if self.recording_enabled {
            self.recorder.record_messages(&processed).await?;
        }
        
        // Serialize and forward
        let serialized = serialize_mcp_messages(&processed)?;
        self.upstream_transport.send(serialized).await?;
        
        Ok(())
    }
}
```

### 2. Reverse Proxy Integration

```rust
// In src/proxy/reverse.rs
impl ReverseProxy {
    async fn handle_mcp_request(&mut self, request: McpMessage) -> McpMessage {
        // Apply server-side interceptors
        let intercepted = self.interceptor_manager.process(request).await?;
        
        // Forward to upstream
        let response = self.forward_to_upstream(intercepted).await?;
        
        // Apply response interceptors
        self.interceptor_manager.process_response(response).await
    }
}
```

## Configuration Schema

```yaml
interceptor:
  enabled: true
  mode: mcp  # or "transport" for legacy
  
  chain:
    fail_fast: true
    parallel: false
    timeout: 5s
    default_action: allow
  
  rules:
    - id: block-dangerous-tools
      name: "Block dangerous tool calls"
      enabled: true
      priority: 100
      conditions:
        - method_equals: "tools/call"
        - param_in:
            field: "name"
            values: ["execute_command", "delete_file"]
      actions:
        - block: "Dangerous tool blocked for safety"
        - log:
            level: warn
            message: "Attempted to call dangerous tool"
    
    - id: rate-limit-expensive
      name: "Rate limit expensive operations"
      enabled: true
      priority: 90
      conditions:
        - method_starts_with: "expensive/"
      actions:
        - rate_limit:
            key: "${session_id}:${method}"
            limit: 10
            window: 60s
    
    - id: cache-readonly
      name: "Cache read-only responses"
      enabled: true
      priority: 80
      conditions:
        - method_in: ["resources/list", "tools/list"]
      actions:
        - cache:
            key: "${method}:${params_hash}"
            ttl: 300s
  
  method_interceptors:
    tools/call:
      class: ToolCallInterceptor
      config:
        allowed_tools:
          - read_file
          - write_file
          - list_directory
        rate_limit:
          per_minute: 60
    
    resources/read:
      class: ResourceAccessInterceptor
      config:
        allowed_paths:
          - /workspace
          - /tmp
        deny_patterns:
          - "*.key"
          - "*.pem"
```

## Performance Considerations

### 1. Message Parsing
- Use zero-copy parsing where possible
- Cache parsed messages within request lifecycle
- Stream large messages instead of buffering

### 2. Rule Evaluation
- Compile regex patterns once at startup
- Use efficient data structures (HashMap for method lookup)
- Short-circuit evaluation for logical operators

### 3. State Management
- Implement TTL for conversation tracking
- Periodic cleanup of old sessions
- Bounded memory usage with eviction policies

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_method_condition() {
        let condition = McpCondition::MethodEquals("tools/call".into());
        let message = create_request("tools/call", json!({}));
        assert!(evaluate_condition(&condition, &message));
    }
    
    #[tokio::test]
    async fn test_rate_limit_action() {
        let action = McpAction::RateLimit {
            key: "test".into(),
            limit: 2,
            window: Duration::from_secs(1),
        };
        
        let msg1 = create_request("test", json!({}));
        let msg2 = create_request("test", json!({}));
        let msg3 = create_request("test", json!({}));
        
        assert!(apply_action(&action, msg1).await.is_allow());
        assert!(apply_action(&action, msg2).await.is_allow());
        assert!(apply_action(&action, msg3).await.is_block());
    }
}
```

### Integration Tests
- Test full message flow through interceptor chain
- Verify correlation between requests and responses
- Test stateful rules across multiple messages
- Benchmark performance with various rule sets

## Migration Path

1. **Phase 1**: Implement MCP parser alongside existing interceptor
2. **Phase 2**: Add MCP interceptor rules without removing transport rules
3. **Phase 3**: Gradually migrate rules from transport to MCP level
4. **Phase 4**: Deprecate transport-level interception for MCP traffic
5. **Phase 5**: Optimize and remove legacy code

## Future Enhancements

1. **Machine Learning Integration**
   - Anomaly detection for unusual message patterns
   - Automatic rule generation from traffic analysis

2. **Distributed Interception**
   - Share state across multiple proxy instances
   - Coordinated rate limiting

3. **Dynamic Rule Loading**
   - Hot reload rules without restart
   - A/B testing of rule sets

4. **Advanced Analytics**
   - Real-time dashboard of intercepted messages
   - Historical analysis of patterns