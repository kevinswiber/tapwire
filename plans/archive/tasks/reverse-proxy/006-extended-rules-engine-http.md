# Task 006: Extended RuleBasedInterceptor with HTTP Conditions

**Phase:** 5 (Reverse Proxy & Authentication)  
**Week:** 2 (Security & Integration)  
**Day:** 6  
**Priority:** Critical  
**Estimated Time:** 6-8 hours

## Overview

Extend the existing Phase 4 RuleBasedInterceptor to support HTTP-specific conditions and authentication context. This task leverages the proven interceptor infrastructure while adding capabilities for HTTP path matching, method filtering, authentication scope validation, and custom HTTP response actions.

## Success Criteria

- [x] Research validated extending existing RuleBasedInterceptor for minimal disruption
- [x] Research confirmed < 1ms policy evaluation performance target achievable
- [ ] HTTP-specific rule conditions (path, method, headers)
- [ ] Authentication context integration (scopes, claims, subjects)
- [ ] Custom HTTP response actions (block with status codes, redirect)
- [ ] Seamless integration with existing hot-reloading and CLI management
- [ ] Performance target: < 1ms additional evaluation overhead
- [ ] Backward compatibility with existing Phase 4 rules
- [ ] Integration with AuthGateway authentication context
- [ ] All tests passing (unit + integration + performance)

## Technical Specifications

### Enhanced InterceptContext for HTTP
```rust
// Extension of existing InterceptContext from Phase 4
#[derive(Debug, Clone)]
pub struct InterceptContext {
    // Existing Phase 4 fields (unchanged for compatibility)
    pub message: TransportMessage,
    pub direction: Direction,
    pub session_id: SessionId,
    pub transport_type: TransportType,
    pub timestamp: Instant,
    pub frame_id: u64,
    pub metadata: BTreeMap<String, String>,
    
    // NEW: Authentication context from AuthGateway
    pub auth_context: Option<AuthContext>,
    
    // NEW: HTTP-specific metadata
    pub http_metadata: Option<HttpMetadata>,
}

#[derive(Debug, Clone)]
pub struct HttpMetadata {
    pub method: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub user_agent: Option<String>,
    pub client_ip: Option<String>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
}

impl InterceptContext {
    // Helper methods for HTTP-specific rule matching
    pub fn http_method(&self) -> Option<&str> {
        self.http_metadata.as_ref().map(|h| h.method.as_str())
    }

    pub fn http_path(&self) -> Option<&str> {
        self.http_metadata.as_ref().map(|h| h.path.as_str())
    }

    pub fn auth_scopes(&self) -> Vec<String> {
        self.auth_context.as_ref()
            .map(|auth| auth.scopes.clone())
            .unwrap_or_default()
    }

    pub fn auth_subject(&self) -> Option<&str> {
        self.auth_context.as_ref().map(|auth| auth.subject.as_str())
    }
}
```

### HTTP-Specific Rule Conditions
```rust
// Extension of existing MatchCondition for HTTP support
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "condition_type")]
pub enum MatchCondition {
    // Existing Phase 4 conditions (unchanged)
    Method(MethodCondition),
    MessageType(MessageTypeCondition),
    SessionAge(SessionAgeCondition),
    Direction(DirectionCondition),
    And(AndCondition),
    Or(OrCondition),
    Not(NotCondition),
    
    // NEW: HTTP-specific conditions
    HttpMethod(HttpMethodCondition),
    HttpPath(HttpPathCondition),
    HttpHeader(HttpHeaderCondition),
    AuthScope(AuthScopeCondition),
    AuthSubject(AuthSubjectCondition),
    AuthClaim(AuthClaimCondition),
    ClientIp(ClientIpCondition),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpMethodCondition {
    pub match_type: StringMatchType, // Exact, Prefix, Suffix, Regex, Contains
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpPathCondition {
    pub match_type: StringMatchType,
    pub value: String,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHeaderCondition {
    pub header_name: String,
    pub match_type: StringMatchType,
    pub value: String,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthScopeCondition {
    pub match_type: ScopeMatchType, // Contains, ContainsAll, ContainsAny, Exact
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSubjectCondition {
    pub match_type: StringMatchType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthClaimCondition {
    pub claim_name: String,
    pub match_type: ClaimMatchType, // Exists, Equals, Contains, Regex
    pub expected_value: Option<serde_json::Value>,
}
```

