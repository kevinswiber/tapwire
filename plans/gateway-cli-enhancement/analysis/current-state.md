# Current State Analysis: Gateway CLI

**Date**: 2025-08-15  
**Status**: Complete

## Executive Summary

The Shadowcat gateway (reverse proxy) has extensive capabilities implemented in the codebase but only exposes a minimal subset through the CLI. This analysis documents the gap between implemented functionality and CLI accessibility.

## Current CLI Options

The gateway CLI (currently `shadowcat reverse`, to be renamed `shadowcat gateway` by Better CLI Interface plan) exposes:

### Basic Options
- `--bind`: Address and port to bind to (default: "127.0.0.1:8080")
- `--upstream`: Single upstream MCP server URL (required)

### Rate Limiting
- `--enable-rate-limit`: Enable rate limiting (default: false)
- `--rate-limit-rpm`: Requests per minute (default: 60)
- `--rate-limit-burst`: Burst size (default: 10)

### Session Management
- `--session-timeout-secs`: Session timeout in seconds (default: 300)
- `--session-cleanup-interval-secs`: Cleanup interval (default: 60)
- `--max-sessions`: Maximum concurrent sessions (default: 1000)

## Module Capabilities Not Exposed

### 1. Authentication & Security
**Current State**: Not exposed via CLI
**Module Support**: Full OAuth 2.1 gateway, JWT validation
```rust
pub auth_config: Option<crate::auth::gateway::AuthGatewayConfig>
```
**Impact**: Cannot enable authentication without config file

### 2. Multiple Upstreams & Load Balancing
**Current State**: Single upstream only
**Module Support**: Multiple upstreams with various strategies
```rust
pub upstream_configs: Vec<ReverseUpstreamConfig>
pub load_balancing_strategy: ReverseLoadBalancingStrategy
```
**Strategies Available**:
- RoundRobin
- WeightedRoundRobin
- LeastConnections
- Random
- WeightedRandom
- HealthyFirst

**Impact**: No high availability or load distribution

### 3. Circuit Breaker
**Current State**: Not exposed
**Module Support**: Full circuit breaker implementation
```rust
pub circuit_breaker_config: Option<CircuitBreakerConfig>
```
**Impact**: No automatic failure recovery

### 4. Interceptors
**Current State**: Not exposed
**Module Support**: Full interceptor chain
```rust
pub interceptor_config: Option<McpInterceptorConfig>
```
**Impact**: Cannot modify/inspect messages in flight

### 5. Recording
**Current State**: Not exposed
**Module Support**: Full recording capability
```rust
pub enable_recording: bool
pub recording_dir: Option<PathBuf>
```
**Impact**: Cannot capture sessions for debugging

### 6. Audit Logging
**Current State**: Not exposed
**Module Support**: Comprehensive audit system
```rust
pub audit_config: Option<AuditConfig>
```
**Impact**: No compliance/security logging

### 7. Connection Pooling
**Current State**: Not configurable
**Module Support**: Per-upstream pool configuration
```rust
pub connection_pool: Option<ReverseUpstreamPoolConfig>
```
**Impact**: Suboptimal connection management

### 8. Health Checks
**Current State**: Not exposed
**Module Support**: Configurable health checks
```rust
pub health_check: Option<ReverseUpstreamHealthCheckConfig>
```
**Impact**: No automatic upstream monitoring

### 9. Basic Options
**Current State**: Hardcoded values
**Module Support**: Configurable
- `cors_enabled`: Hardcoded as true
- `trace_enabled`: Hardcoded as true
- `max_body_size`: Uses default constant

## Configuration Comparison

| Feature | CLI Exposed | Module Capable | Priority |
|---------|------------|----------------|----------|
| Single Upstream | ✅ | ✅ | - |
| Multiple Upstreams | ❌ | ✅ | HIGH |
| Load Balancing | ❌ | ✅ | HIGH |
| Authentication | ❌ | ✅ | HIGH |
| Circuit Breaker | ❌ | ✅ | HIGH |
| Recording | ❌ | ✅ | MEDIUM |
| Interceptors | ❌ | ✅ | MEDIUM |
| Audit Logging | ❌ | ✅ | MEDIUM |
| Connection Pooling | ❌ | ✅ | LOW |
| Health Checks | ❌ | ✅ | LOW |
| CORS Control | ❌ | ✅ | LOW |
| Body Size Limit | ❌ | ✅ | LOW |

## Implementation Patterns

### Current Pattern
The CLI directly builds configuration in the command handler:
```rust
// Direct configuration in run_gateway (currently run_reverse_proxy)
let upstream_config = if upstream.starts_with("http://") {
    ReverseUpstreamConfig::http("default", &upstream)
} else {
    // stdio handling
};
```

### Recommended Pattern
1. Support both CLI arguments and config file
2. Use builder pattern for complex configurations
3. Provide sensible defaults
4. Allow progressive enhancement

## Backward Compatibility Concerns

1. **Existing `--upstream` flag**: Must continue to work as single upstream
2. **Rate limiting flags**: Current granularity must be preserved
3. **Session management**: Existing flags must maintain behavior

## Recommendations

### Phase 1: Critical Features (Production Ready)
1. Multiple upstreams via `--upstream-file`
2. Load balancing via `--load-balancing`
3. Authentication via `--auth-config`
4. Circuit breaker basic options

### Phase 2: Observability
1. Recording via `--enable-recording`
2. Audit logging via `--audit-config`
3. Interceptor configuration

### Phase 3: Fine Tuning
1. Connection pool options
2. Health check configuration
3. CORS and body size options

## CLI Design Considerations

### Option Grouping
```bash
# Basic operation (after Better CLI Interface rename)
shadowcat gateway --bind 127.0.0.1:8080 --upstream http://server

# With config file
shadowcat gateway --config gateway.yaml

# Mixed (CLI overrides config)
shadowcat gateway --config base.yaml --enable-recording
```

### Complex Configuration via File
For complex setups, support YAML/JSON:
```yaml
upstreams:
  - id: primary
    url: http://server1:8080
    weight: 2
  - id: secondary
    url: http://server2:8080
    weight: 1
load_balancing: weighted_round_robin
circuit_breaker:
  threshold: 5
  timeout: 30
```

## Testing Requirements

1. **Backward compatibility tests**: Ensure existing CLI works
2. **Configuration precedence**: CLI > File > Defaults
3. **Validation tests**: Invalid configurations fail gracefully
4. **Integration tests**: Each feature works end-to-end

## Conclusion

The gateway module is feature-complete but the CLI severely limits its usability. Exposing these capabilities would make Shadowcat production-ready for enterprise deployments. Priority should be on features that enable high availability, security, and observability.