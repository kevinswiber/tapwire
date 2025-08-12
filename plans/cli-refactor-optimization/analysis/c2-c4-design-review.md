# C.2 and C.4 Design Review

## Executive Summary

We've designed two critical production features for Shadowcat:
1. **C.2**: Configuration File Support (3 hours)
2. **C.4**: Telemetry/Metrics (4 hours)

Both are well-designed and can be implemented independently or together. Here's our analysis and recommendations.

## C.2: Configuration File Support

### Strengths
- Comprehensive configuration schema covering all Shadowcat features
- Clear hierarchy: CLI > ENV > File > Defaults
- Uses existing `config` crate (already in dependencies)
- Standard locations for config discovery
- TOML/YAML support for flexibility

### Design Decisions to Consider

1. **Single vs Multiple Config Files**
   - Current: Single file with all sections
   - Alternative: Split configs (shadowcat.toml, interceptor-rules.yaml, auth.toml)
   - **Recommendation**: Start with single file, add includes later if needed

2. **Config Validation Timing**
   - Option A: Validate on load (fail fast)
   - Option B: Validate on use (partial configs work)
   - **Recommendation**: Validate critical fields on load, optional fields on use

3. **Feature Flags**
   - Current design doesn't mention feature gating
   - **Recommendation**: Add `config-file` feature flag for gradual rollout

### Implementation Priority
High - This is foundational for production deployments

## C.4: Telemetry/Metrics

### Strengths
- Industry-standard tools (OpenTelemetry, Prometheus)
- Comprehensive metrics coverage
- Well-structured span hierarchy
- Performance-conscious design (sampling, lazy init)

### Design Decisions to Consider

1. **Dependency Weight**
   - OpenTelemetry + Prometheus adds ~5-10MB to binary
   - **Recommendation**: Make telemetry an optional feature (`telemetry` feature flag)

2. **Metrics Granularity**
   - Current: Detailed metrics for all components
   - Risk: Cardinality explosion with many labels
   - **Recommendation**: Start with essential metrics, add more based on needs

3. **Tracing Integration**
   - Current: Uses `tracing` crate with OpenTelemetry bridge
   - Alternative: Direct OpenTelemetry API
   - **Recommendation**: Stick with `tracing` for consistency with existing code

### Implementation Priority
Medium - Important for production but not blocking

## Implementation Approach Options

### Option A: Sequential Implementation (Recommended)
1. **First**: Implement C.2 (Config Files) - 3 hours
   - Foundation for all other features
   - Immediately useful for users
   - Simpler to test in isolation

2. **Then**: Implement C.4 (Telemetry) - 4 hours
   - Can use config system from C.2
   - Build on stable foundation

**Pros**: 
- Clear dependencies
- Easier to test
- Can ship C.2 immediately

**Cons**:
- Takes longer to get both features

### Option B: Parallel Implementation
1. Implement both simultaneously
2. Wire them together at the end

**Pros**:
- Faster if we have parallel resources
- Can design with both in mind

**Cons**:
- Risk of integration issues
- Harder to test
- More context switching

### Option C: Minimal MVP First
1. Simplified config (just essential fields) - 1.5 hours
2. Basic metrics (just counters) - 1.5 hours
3. Iterate to full implementation

**Pros**:
- Quick wins
- Early feedback
- Lower risk

**Cons**:
- More refactoring later
- Users might depend on MVP API

## Specific Implementation Recommendations

### For C.2 (Config Files)

1. **Start Small**
   ```toml
   # Minimal MVP config
   [server]
   bind = "127.0.0.1:8080"
   
   [proxy]
   upstream = "http://localhost:3000"
   
   [logging]
   level = "info"
   ```

2. **Use Serde Defaults**
   ```rust
   #[derive(Deserialize)]
   #[serde(default)]
   pub struct ServerConfig {
       #[serde(default = "default_bind")]
       pub bind: String,
   }
   ```

3. **Config Loading Pattern**
   ```rust
   // Simple, testable approach
   pub fn load_config(path: Option<&Path>) -> Result<ShadowcatConfig> {
       let mut builder = Config::builder()
           .add_source(Config::try_from(&ShadowcatConfig::default())?)
           .add_source(Environment::with_prefix("SHADOWCAT").separator("_"));
       
       if let Some(path) = path {
           builder = builder.add_source(File::from(path));
       }
       
       builder.build()?.try_deserialize()
   }
   ```

### For C.4 (Telemetry)

1. **Feature Flag Everything**
   ```toml
   [features]
   default = []
   telemetry = ["opentelemetry", "opentelemetry-otlp", "prometheus"]
   ```

2. **Zero-Cost When Disabled**
   ```rust
   #[cfg(feature = "telemetry")]
   pub fn init_telemetry(config: &TelemetryConfig) -> Result<TelemetryGuard> {
       // Implementation
   }
   
   #[cfg(not(feature = "telemetry"))]
   pub fn init_telemetry(_: &TelemetryConfig) -> Result<TelemetryGuard> {
       Ok(TelemetryGuard::noop())
   }
   ```

3. **Essential Metrics Only**
   - `shadowcat_proxy_requests_total`
   - `shadowcat_proxy_request_duration_seconds`
   - `shadowcat_proxy_errors_total`
   - `shadowcat_sessions_active`

## Testing Strategy

### C.2 Testing Priorities
1. Config file parsing (TOML/YAML)
2. Environment variable overrides
3. Invalid config handling
4. Default values

### C.4 Testing Priorities
1. Metrics collection accuracy
2. Performance overhead measurement
3. Disabled telemetry has zero cost
4. Prometheus endpoint availability

## Risk Analysis

### C.2 Risks
- **Breaking changes**: Mitigate with feature flag
- **Config complexity**: Start minimal, expand gradually
- **Security**: Validate all paths/URLs, mask sensitive values in logs

### C.4 Risks
- **Performance overhead**: Use sampling, measure impact
- **Binary size**: Make optional feature
- **Cardinality explosion**: Limit label values

## Decision Matrix

| Criteria | C.2 First | C.4 First | Both Parallel | MVP First |
|----------|-----------|-----------|---------------|-----------|
| Risk | Low | Medium | High | Low |
| Time to First Feature | 3h | 4h | 7h | 1.5h |
| Time to Both Features | 7h | 7h | 7h | 8h |
| Testing Complexity | Low | Medium | High | Low |
| User Value | High | Medium | High | Medium |
| **Recommendation** | ✅ | | | |

## Final Recommendation

**Implement C.2 (Config Files) first**, then C.4 (Telemetry):

1. C.2 provides immediate value and is foundational
2. Lower risk and complexity
3. Can ship incrementally
4. C.4 can build on C.2's config system

### Suggested Timeline

**Session 1 (3 hours)**: C.2 Implementation
- Hour 1: Config schema and loader
- Hour 2: Integration with CLI and builders
- Hour 3: Testing and documentation

**Session 2 (4 hours)**: C.4 Implementation
- Hour 1: Dependencies and telemetry module
- Hour 2: Metrics implementation
- Hour 3: Instrumentation points
- Hour 4: Testing and documentation

## Questions for User

1. **Feature Flags**: Should we gate these features behind flags initially?
2. **Config Format**: Strong preference for TOML vs YAML?
3. **Metrics Backend**: Any preference besides Prometheus?
4. **Deployment Target**: Kubernetes, Docker, or bare metal?
5. **Existing Monitoring**: What observability stack is already in place?

## Next Steps

Once we decide on the approach:
1. Create feature branch for implementation
2. Update TodoWrite with detailed subtasks
3. Begin implementation following the task file
4. Create PR with clear testing instructions