### Enhanced Rule Evaluation Logic
```rust
// Extension of existing RuleEngine for HTTP conditions
impl RuleEngine {
    pub fn evaluate_condition(
        &self,
        condition: &MatchCondition,
        context: &InterceptContext,
    ) -> Result<bool, EvaluationError> {
        match condition {
            // Existing Phase 4 conditions (unchanged)
            MatchCondition::Method(c) => self.evaluate_method_condition(c, context),
            MatchCondition::MessageType(c) => self.evaluate_message_type_condition(c, context),
            MatchCondition::SessionAge(c) => self.evaluate_session_age_condition(c, context),
            MatchCondition::Direction(c) => self.evaluate_direction_condition(c, context),
            MatchCondition::And(c) => self.evaluate_and_condition(c, context),
            MatchCondition::Or(c) => self.evaluate_or_condition(c, context),
            MatchCondition::Not(c) => self.evaluate_not_condition(c, context),
            
            // NEW: HTTP-specific condition evaluation
            MatchCondition::HttpMethod(c) => self.evaluate_http_method_condition(c, context),
            MatchCondition::HttpPath(c) => self.evaluate_http_path_condition(c, context),
            MatchCondition::HttpHeader(c) => self.evaluate_http_header_condition(c, context),
            MatchCondition::AuthScope(c) => self.evaluate_auth_scope_condition(c, context),
            MatchCondition::AuthSubject(c) => self.evaluate_auth_subject_condition(c, context),
            MatchCondition::AuthClaim(c) => self.evaluate_auth_claim_condition(c, context),
            MatchCondition::ClientIp(c) => self.evaluate_client_ip_condition(c, context),
        }
    }

    fn evaluate_http_method_condition(
        &self,
        condition: &HttpMethodCondition,
        context: &InterceptContext,
    ) -> Result<bool, EvaluationError> {
        let method = context.http_method()
            .ok_or(EvaluationError::MissingHttpMetadata)?;
            
        Ok(self.string_matches(&condition.match_type, method, &condition.value))
    }

    fn evaluate_http_path_condition(
        &self,
        condition: &HttpPathCondition,
        context: &InterceptContext,
    ) -> Result<bool, EvaluationError> {
        let path = context.http_path()
            .ok_or(EvaluationError::MissingHttpMetadata)?;
            
        let normalized_path = if condition.case_sensitive {
            path.to_string()
        } else {
            path.to_lowercase()
        };
        
        let normalized_value = if condition.case_sensitive {
            condition.value.clone()
        } else {
            condition.value.to_lowercase()
        };
        
        Ok(self.string_matches(&condition.match_type, &normalized_path, &normalized_value))
    }

    fn evaluate_auth_scope_condition(
        &self,
        condition: &AuthScopeCondition,
        context: &InterceptContext,
    ) -> Result<bool, EvaluationError> {
        let auth_scopes = context.auth_scopes();
        
        match condition.match_type {
            ScopeMatchType::Contains => {
                Ok(condition.scopes.iter().any(|scope| auth_scopes.contains(scope)))
            }
            ScopeMatchType::ContainsAll => {
                Ok(condition.scopes.iter().all(|scope| auth_scopes.contains(scope)))
            }
            ScopeMatchType::ContainsAny => {
                Ok(condition.scopes.iter().any(|scope| auth_scopes.contains(scope)))
            }
            ScopeMatchType::Exact => {
                let mut expected_scopes = condition.scopes.clone();
                let mut actual_scopes = auth_scopes.clone();
                expected_scopes.sort();
                actual_scopes.sort();
                Ok(expected_scopes == actual_scopes)
            }
        }
    }

    fn evaluate_auth_claim_condition(
        &self,
        condition: &AuthClaimCondition,
        context: &InterceptContext,
    ) -> Result<bool, EvaluationError> {
        let auth_context = context.auth_context.as_ref()
            .ok_or(EvaluationError::MissingAuthContext)?;
            
        match condition.match_type {
            ClaimMatchType::Exists => {
                Ok(auth_context.custom_claims.contains_key(&condition.claim_name))
            }
            ClaimMatchType::Equals => {
                if let Some(expected) = &condition.expected_value {
                    let actual = auth_context.custom_claims.get(&condition.claim_name);
                    Ok(actual.map_or(false, |v| v == expected))
                } else {
                    Err(EvaluationError::MissingExpectedValue)
                }
            }
            ClaimMatchType::Contains => {
                // Implementation for string containment in claim values
                if let Some(expected) = &condition.expected_value {
                    if let Some(actual) = auth_context.custom_claims.get(&condition.claim_name) {
                        // Convert both to strings and check containment
                        let actual_str = actual.to_string();
                        let expected_str = expected.to_string();
                        Ok(actual_str.contains(&expected_str))
                    } else {
                        Ok(false)
                    }
                } else {
                    Err(EvaluationError::MissingExpectedValue)
                }
            }
            ClaimMatchType::Regex => {
                // Implementation with regex matching for claim values
                if let Some(expected) = &condition.expected_value {
                    if let Some(actual) = auth_context.custom_claims.get(&condition.claim_name) {
                        let pattern = expected.as_str()
                            .ok_or(EvaluationError::InvalidRegexPattern)?;
                        let regex = regex::Regex::new(pattern)
                            .map_err(|_| EvaluationError::InvalidRegexPattern)?;
                        let actual_str = actual.to_string();
                        Ok(regex.is_match(&actual_str))
                    } else {
                        Ok(false)
                    }
                } else {
                    Err(EvaluationError::MissingExpectedValue)
                }
            }
        }
    }
}
```

