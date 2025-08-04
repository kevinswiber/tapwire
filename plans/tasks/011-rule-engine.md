# Task 011: Rule Engine Implementation

**Phase:** 4 - Interception & Rule Engine  
**Priority:** High  
**Estimated Effort:** 3 days  
**Assignee:** Development Team  
**Status:** Not Started

---

## Overview

Implement a powerful rule engine that enables developers to define complex conditions for intercepting MCP messages. The rule engine uses a JSON-based domain-specific language for matching message patterns and provides high-performance rule evaluation with early exit optimization.

## Objectives

- Create flexible JSON-based rule matching language
- Implement high-performance rule evaluation engine
- Support complex logical conditions (AND, OR, NOT)
- Enable context-aware matching (session, transport, timing)
- Provide rule validation and testing utilities
- Support rule priorities and chaining

## Technical Requirements

### Core Components

#### 1. Rule Definition Schema
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub priority: u32,
    pub conditions: RuleCondition,
    pub actions: Vec<ActionSpec>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    // Logical operators
    And(Vec<RuleCondition>),
    Or(Vec<RuleCondition>),
    Not(Box<RuleCondition>),
    
    // Message matching
    Method(StringMatcher),
    MessageType(MessageTypeMatcher),
    Params(JsonPathMatcher),
    Headers(HeaderMatcher),
    
    // Context matching
    Session(SessionMatcher),
    Transport(TransportMatcher),
    Direction(DirectionMatcher),
    Timing(TimingMatcher),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringMatcher {
    Exact(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    Regex(String),
    OneOf(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageTypeMatcher {
    Request,
    Response,
    Notification,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPathMatcher {
    pub path: String,
    pub matcher: JsonValueMatcher,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JsonValueMatcher {
    Exists,
    NotExists,
    Equals(serde_json::Value),
    Contains(String),
    GreaterThan(f64),
    LessThan(f64),
    Regex(String),
    ArrayLength { min: Option<usize>, max: Option<usize> },
}
```

#### 2. Rule Engine
```rust
pub struct RuleEngine {
    rules: RwLock<Vec<Arc<Rule>>>,
    regex_cache: RwLock<HashMap<String, regex::Regex>>,
    json_path_cache: RwLock<HashMap<String, jsonpath_lib::Selector>>,
    metrics: RuleEngineMetrics,
    config: RuleEngineConfig,
}

impl RuleEngine {
    pub fn new(config: RuleEngineConfig) -> Self;
    
    pub async fn add_rule(&self, rule: Rule) -> RuleResult<()>;
    pub async fn remove_rule(&self, rule_id: &str) -> RuleResult<()>;
    pub async fn enable_rule(&self, rule_id: &str) -> RuleResult<()>;
    pub async fn disable_rule(&self, rule_id: &str) -> RuleResult<()>;
    pub async fn get_rule(&self, rule_id: &str) -> RuleResult<Option<Arc<Rule>>>;
    pub async fn list_rules(&self) -> RuleResult<Vec<Arc<Rule>>>;
    
    pub async fn evaluate(&self, ctx: &InterceptContext) -> RuleResult<Vec<RuleMatch>>;
    pub async fn test_rule(&self, rule: &Rule, ctx: &InterceptContext) -> RuleResult<bool>;
    
    pub fn get_metrics(&self) -> RuleEngineMetrics;
    pub async fn reload_rules(&self, rules: Vec<Rule>) -> RuleResult<()>;
}

#[derive(Debug, Clone)]
pub struct RuleMatch {
    pub rule_id: String,
    pub rule_name: String,
    pub priority: u32,
    pub actions: Vec<ActionSpec>,
    pub evaluation_time_us: u64,
}

#[derive(Debug, Clone)]
pub struct RuleEngineConfig {
    pub max_rules: usize,
    pub max_evaluation_time_ms: u64,
    pub enable_caching: bool,
    pub cache_size: usize,
    pub enable_metrics: bool,
}

impl Default for RuleEngineConfig {
    fn default() -> Self {
        Self {
            max_rules: 1000,
            max_evaluation_time_ms: 100,
            enable_caching: true,
            cache_size: 1000,
            enable_metrics: true,
        }
    }
}
```

#### 3. Rule Matching Implementation
```rust
impl RuleEngine {
    async fn evaluate_condition(&self, condition: &RuleCondition, ctx: &InterceptContext) -> RuleResult<bool> {
        match condition {
            RuleCondition::And(conditions) => {
                for cond in conditions {
                    if !self.evaluate_condition(cond, ctx).await? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            
            RuleCondition::Or(conditions) => {
                for cond in conditions {
                    if self.evaluate_condition(cond, ctx).await? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            
            RuleCondition::Not(condition) => {
                Ok(!self.evaluate_condition(condition, ctx).await?)
            }
            
            RuleCondition::Method(matcher) => {
                match ctx.message.method() {
                    Some(method) => Ok(self.match_string(matcher, method)),
                    None => Ok(false),
                }
            }
            
            RuleCondition::MessageType(matcher) => {
                Ok(self.match_message_type(matcher, &ctx.message))
            }
            
            RuleCondition::Params(matcher) => {
                self.match_json_path(matcher, &ctx.message).await
            }
            
            RuleCondition::Session(matcher) => {
                Ok(self.match_session(matcher, ctx))
            }
            
            RuleCondition::Transport(matcher) => {
                Ok(self.match_transport(matcher, ctx))
            }
            
            RuleCondition::Direction(matcher) => {
                Ok(matcher.matches(ctx.direction))
            }
            
            RuleCondition::Timing(matcher) => {
                Ok(self.match_timing(matcher, ctx))
            }
        }
    }
    
    fn match_string(&self, matcher: &StringMatcher, value: &str) -> bool {
        match matcher {
            StringMatcher::Exact(expected) => value == expected,
            StringMatcher::Contains(substring) => value.contains(substring),
            StringMatcher::StartsWith(prefix) => value.starts_with(prefix),
            StringMatcher::EndsWith(suffix) => value.ends_with(suffix),
            StringMatcher::Regex(pattern) => {
                self.get_cached_regex(pattern)
                    .map(|re| re.is_match(value))
                    .unwrap_or(false)
            }
            StringMatcher::OneOf(options) => options.contains(&value.to_string()),
        }
    }
    
    async fn match_json_path(&self, matcher: &JsonPathMatcher, message: &TransportMessage) -> RuleResult<bool> {
        // Extract JSON from message based on type
        let json_value = match message {
            TransportMessage::Request { params, .. } => params,
            TransportMessage::Response { result, error, .. } => {
                result.as_ref().or(error.as_ref()).unwrap_or(&serde_json::Value::Null)
            }
            TransportMessage::Notification { params, .. } => params,
        };
        
        let selector = self.get_cached_json_path(&matcher.path)?;
        let selected_values = selector.find(json_value);
        
        match &matcher.matcher {
            JsonValueMatcher::Exists => Ok(!selected_values.is_empty()),
            JsonValueMatcher::NotExists => Ok(selected_values.is_empty()),
            JsonValueMatcher::Equals(expected) => {
                Ok(selected_values.iter().any(|v| *v == expected))
            }
            JsonValueMatcher::Contains(substring) => {
                Ok(selected_values.iter().any(|v| {
                    v.as_str().map(|s| s.contains(substring)).unwrap_or(false)
                }))
            }
            JsonValueMatcher::GreaterThan(threshold) => {
                Ok(selected_values.iter().any(|v| {
                    v.as_f64().map(|n| n > *threshold).unwrap_or(false)
                }))
            }
            JsonValueMatcher::LessThan(threshold) => {
                Ok(selected_values.iter().any(|v| {
                    v.as_f64().map(|n| n < *threshold).unwrap_or(false)
                }))
            }
            JsonValueMatcher::Regex(pattern) => {
                let regex = self.get_cached_regex(pattern).ok_or_else(|| {
                    RuleError::InvalidPattern(format!("Invalid regex pattern: {}", pattern))
                })?;
                Ok(selected_values.iter().any(|v| {
                    v.as_str().map(|s| regex.is_match(s)).unwrap_or(false)
                }))
            }
            JsonValueMatcher::ArrayLength { min, max } => {
                Ok(selected_values.iter().any(|v| {
                    v.as_array().map(|arr| {
                        let len = arr.len();
                        min.map(|m| len >= m).unwrap_or(true) &&
                        max.map(|m| len <= m).unwrap_or(true)
                    }).unwrap_or(false)
                }))
            }
        }
    }
}
```

#### 4. Context Matchers
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMatcher {
    pub id: Option<StringMatcher>,
    pub tags: Option<TagMatcher>,
    pub transport_type: Option<TransportType>,
    pub duration_ms: Option<RangeMatcher<u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMatcher {
    pub transport_type: TransportType,
    pub headers: Option<HashMap<String, StringMatcher>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMatcher {
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
    pub hour_of_day: Option<RangeMatcher<u8>>,
    pub day_of_week: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagMatcher {
    Any(Vec<String>),
    All(Vec<String>),
    None(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeMatcher<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}
```

## Implementation Details

### Phase 1: Core Rule Types and Parsing (Day 1)
1. Define Rule and RuleCondition enums with full JSON schema
2. Implement serde serialization/deserialization
3. Create rule validation with helpful error messages
4. Add rule priority and metadata support
5. Write comprehensive unit tests for rule parsing

### Phase 2: Basic Rule Evaluation (Day 1-2)
1. Implement RuleEngine with basic condition evaluation
2. Add string matching with regex support and caching
3. Implement logical operators (AND, OR, NOT)
4. Create message type and method matching
5. Add performance monitoring and metrics

### Phase 3: Advanced Matching Features (Day 2-3)
1. Implement JSON path matching with jsonpath library
2. Add session and transport context matching
3. Create timing-based conditions
4. Implement rule caching and optimization
5. Add rule testing and validation utilities

### Phase 4: Integration and Optimization (Day 3)
1. Integrate with InterceptorChain
2. Add rule hot-reloading capabilities
3. Performance optimization with early exit
4. Memory usage optimization
5. Comprehensive testing and benchmarking

## Acceptance Criteria

### Functional Requirements
- [ ] Rule engine can parse complex JSON rule definitions
- [ ] All logical operators (AND, OR, NOT) work correctly
- [ ] String matching supports exact, contains, regex, etc.
- [ ] JSON path matching works with MCP message structures
- [ ] Session and transport context matching is accurate
- [ ] Rule priorities determine evaluation order
- [ ] Rule validation provides helpful error messages

### Performance Requirements
- [ ] Rule evaluation completes in < 100μs per rule
- [ ] Regex and JSON path patterns are cached efficiently
- [ ] Memory usage stays under 1MB per 100 rules
- [ ] Rule loading and validation completes in < 10ms
- [ ] Concurrent rule evaluation scales to 1000+ ops/sec

### Quality Requirements
- [ ] 100% test coverage for rule evaluation logic
- [ ] Integration tests with all message and context types
- [ ] Performance benchmarks for rule evaluation
- [ ] Error handling with detailed error messages
- [ ] Documentation for rule language syntax

## Test Plan

### Unit Tests
```rust
#[tokio::test]
async fn test_rule_parsing() {
    let rule_json = r#"{
        "id": "test-rule",
        "name": "Test Rule",
        "enabled": true,
        "priority": 100,
        "conditions": {
            "And": [
                { "Method": { "Exact": "initialize" } },
                { "Transport": { "transport_type": "Stdio" } }
            ]
        },
        "actions": []
    }"#;
    
    let rule: Rule = serde_json::from_str(rule_json).unwrap();
    assert_eq!(rule.id, "test-rule");
    assert_eq!(rule.priority, 100);
}

#[tokio::test]
async fn test_string_matching() {
    let engine = RuleEngine::new(RuleEngineConfig::default());
    
    // Test exact match
    assert!(engine.match_string(&StringMatcher::Exact("test".to_string()), "test"));
    assert!(!engine.match_string(&StringMatcher::Exact("test".to_string()), "TEST"));
    
    // Test contains
    assert!(engine.match_string(&StringMatcher::Contains("est".to_string()), "test"));
    
    // Test regex
    assert!(engine.match_string(&StringMatcher::Regex(r"^test\d+$".to_string()), "test123"));
}

#[tokio::test]
async fn test_logical_operators() {
    let engine = RuleEngine::new(RuleEngineConfig::default());
    let ctx = create_test_context();
    
    // Test AND condition
    let and_condition = RuleCondition::And(vec![
        RuleCondition::Method(StringMatcher::Exact("test".to_string())),
        RuleCondition::Transport(TransportMatcher { transport_type: TransportType::Stdio, headers: None }),
    ]);
    
    let result = engine.evaluate_condition(&and_condition, &ctx).await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn test_json_path_matching() {
    let engine = RuleEngine::new(RuleEngineConfig::default());
    
    let message = TransportMessage::new_request(
        "1".to_string(),
        "test".to_string(),
        json!({"user": {"name": "Alice", "age": 30}})
    );
    
    let ctx = InterceptContext {
        message,
        direction: Direction::ClientToServer,
        session_id: SessionId::new(),
        transport_type: TransportType::Stdio,
        timestamp: 0,
        metadata: HashMap::new(),
    };
    
    let matcher = JsonPathMatcher {
        path: "$.user.name".to_string(),
        matcher: JsonValueMatcher::Equals(json!("Alice")),
    };
    
    let result = engine.match_json_path(&matcher, &ctx.message).await.unwrap();
    assert!(result);
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_rule_engine_integration() {
    let mut engine = RuleEngine::new(RuleEngineConfig::default());
    
    let rule = Rule {
        id: "integration-test".to_string(),
        name: "Integration Test Rule".to_string(),
        description: None,
        enabled: true,
        priority: 100,
        conditions: RuleCondition::Method(StringMatcher::Exact("initialize".to_string())),
        actions: vec![],
        metadata: HashMap::new(),
    };
    
    engine.add_rule(rule).await.unwrap();
    
    let ctx = create_initialize_context();
    let matches = engine.evaluate(&ctx).await.unwrap();
    
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].rule_id, "integration-test");
}
```

### Performance Tests
```rust
#[tokio::test]
async fn test_rule_evaluation_performance() {
    let engine = RuleEngine::new(RuleEngineConfig::default());
    
    // Add 100 rules
    for i in 0..100 {
        let rule = create_test_rule(&format!("rule-{}", i));
        engine.add_rule(rule).await.unwrap();
    }
    
    let ctx = create_test_context();
    let start = std::time::Instant::now();
    
    for _ in 0..1000 {
        let _ = engine.evaluate(&ctx).await.unwrap();
    }
    
    let elapsed = start.elapsed();
    let avg_per_eval = elapsed / 1000;
    
    // Should complete in under 100μs per evaluation
    assert!(avg_per_eval.as_micros() < 100);
}
```

## Dependencies

### Internal Dependencies
- InterceptContext from interceptor engine (Task 010)
- TransportMessage and SessionId from transport layer
- Error handling framework

### External Dependencies
- `regex` crate for regular expression matching
- `jsonpath-lib` crate for JSON path queries
- `serde_json` for JSON manipulation

## Risks and Mitigations

### Risk: Regex Performance
**Impact:** Complex regex patterns could slow down rule evaluation  
**Mitigation:**
- Implement regex compilation caching
- Set timeout limits on regex evaluation
- Provide regex complexity validation
- Add performance monitoring and alerts

### Risk: JSON Path Complexity
**Impact:** Complex JSON path queries could be slow or consume memory  
**Mitigation:**
- Cache compiled JSON path selectors
- Limit JSON path depth and complexity
- Implement query timeout handling
- Memory usage monitoring

### Risk: Rule Language Complexity
**Impact:** Rule syntax could become too complex for users  
**Mitigation:**
- Start with simple conditions and expand incrementally
- Provide extensive examples and documentation
- Add rule validation with helpful error messages
- Create rule builder utilities in CLI

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Tests passing with > 95% coverage
- [ ] Performance benchmarks meet requirements
- [ ] Rule language documented with examples
- [ ] Integration with interceptor engine working
- [ ] Code review completed and approved
- [ ] No memory leaks detected
- [ ] Error handling comprehensive

## Follow-up Tasks

- **Task 012:** Intercept Actions Implementation  
- **Task 013:** CLI Intercept Management
- **Task 014:** Persistent Rule Storage
- Rule performance optimization based on usage patterns
- Advanced rule features (conditions on message history, rate limiting)