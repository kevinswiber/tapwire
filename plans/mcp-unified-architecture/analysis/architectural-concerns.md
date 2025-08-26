# Architectural Concerns and Design Requirements

## 1. Storage Backends Clarification

### Decision Update
- **NO SQLite** for session storage
- **Redis** will be the distributed storage option (future)
- **In-Memory** is the default and primary implementation

```toml
[features]
default = ["sse", "metrics"]
redis-sessions = ["redis", "serde_json"]  # Future feature
# No SQLite feature
```

## 2. Session Cleanup Architecture (Critical)

### Current Problems (from shadowcat analysis)
The reverse proxy has serious session cleanup issues:

#### ⚠️ Critical Issues
1. **Cleanup task not started**: SessionManager created but `start_cleanup_task()` never called
2. **Config mapping broken**: ReverseSessionConfig values don't reach SessionConfig
3. **Memory growth risk**: Sessions accumulate until hard limit, then emergency cleanup

### Required Cleanup Architecture

#### Multi-Tier Cleanup Strategy
```rust
pub enum CleanupTrigger {
    Periodic,      // Regular interval (60s default)
    MemoryPressure, // When approaching limits
    Emergency,      // When over limits
    Shutdown,       // Graceful shutdown
}

pub struct CleanupStrategy {
    // Regular cleanup (every 60s)
    regular: CleanupParams {
        max_idle: Duration::from_secs(3600),  // 1 hour
        max_age: Duration::from_secs(86400),  // 24 hours
    },
    
    // Emergency cleanup (memory pressure)
    emergency: CleanupParams {
        max_idle: Duration::from_secs(300),   // 5 minutes
        max_age: Duration::from_secs(3600),   // 1 hour
    },
    
    // LRU eviction (last resort)
    lru_target_percent: f32,  // Keep at 90% capacity
}
```

#### Cleanup Task Lifecycle
```rust
impl SessionManager {
    pub fn new(store: Arc<dyn SessionStore>, config: SessionConfig) -> Self {
        let manager = Self { ... };
        
        // Auto-start cleanup task (decision: start by default)
        manager.start_cleanup_task();
        
        manager
    }
    
    fn start_cleanup_task(&self) {
        let cleanup_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.cleanup_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        self.run_regular_cleanup().await;
                    }
                    _ = self.memory_pressure_signal.notified() => {
                        self.run_emergency_cleanup().await;
                    }
                    _ = self.shutdown.cancelled() => {
                        self.run_shutdown_cleanup().await;
                        break;
                    }
                }
            }
        });
    }
}
```

#### Memory Pressure Detection
```rust
impl SessionManager {
    async fn check_memory_pressure(&self) -> MemoryStatus {
        let current = self.sessions.read().await.len();
        let max = self.config.max_sessions.unwrap_or(usize::MAX);
        
        match (current as f32 / max as f32) {
            x if x > 0.95 => MemoryStatus::Critical,   // >95% - emergency
            x if x > 0.85 => MemoryStatus::High,       // >85% - aggressive
            x if x > 0.75 => MemoryStatus::Moderate,   // >75% - proactive
            _ => MemoryStatus::Normal,
        }
    }
    
    async fn handle_memory_pressure(&self, status: MemoryStatus) {
        match status {
            MemoryStatus::Critical => {
                self.run_emergency_cleanup().await;
                self.evict_lru_sessions(10).await;  // Evict 10%
            }
            MemoryStatus::High => {
                self.run_aggressive_cleanup().await;
            }
            MemoryStatus::Moderate => {
                self.run_proactive_cleanup().await;
            }
            MemoryStatus::Normal => {}
        }
    }
}
```

### Configuration Requirements

#### Proper Config Mapping
```rust
// MCP crate config
pub struct SessionConfig {
    // Cleanup intervals
    pub cleanup_interval: Duration,
    pub cleanup_on_shutdown: bool,
    
    // Session limits
    pub max_sessions: Option<usize>,
    pub max_idle_time: Option<Duration>,
    pub max_session_age: Option<Duration>,
    
    // Memory management
    pub memory_pressure_threshold: f32,  // 0.85 = 85%
    pub lru_eviction_percent: f32,       // 0.10 = 10%
    
    // Rate limiting
    pub max_requests_per_second: u32,
}

// Ensure proxy configs map properly
impl From<ReverseSessionConfig> for SessionConfig {
    fn from(reverse: ReverseSessionConfig) -> Self {
        SessionConfig {
            cleanup_interval: Duration::from_secs(reverse.cleanup_interval_secs),
            max_sessions: Some(reverse.max_sessions),
            max_idle_time: Some(Duration::from_secs(reverse.session_timeout_secs)),
            // ... proper mapping
        }
    }
}
```