### HTTP-Specific Actions
```rust
// Extension of existing InterceptAction for HTTP responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action_type")]
pub enum InterceptAction {
    // Existing Phase 4 actions (unchanged)
    Allow,
    Block(BlockAction),
    Modify(ModifyAction),
    Log(LogAction),
    Delay(DelayAction),
    
    // NEW: HTTP-specific actions
    HttpBlock(HttpBlockAction),
    HttpRedirect(HttpRedirectAction),
    HttpSetHeader(HttpSetHeaderAction),
    HttpRemoveHeader(HttpRemoveHeaderAction),
    HttpSetStatus(HttpSetStatusAction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpBlockAction {
    pub status_code: u16,
    pub reason: String,
    pub custom_headers: Option<HashMap<String, String>>,
    pub response_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRedirectAction {
    pub status_code: u16, // 301, 302, 307, 308
    pub location: String,
    pub preserve_query: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSetHeaderAction {
    pub header_name: String,
    pub header_value: String,
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRemoveHeaderAction {
    pub header_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSetStatusAction {
    pub status_code: u16,
    pub reason_phrase: Option<String>,
}
```

### Action Execution for HTTP
```rust
// Extension of existing ActionExecutor for HTTP actions
impl ActionExecutor {
    pub async fn execute_action(
        &self,
        action: &InterceptAction,
        context: &mut InterceptContext,
    ) -> Result<ActionResult, ExecutionError> {
        match action {
            // Existing Phase 4 actions (unchanged)
            InterceptAction::Allow => Ok(ActionResult::Continue),
            InterceptAction::Block(action) => self.execute_block_action(action, context).await,
            InterceptAction::Modify(action) => self.execute_modify_action(action, context).await,
            InterceptAction::Log(action) => self.execute_log_action(action, context).await,
            InterceptAction::Delay(action) => self.execute_delay_action(action, context).await,
            
            // NEW: HTTP-specific action execution
            InterceptAction::HttpBlock(action) => self.execute_http_block_action(action, context).await,
            InterceptAction::HttpRedirect(action) => self.execute_http_redirect_action(action, context).await,
            InterceptAction::HttpSetHeader(action) => self.execute_http_set_header_action(action, context).await,
            InterceptAction::HttpRemoveHeader(action) => self.execute_http_remove_header_action(action, context).await,
            InterceptAction::HttpSetStatus(action) => self.execute_http_set_status_action(action, context).await,
        }
    }

    async fn execute_http_block_action(
        &self,
        action: &HttpBlockAction,
        context: &mut InterceptContext,
    ) -> Result<ActionResult, ExecutionError> {
        // Create HTTP error response
        let mut response_builder = http::Response::builder()
            .status(action.status_code);

        // Add custom headers if specified
        if let Some(headers) = &action.custom_headers {
            for (name, value) in headers {
                response_builder = response_builder.header(name, value);
            }
        }

        // Set content type for response body
        if action.response_body.is_some() {
            response_builder = response_builder.header("Content-Type", "text/plain");
        }

        let response_body = action.response_body
            .clone()
            .unwrap_or_else(|| action.reason.clone());

        let http_response = response_builder
            .body(response_body)
            .map_err(|e| ExecutionError::HttpResponseCreation(e.to_string()))?;

        // Store response in context for HTTP middleware to use
        context.metadata.insert(
            "http_response".to_string(),
            serde_json::to_string(&HttpInterceptResponse {
                status_code: action.status_code,
                headers: action.custom_headers.clone().unwrap_or_default(),
                body: response_body,
            }).unwrap()
        );

        Ok(ActionResult::Block)
    }

    async fn execute_http_redirect_action(
        &self,
        action: &HttpRedirectAction,
        context: &mut InterceptContext,
    ) -> Result<ActionResult, ExecutionError> {
        let mut location = action.location.clone();
        
        // Preserve query parameters if requested
        if action.preserve_query {
            if let Some(http_meta) = &context.http_metadata {
                if !http_meta.query_params.is_empty() {
                    let query_string = http_meta.query_params
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&");
                    location = format!("{}?{}", location, query_string);
                }
            }
        }

        // Create redirect response
        let http_response = http::Response::builder()
            .status(action.status_code)
            .header("Location", location)
            .body("")
            .map_err(|e| ExecutionError::HttpResponseCreation(e.to_string()))?;

        context.metadata.insert(
            "http_response".to_string(),
            serde_json::to_string(&HttpInterceptResponse {
                status_code: action.status_code,
                headers: [("Location".to_string(), action.location.clone())].into(),
                body: String::new(),
            }).unwrap()
        );

        Ok(ActionResult::Block) // Redirect terminates request processing
    }
}
```

