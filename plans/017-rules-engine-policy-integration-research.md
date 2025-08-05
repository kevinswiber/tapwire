# Rules Engine & Policy Integration Research Report

**Research Period:** August 4, 2025  
**Researcher:** Claude Code Session  
**Status:** Complete - Day 3 Research Deliverable  
**Purpose:** Analysis of existing Phase 4 interceptor patterns and policy engine integration for Shadowcat Phase 5

---

## Executive Summary

**Key Findings:**
- **Phase 4 interceptor infrastructure is highly suitable** for reverse proxy security policies with minimal modification
- **Extend existing RuleEngine** approach is optimal - leverages mature hot-reloading, CLI, and JSONPath matching
- **AuthContext integration** requires only InterceptContext extension and HTTP-specific rule conditions
- **External policy engines** (OPA/Cedar) offer benefits but may be overkill for initial implementation

**Critical Decisions:**
1. **Architecture Choice:** Extend existing RuleBasedInterceptor for security policies
2. **Integration Pattern:** Layer-based middleware with InterceptContext enhancement
3. **Policy Engine Strategy:** Hybrid approach - extend existing for HTTP, evaluate external engines for advanced use cases
4. **Performance Target:** < 1ms policy evaluation using existing rule engine optimizations

---

## Research Methodology

### Approach and Criteria
- **Codebase Analysis:** Deep examination of Phase 4 interceptor architecture
- **Performance Analysis:** Evaluation of existing rule engine performance characteristics
- **External Engine Evaluation:** Research on OPA and Cedar integration options
- **HTTP Integration Patterns:** Analysis of Axum middleware patterns for policy enforcement

### Sources Consulted
- Shadowcat Phase 4 interceptor source code (`src/interceptor/`)
- OPA performance benchmarks and Rust integration options
- Cedar policy language performance characteristics
- Axum middleware ecosystem for authorization patterns

---

## Detailed Analysis

### Phase 4 Interceptor Architecture Analysis

#### Existing Infrastructure Strengths

**InterceptorChain Design:**
```rust
// Highly suitable for auth policy integration
pub struct InterceptorChain {
    registry: InterceptorRegistry,       // Priority-based ordering
    metrics: Arc<RwLock<InterceptorMetrics>>, // Performance tracking
    enabled: bool,                       // Runtime enable/disable
}

// Process messages through registered interceptors
async fn intercept(&self, ctx: &InterceptContext) -> InterceptResult<InterceptAction>
```

**RuleBasedInterceptor Capabilities:**
- **Production-Ready:** 127 tests passing, comprehensive error handling
- **Hot-Reloading:** File watching with atomic rule replacement and rollback
- **Performance Optimized:** < 45µs average evaluation time (Phase 4 metrics)
- **CLI Integration:** Complete `shadowcat intercept` command suite
- **Flexible Actions:** Continue, Block, Modify, Pause, Delay, Mock

**RuleEngine Features:**
- **JSONPath Matching:** Basic implementation with potential for enhancement
- **Logical Operators:** AND, OR, NOT with nested conditions
- **Multiple Matchers:** String, Value, Range, Session, Direction, Transport
- **Priority-Based Processing:** Rules evaluated by priority order
- **Runtime Management:** Add, remove, enable/disable rules without restart

#### Integration Requirements for HTTP/Auth

**InterceptContext Enhancement Needed:**
```rust
#[derive(Debug, Clone)]
pub struct InterceptContext {
    pub message: TransportMessage,
    pub direction: Direction,
    pub session_id: SessionId,
    pub transport_type: TransportType,
    pub timestamp: Instant,
    pub frame_id: u64,
    pub metadata: BTreeMap<String, String>,
    
    // NEW: Authentication context for reverse proxy
    pub auth_context: Option<AuthContext>,
    
    // NEW: HTTP-specific metadata
    pub http_metadata: Option<HttpMetadata>,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub scopes: Vec<String>,
    pub permissions: Vec<String>,
    pub token_claims: TokenClaims,
    pub client_ip: Option<String>,
    pub authenticated: bool,
}

#[derive(Debug, Clone)]
pub struct HttpMetadata {
    pub method: String,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub remote_addr: Option<String>,
}
```