## 3. Protocol Version Handling with Interceptors

### The Problem
MCP has multiple protocol versions (2024-11-05, 2025-03-26, 2025-06-18). Interceptors need version awareness for:

1. **Version-Specific Validation**
   ```rust
   // Interceptor needs to know protocol version to validate
   impl ProtocolValidator {
       async fn intercept(&self, ctx: &InterceptContext) -> Result<InterceptAction> {
           let version = ctx.protocol_version.as_ref()
               .ok_or(Error::MissingVersion)?;
           
           match version.as_str() {
               "2024-11-05" => self.validate_v2024(ctx),
               "2025-03-26" => self.validate_v2025_03(ctx),
               "2025-06-18" => self.validate_v2025_06(ctx),
               _ => Err(Error::UnsupportedVersion),
           }
       }
   }
   ```

2. **Feature Availability**
   ```rust
   // Some features only available in newer versions
   if ctx.protocol_version < "2025-03-26" && ctx.message.has_batch() {
       return InterceptAction::Block { 
           reason: "Batch not supported in this version".into() 
       };
   }
   ```

3. **Message Transformation**
   ```rust
   // Convert between protocol versions
   impl VersionAdapter {
       async fn intercept(&self, ctx: &InterceptContext) -> Result<InterceptAction> {
           if self.target_version != ctx.protocol_version {
               let converted = self.convert_message(
                   ctx.message.clone(),
                   &ctx.protocol_version,
                   &self.target_version
               )?;
               return Ok(InterceptAction::Modify(converted));
           }
           Ok(InterceptAction::Continue)
       }
   }
   ```

### Design Requirement
```rust
pub struct InterceptContext {
    pub message: ProtocolMessage,
    pub direction: Direction,
    pub session_id: SessionId,
    pub transport_type: TransportType,
    pub protocol_version: Option<ProtocolVersion>, // Critical for interceptors
    pub timestamp: Instant,
    pub frame_id: u64,
    pub metadata: BTreeMap<String, String>,
}

// Session tracks negotiated version
pub struct Session {
    pub id: SessionId,
    pub protocol_version: Option<ProtocolVersion>, // Set after initialize
    // ...
}
```

### Version Negotiation Flow
```rust
impl SessionManager {
    pub async fn track_initialize(&self, session_id: SessionId, msg: &ProtocolMessage) {
        if let Some(session) = self.get_session_mut(session_id).await {
            // Extract version from initialize response
            if msg.is_initialize_response() {
                let version = extract_protocol_version(msg);
                session.protocol_version = Some(version);
                
                // Notify interceptors of version
                self.notify_version_negotiated(session_id, version).await;
            }
        }
    }
}
```

## 4. Interceptor Ordering and Version Handling (Finalized)

### Ordering Decision: Registration Order
**Decision**: Use registration order (first registered, first executed)
**Rationale**: Simplest, predictable, works for most cases

```rust
impl InterceptorChain {
    pub async fn register(&mut self, interceptor: Arc<dyn Interceptor>) {
        self.interceptors.push(interceptor);
        // Order maintained by Vec, no sorting needed
    }
}

// Usage - order matters!
chain.register(auth_interceptor).await;      // First
chain.register(rate_limit_interceptor).await; // Second
chain.register(validator_interceptor).await;  // Third
```

### Version Handling: Interceptor Self-Selection
**Decision**: Interceptors decide whether to handle based on context
**Optimization**: Optional version hints for performance

```rust
#[async_trait]
pub trait Interceptor: Send + Sync {
    /// Optional: Declare supported versions for optimization
    /// Return None to handle all versions (checked at runtime)
    fn supported_versions(&self) -> Option<Vec<ProtocolVersion>> {
        None // Default: interceptor decides per-request
    }
    
    /// Main intercept method - interceptor decides based on context
    async fn intercept(&self, context: &InterceptContext) -> Result<InterceptAction> {
        // Interceptor can check version and decide
        if let Some(version) = &context.protocol_version {
            if !self.can_handle_version(version) {
                return Ok(InterceptAction::Continue); // Skip silently
            }
        }
        
        // Process the request
        self.process(context).await
    }
    
    /// Helper for version checking (implementor can override)
    fn can_handle_version(&self, version: &ProtocolVersion) -> bool {
        true // Default: handle all versions
    }
}
```