### Integration with Existing Hot-Reloading
```rust
// Ensure HTTP rules work with existing hot-reload system
impl RulesManager {
    // Existing hot-reload functionality works unchanged
    // HTTP rules are just additional condition/action types
    
    pub async fn validate_http_rule(&self, rule: &InterceptRule) -> Result<(), ValidationError> {
        // Validate HTTP-specific conditions and actions
        for condition in &rule.match_conditions.conditions {
            self.validate_http_condition(condition)?;
        }
        
        for action in &rule.actions {
            self.validate_http_action(action)?;
        }
        
        Ok(())
    }

    fn validate_http_condition(&self, condition: &MatchCondition) -> Result<(), ValidationError> {
        match condition {
            MatchCondition::HttpPath(c) => {
                // Validate path patterns
                if c.match_type == StringMatchType::Regex {
                    regex::Regex::new(&c.value)
                        .map_err(|_| ValidationError::InvalidRegex(c.value.clone()))?;
                }
                Ok(())
            }
            MatchCondition::AuthScope(c) => {
                // Validate scope names
                for scope in &c.scopes {
                    if scope.is_empty() {
                        return Err(ValidationError::EmptyScope);
                    }
                }
                Ok(())
            }
            _ => Ok(()), // Delegate to existing validation
        }
    }
}
```

## Implementation Steps

### Step 1: Extend InterceptContext
- Add HttpMetadata and AuthContext fields
- Implement helper methods for HTTP rule matching
- Ensure backward compatibility with existing Phase 4 usage
- Add serialization support for new fields

### Step 2: Add HTTP Conditions
- Implement HTTP-specific match conditions
- Add authentication context conditions
- Extend evaluation logic in RuleEngine
- Add validation for new condition types

### Step 3: Implement HTTP Actions
- Add HTTP response actions (block, redirect, headers)
- Extend ActionExecutor for HTTP action execution
- Create HTTP response structures for middleware consumption
- Test action execution and response generation

### Step 4: Integration Testing
- Test HTTP rules with existing interceptor chain
- Validate hot-reloading works with HTTP rules
- Test CLI management of HTTP rules
- Verify performance meets targets

### Step 5: Documentation and Examples
- Create example HTTP rule configurations
- Document new condition and action types
- Update CLI help for HTTP-specific commands
- Add performance benchmarks

## Dependencies

### Blocked By
- Task 004: AuthGateway Core Implementation (authentication context)
- Task 005: Connection Pool and Circuit Breaker (HTTP metadata)

### Blocks
- Task 007: Rate Limiting and Audit Logging Integration
- Task 008: End-to-End Integration Testing

### Integrates With
- Existing Phase 4 RuleBasedInterceptor (foundation)
- Existing Phase 4 hot-reloading and CLI management
- AuthGateway authentication context

## Testing Requirements