**HTTP-Specific Rule Conditions:**
```json
{
  "version": "1.0",
  "rules": [
    {
      "id": "require-admin-scope",
      "name": "Admin endpoint access control",
      "match_conditions": {
        "operator": "and",
        "conditions": [
          {
            "http_path": {
              "match_type": "prefix",
              "value": "/admin/"
            }
          },
          {
            "auth_context": {
              "scope": {
                "match_type": "contains",
                "value": "admin"
              }
            }
          }
        ]
      },
      "actions": [
        {
          "action_type": "block",
          "parameters": {
            "reason": "Admin scope required for admin endpoints",
            "http_status": 403
          }
        }
      ]
    }
  ]
}
```

### Performance Analysis

#### Current Rule Engine Performance

**Measured Performance (Phase 4):**
- **Average Evaluation Time:** 45µs per message (with complex rules)
- **Memory Usage:** < 10MB for 1000 rules
- **Hot-Reload Time:** < 1 second for rule file changes
- **Concurrent Processing:** Thread-safe with Arc/RwLock patterns

**Performance Characteristics:**
- **Rule Complexity:** O(n) for rule count, O(m) for conditions per rule
- **JSONPath Evaluation:** Basic dot notation, room for optimization
- **Memory Efficiency:** Rules stored as structured data, minimal overhead
- **Caching:** No rule result caching (opportunity for optimization)

#### Projected HTTP/Auth Performance

**Authentication Context Overhead:**
- **Token Validation:** Amortized via JWT caching (separate component)
- **Auth Context Creation:** ~1µs additional per request
- **HTTP Metadata Extraction:** ~0.5µs for header processing
- **Policy Evaluation:** ~50µs total (including auth context)

**Performance Optimizations Available:**
1. **Rule Result Caching:** Cache policy decisions based on user/path combinations
2. **Condition Short-Circuiting:** Early exit on first failed condition
3. **Rule Indexing:** Index rules by HTTP path/method for faster lookup
4. **Batch Evaluation:** Evaluate multiple related policies together

### External Policy Engine Evaluation

#### Open Policy Agent (OPA) Analysis

**Performance Characteristics:**
- **Evaluation Time:** 29.8-45µs per operation (comparable to existing engine)
- **Language:** Rego policies compile to WebAssembly for embedding
- **Integration:** `opa-rs` crate provides Rust integration via WASM
- **Memory Usage:** Higher overhead due to WASM runtime

**Pros:**
- Industry standard with extensive ecosystem
- Rich policy language with advanced features
- Battle-tested in production environments
- Strong tooling and debugging capabilities

**Cons:**
- Additional complexity and dependencies
- WASM runtime overhead
- Learning curve for Rego language
- May be overkill for MCP-specific policies

#### Cedar Policy Language Analysis

**Performance Characteristics:**
- **Evaluation Time:** < 1ms authorization latency (AWS Verified Permissions)
- **Language:** Cedar policies in human-readable format
- **Integration:** Native Rust implementation, no FFI overhead
- **Memory Usage:** Optimized for bounded latency (O(n²) worst case)

**Pros:**
- Designed for millisecond-scale authorization
- Native Rust implementation (no FFI/WASM overhead)
- Formal verification and safety guarantees
- AWS backing and active development

**Cons:**
- Newer ecosystem with less community adoption
- More complex than needed for basic HTTP policies
- Learning curve for Cedar policy syntax
- Potential vendor lock-in concerns

### HTTP Integration Patterns

#### Axum Middleware Integration