### Optimized Chain Execution
```rust
impl InterceptorChain {
    pub async fn intercept(&self, context: &InterceptContext) -> Result<InterceptAction> {
        for interceptor in &self.interceptors {
            // Performance optimization: skip if version not supported
            if let Some(supported) = interceptor.supported_versions() {
                if let Some(ctx_version) = &context.protocol_version {
                    if !supported.contains(ctx_version) {
                        continue; // Skip without calling intercept()
                    }
                }
            }
            
            // Let interceptor handle
            match interceptor.intercept(context).await? {
                InterceptAction::Continue => continue,
                action => return Ok(action), // Stop on any non-continue
            }
        }
        
        Ok(InterceptAction::Continue)
    }
}
```

### Example: Version-Aware Interceptor
```rust
pub struct BatchValidatorInterceptor;

#[async_trait]
impl Interceptor for BatchValidatorInterceptor {
    fn supported_versions(&self) -> Option<Vec<ProtocolVersion>> {
        // Only handle versions that support batch
        Some(vec![
            ProtocolVersion::V2025_03_26,
            ProtocolVersion::V2025_06_18,
        ])
    }
    
    async fn intercept(&self, context: &InterceptContext) -> Result<InterceptAction> {
        // We know version supports batch if we get here
        if context.message.is_batch() {
            // Validate batch format
            self.validate_batch(&context.message)?;
        }
        Ok(InterceptAction::Continue)
    }
}

## 5. Additional Architectural Requirements

### Connection Pool Coordination
```rust
// Pool and SessionManager are independent but coordinated
impl Server {
    async fn handle_connection(&self, conn: Connection) {
        // 1. Pool manages connection
        let pooled = self.pool.manage(conn);
        
        // 2. SessionManager tracks session
        let session = self.session_manager.create_session();
        
        // 3. Link them via metadata
        session.metadata.insert("connection_id", pooled.id());
        
        // 4. On cleanup, notify both
        scopeguard::defer! {
            self.session_manager.cleanup(session.id);
            self.pool.release(pooled);
        }
    }
}
```

### Graceful Shutdown Sequence
```rust
impl Server {
    async fn shutdown(&self) {
        // 1. Stop accepting new connections
        self.shutdown.cancel();
        
        // 2. Notify all interceptors
        self.interceptor_chain.shutdown().await;
        
        // 3. Flush pending sessions
        self.session_manager.flush_pending().await;
        
        // 4. Run final cleanup
        self.session_manager.run_shutdown_cleanup().await;
        
        // 5. Wait for connections to close (with timeout)
        timeout(Duration::from_secs(30), 
            self.active_connections.shutdown()).await;
    }
}
```

## Implementation Priorities

### Must Have (Phase 1)
- [ ] Proper cleanup task lifecycle
- [ ] Config mapping that works
- [ ] Memory pressure handling
- [ ] Protocol version in context

### Should Have (Phase 2)  
- [ ] Multi-tier cleanup strategies
- [ ] LRU eviction
- [ ] Interceptor ordering mechanism
- [ ] Version-aware interceptors

### Nice to Have (Phase 3)
- [ ] Advanced memory metrics
- [ ] Cleanup strategy tuning
- [ ] Interceptor dependency resolution
- [ ] Protocol version adapters

## Testing Requirements

### Session Cleanup Tests
```rust
#[tokio::test]
async fn test_cleanup_task_starts_automatically() {
    let manager = SessionManager::new(store, config);
    // Should start cleanup task automatically
    assert!(manager.cleanup_handle.read().await.is_some());
}

#[tokio::test]
async fn test_memory_pressure_cleanup() {
    let manager = create_manager_near_capacity();
    manager.create_session().await; // Triggers pressure
    
    // Should run emergency cleanup
    assert!(manager.session_count() < manager.max_sessions());
}
```

### Protocol Version Tests
```rust
#[tokio::test]
async fn test_interceptor_receives_protocol_version() {
    let interceptor = VersionAwareInterceptor::new();
    let context = InterceptContext {
        protocol_version: Some("2025-06-18".into()),
        // ...
    };
    
    let action = interceptor.intercept(&context).await?;
    assert!(interceptor.saw_version("2025-06-18"));
}
```