### Unit Tests
- [ ] HTTP condition evaluation accuracy
- [ ] Authentication context condition matching
- [ ] HTTP action execution correctness
- [ ] Backward compatibility with Phase 4 rules
- [ ] Rule validation for HTTP conditions/actions

### Integration Tests
- [ ] End-to-end HTTP request interception
- [ ] Authentication-based rule enforcement
- [ ] HTTP response action delivery
- [ ] Hot-reloading with HTTP rules
- [ ] CLI management of HTTP rules

### Performance Tests
- [ ] HTTP rule evaluation overhead (target: < 1ms additional)
- [ ] Authentication context processing time
- [ ] HTTP action execution performance
- [ ] Concurrent HTTP rule evaluation
- [ ] Memory usage impact of HTTP extensions

### Security Tests
- [ ] Authentication bypass prevention
- [ ] HTTP header injection prevention
- [ ] Path traversal attack blocking
- [ ] Scope escalation prevention
- [ ] HTTP response tampering protection

## Configuration Schema

### Example HTTP Rules
```json
{
  "rules": [
    {
      "id": "admin-access-control",
      "priority": 100,
      "enabled": true,
      "match_conditions": {
        "operator": "and",
        "conditions": [
          {
            "condition_type": "HttpPath",
            "match_type": "prefix",
            "value": "/admin/",
            "case_sensitive": false
          },
          {
            "condition_type": "AuthScope",
            "match_type": "contains",
            "scopes": ["admin"]
          }
        ]
      },
      "actions": [
        {
          "action_type": "Allow"
        }
      ]
    },
    {
      "id": "block-unauthorized-admin",
      "priority": 90,
      "enabled": true,
      "match_conditions": {
        "operator": "and",
        "conditions": [
          {
            "condition_type": "HttpPath",
            "match_type": "prefix",
            "value": "/admin/",
            "case_sensitive": false
          },
          {
            "condition_type": "Not",
            "condition": {
              "condition_type": "AuthScope",
              "match_type": "contains",
              "scopes": ["admin"]
            }
          }
        ]
      },
      "actions": [
        {
          "action_type": "HttpBlock",
          "status_code": 403,
          "reason": "Admin access required",
          "response_body": "Access denied: Admin privileges required"
        }
      ]
    }
  ]
}
```

## Performance Requirements

- **HTTP rule evaluation:** < 1ms additional overhead
- **Authentication context processing:** < 100µs
- **HTTP action execution:** < 500µs
- **Memory per HTTP rule:** < 1KB
- **Concurrent rule evaluation:** 1000+ requests/second

## Risk Assessment

**Low Risk**: Building on proven Phase 4 infrastructure, well-defined extension points.

**Mitigation Strategies**:
- Extensive testing with existing Phase 4 functionality
- Gradual rollout of HTTP-specific features
- Performance monitoring during development
- Comprehensive backward compatibility testing

## Completion Checklist

- [ ] InterceptContext extended with HTTP and auth metadata
- [ ] HTTP-specific conditions implemented and tested
- [ ] Authentication context conditions working correctly
- [ ] HTTP response actions functional
- [ ] Integration with existing hot-reloading working
- [ ] CLI management supporting HTTP rules
- [ ] Performance targets met (< 1ms additional overhead)
- [ ] Backward compatibility with Phase 4 rules verified
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Performance benchmarks meeting targets
- [ ] Security tests validating enforcement
- [ ] Example configurations documented
- [ ] Code review completed

## Files Modified/Created

### New Files
- `src/interceptor/http_conditions.rs`: HTTP-specific condition implementations
- `src/interceptor/http_actions.rs`: HTTP-specific action implementations
- `src/interceptor/http_metadata.rs`: HTTP metadata structures
- `tests/unit/http_rules_test.rs`: Unit tests for HTTP extensions
- `tests/integration/http_interception_test.rs`: Integration tests

### Modified Files
- `src/interceptor/mod.rs`: Export HTTP extensions
- `src/interceptor/engine.rs`: Add HTTP condition evaluation
- `src/interceptor/actions.rs`: Add HTTP action execution
- `src/interceptor/types.rs`: Extend InterceptContext
- `src/proxy/reverse.rs`: Add HTTP metadata extraction
- Examples and documentation files

## Next Task
Upon completion, proceed to **Task 007: Rate Limiting and Audit Logging Integration** which adds comprehensive rate limiting across all authentication and interception layers with unified audit logging.