**Layer-Based Architecture:**
```rust
// Policy enforcement middleware with auth context
async fn policy_enforcement_middleware(
    State(interceptor_chain): State<Arc<InterceptorChain>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Extract HTTP metadata
    let http_metadata = HttpMetadata {
        method: request.method().to_string(),
        path: request.uri().path().to_string(),
        headers: extract_headers(request.headers()),
        remote_addr: extract_remote_addr(&request),
    };
    
    // 2. Get auth context from previous middleware
    let auth_context = request.extensions()
        .get::<AuthContext>()
        .cloned();
    
    // 3. Create intercept context
    let intercept_context = InterceptContext {
        message: http_to_transport_message(&request).await?,
        direction: Direction::ClientToServer,
        session_id: extract_session_id(&request)?,
        transport_type: TransportType::Http,
        timestamp: Instant::now(),
        frame_id: generate_frame_id(),
        metadata: BTreeMap::new(),
        auth_context,
        http_metadata: Some(http_metadata),
    };
    
    // 4. Process through interceptor chain
    match interceptor_chain.intercept(&intercept_context).await? {
        InterceptAction::Continue => Ok(next.run(request).await),
        InterceptAction::Block { reason } => {
            Err(StatusCode::FORBIDDEN) // Convert to HTTP response
        },
        InterceptAction::Modify(msg) => {
            // Update request with modified message
            let modified_request = transport_to_http_message(msg, request)?;
            Ok(next.run(modified_request).await)
        },
        // Handle other actions...
    }
}
```

**Router Configuration:**
```rust
fn create_mcp_router(interceptor_chain: Arc<InterceptorChain>) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp_request))
        .layer(middleware::from_fn_with_state(
            interceptor_chain.clone(),
            policy_enforcement_middleware
        ))
        .layer(middleware::from_fn_with_state(
            auth_gateway.clone(),
            jwt_auth_middleware
        ))
        .with_state(interceptor_chain)
}
```

---

## Recommendations

### Primary Architecture Choice: Extend Existing RuleEngine

**Rationale:**
1. **Proven Performance:** Existing engine already meets < 1ms target for policy evaluation
2. **Rich Feature Set:** Hot-reloading, CLI management, priority-based processing
3. **Minimal Learning Curve:** Team already familiar with rule syntax and patterns
4. **Integration Ready:** Seamless integration with existing interceptor infrastructure

### Enhancement Strategy

**Phase 1: Core HTTP Extensions**
1. **Extend InterceptContext:** Add `auth_context` and `http_metadata` fields
2. **HTTP Rule Conditions:** Add path, method, header, and auth context matchers
3. **HTTP Action Types:** Add HTTP status code responses and redirects
4. **Policy-Specific CLI:** Extend `shadowcat intercept` with auth policy commands

**Phase 2: Performance Optimizations**
1. **Rule Result Caching:** Cache policy decisions for common user/path combinations
2. **Condition Indexing:** Build indexes for faster rule lookup by HTTP attributes
3. **JSONPath Enhancement:** Integrate full JSONPath library for advanced matching
4. **Batch Evaluation:** Optimize for multiple rule evaluation

**Phase 3: Advanced Features (Optional)**
1. **External Engine Integration:** Add OPA/Cedar as alternative backends
2. **Policy Templates:** Pre-built policy templates for common patterns
3. **Policy Testing:** Framework for testing policies against sample requests

### HTTP-Specific Enhancements

**New Rule Condition Types:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpCondition {
    Path(StringMatcher),
    Method(StringMatcher),
    Header { name: String, matcher: StringMatcher },
    RemoteAddr(StringMatcher),
    AuthUser(StringMatcher),
    AuthScope(StringMatcher),
    AuthPermission(StringMatcher),
}
```

**New Action Types:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpActionType {
    HttpBlock { status: u16, reason: String },
    HttpRedirect { location: String, status: u16 },
    SetHeader { name: String, value: String },
    RemoveHeader { name: String },
}
```

---

## Risk Assessment

### Known Limitations and Risks

**Existing Engine Limitations:**
- **JSONPath Implementation:** Current implementation is basic, may need enhancement
- **Rule Complexity:** Complex rules could impact performance under high load
- **Memory Usage:** Rule storage grows linearly with policy count

**HTTP Integration Risks:**
- **Context Overhead:** Additional auth/HTTP metadata may impact memory usage
- **Serialization Cost:** Converting between HTTP and TransportMessage formats
- **Concurrent Access:** Rule evaluation may become bottleneck under high concurrency

**External Engine Integration Risks:**
- **Complexity:** Adding OPA/Cedar increases system complexity
- **Performance:** External engines may not meet < 1ms latency requirements
- **Dependencies:** Additional crates and runtime dependencies

### Mitigation Strategies

1. **Performance Monitoring:** Implement comprehensive metrics for policy evaluation
2. **Caching Strategy:** Implement policy result caching for frequent patterns
3. **Gradual Migration:** Start with extended RuleEngine, add external engines as needed
4. **Load Testing:** Comprehensive performance testing under realistic loads
5. **Fallback Options:** Design with ability to disable policy enforcement for emergencies

---

## Implementation Impact

### Changes Required

**Core InterceptContext Extension:**
```rust
// Add to existing InterceptContext struct
pub auth_context: Option<AuthContext>,
pub http_metadata: Option<HttpMetadata>,
```

**Rule Engine Enhancements:**
- New condition matchers for HTTP attributes
- New action types for HTTP responses
- Enhanced JSONPath evaluation for auth context fields

**CLI Extensions:**
```bash
# New policy-specific commands
shadowcat intercept policy list
shadowcat intercept policy add ./security-policy.json
shadowcat intercept policy test --user user123 --path /admin/users
```

**Integration Points:**
- HTTP → TransportMessage conversion utilities
- AuthContext → Rule condition evaluation
- InterceptAction → HTTP response conversion

### Testing Requirements

**Policy Evaluation Testing:**
- Unit tests for auth context rule matching
- Integration tests with realistic HTTP requests
- Performance benchmarks for policy evaluation latency
- Load testing with concurrent policy evaluations

**HTTP Integration Testing:**
- Middleware layer integration testing
- HTTP status code and header manipulation testing
- Session management with policy enforcement
- Error handling and fallback scenarios

---

## References

### Existing Codebase
- [InterceptorChain Implementation](../shadowcat/src/interceptor/engine.rs)
- [RuleBasedInterceptor Implementation](../shadowcat/src/interceptor/rules_interceptor.rs)
- [Rule Engine Core](../shadowcat/src/interceptor/rules.rs)
- [Phase 4 Completion Report](012-phase4-final-completion-report.md)

### External Policy Engines
- [OPA Performance Benchmarks](https://www.openpolicyagent.org/docs/latest/policy-performance/)
- [Cedar Policy Language Performance](https://aws.amazon.com/blogs/security/how-we-designed-cedar-to-be-intuitive-to-use-fast-and-safe/)
- [OPA Rust Integration](https://github.com/myagley/opa-rs)
- [Cedar Rust Implementation](https://github.com/cedar-policy/cedar)

### HTTP Integration Patterns  
- [Axum Middleware Documentation](https://docs.rs/axum/latest/axum/middleware/)
- [JWT Authentication with Axum 2025](https://codevoweb.com/jwt-authentication-in-rust-using-axum-framework/)
- [Axum Casbin Authorization](https://github.com/casbin-rs/axum-casbin)

---

**Conclusion:** The existing Phase 4 interceptor infrastructure provides an excellent foundation for reverse proxy security policies. Extending the RuleEngine with HTTP-specific conditions and auth context integration offers the optimal balance of performance, maintainability, and feature completeness. External policy engines like Cedar or OPA remain viable options for future advanced use cases but are not necessary for initial implementation. The measured performance characteristics indicate the enhanced system can easily meet the < 1ms policy evaluation target while maintaining all existing hot-reloading and CLI management capabilities.