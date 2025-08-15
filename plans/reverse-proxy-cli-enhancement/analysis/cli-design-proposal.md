# Reverse Proxy CLI Design Proposal

**Date**: 2025-08-15  
**Status**: Draft  
**Version**: 1.0

## Executive Summary

This proposal outlines a comprehensive design of the Shadowcat reverse proxy CLI to expose the full capabilities of the reverse proxy module with a clean, intuitive interface unconstrained by legacy requirements.

## Design Principles

1. **Clean Design**: No legacy constraints - design the ideal CLI from scratch
2. **Progressive Disclosure**: Basic usage is simple; advanced features are optional
3. **Configuration Priority**: CLI args > Config file > Environment vars > Defaults
4. **Consistency**: Match patterns from other Shadowcat commands (e.g., intercept command)
5. **Discoverability**: Features should be easy to find in help
6. **Composability**: Options should work well together

## Proposed CLI Structure

### Basic Usage (Simplified)
```bash
# Simple single upstream
shadowcat reverse http://server
shadowcat reverse stdio "echo test"

# With custom bind address
shadowcat reverse --bind 127.0.0.1:8080 http://server
```

### Configuration File Support
```bash
# Primary configuration via file
shadowcat reverse --config reverse-proxy.yaml

# Override specific settings
shadowcat reverse --config base.yaml --bind 0.0.0.0:9000

# Multiple config files with merging
shadowcat reverse --config base.yaml --config overrides.yaml
```

### Multiple Upstreams
```bash
# Multiple upstreams as positional arguments
shadowcat reverse http://server1:8080 http://server2:8080

# With load balancing strategy
shadowcat reverse \
  --load-balancing weighted-round-robin \
  "http://server1:8080,weight=2" \
  "http://server2:8080,weight=1"

# Named upstreams for better tracking
shadowcat reverse \
  "primary=http://server1:8080" \
  "secondary=http://server2:8080"
```

### Authentication Configuration
```bash
# OAuth 2.1 gateway with inline configuration
shadowcat reverse \
  --auth-type oauth \
  --auth-issuer https://auth.example.com \
  --auth-audience api.example.com \
  --auth-jwks-url https://auth.example.com/.well-known/jwks.json

# JWT validation
shadowcat reverse \
  --auth-type jwt \
  --auth-secret-file /path/to/secret.key \
  --auth-algorithm RS256

# Auth configuration file
shadowcat reverse --auth-config auth.yaml
```

### Circuit Breaker
```bash
# Basic circuit breaker
shadowcat reverse \
  --upstream http://server \
  --circuit-breaker \
  --circuit-threshold 5 \
  --circuit-timeout 30

# Per-upstream circuit breakers
shadowcat reverse \
  --upstream "primary=http://server1,circuit=5/30" \
  --upstream "secondary=http://server2,circuit=10/60"
```

### Recording and Replay
```bash
# Enable recording with default directory
shadowcat reverse --upstream http://server --enable-recording

# Custom recording directory
shadowcat reverse --upstream http://server \
  --enable-recording \
  --recording-dir ./tapes

# Recording with session filtering
shadowcat reverse --upstream http://server \
  --enable-recording \
  --recording-filter "method=tools/*"
```

### Interceptors
```bash
# Load interceptor rules from file
shadowcat reverse --upstream http://server \
  --interceptor-rules rules.yaml

# Inline interceptor for simple cases
shadowcat reverse --upstream http://server \
  --intercept-block "method=dangerous/*" \
  --intercept-log "method=tools/*"
```

### Advanced Options
```bash
# Connection pooling
shadowcat reverse --upstream http://server \
  --pool-max-connections 50 \
  --pool-min-idle 5 \
  --pool-idle-timeout 300

# Health checks
shadowcat reverse \
  --upstream http://server \
  --health-check \
  --health-interval 30 \
  --health-timeout 5 \
  --health-path /health

# Body size and CORS
shadowcat reverse --upstream http://server \
  --max-body-size 10MB \
  --cors-disable

# Audit logging
shadowcat reverse --upstream http://server \
  --audit-log ./audit.log \
  --audit-level detailed
```

## Argument Organization

### Positional Arguments
- `<UPSTREAMS>...` - One or more upstream server specifications

### Core Operation
- `--bind <ADDR>` - Bind address (default: 127.0.0.1:8080)
- `--config <FILE>` - Configuration file (YAML/JSON)

### Upstream Configuration
- `--load-balancing <STRATEGY>` - Load balancing strategy
- `--upstream-timeout <SECS>` - Upstream connection timeout

### Authentication & Security
- `--auth-type <TYPE>` - Authentication type (oauth, jwt, none)
- `--auth-config <FILE>` - Authentication configuration file
- `--auth-issuer <URL>` - OAuth issuer URL
- `--auth-audience <AUD>` - Expected audience
- `--auth-jwks-url <URL>` - JWKS endpoint
- `--auth-secret-file <FILE>` - JWT secret key file
- `--auth-algorithm <ALG>` - JWT algorithm

### Rate Limiting (existing, enhanced)
- `--enable-rate-limit` - Enable rate limiting
- `--rate-limit-rpm <N>` - Requests per minute
- `--rate-limit-burst <N>` - Burst size
- `--rate-limit-per-ip` - Enable per-IP limiting
- `--rate-limit-per-session` - Enable per-session limiting

### Circuit Breaker
- `--circuit-breaker` - Enable circuit breaker
- `--circuit-threshold <N>` - Failure threshold
- `--circuit-timeout <SECS>` - Recovery timeout
- `--circuit-half-open-requests <N>` - Requests in half-open state

### Recording
- `--enable-recording` - Enable session recording
- `--recording-dir <DIR>` - Recording directory
- `--recording-filter <PATTERN>` - Filter recorded sessions
- `--recording-format <FORMAT>` - Recording format (tape, jsonl)

### Interceptors
- `--interceptor-rules <FILE>` - Load interceptor rules
- `--intercept-block <PATTERN>` - Block matching requests
- `--intercept-log <PATTERN>` - Log matching requests
- `--intercept-pause` - Enable pause mode

### Connection Management
- `--pool-max-connections <N>` - Max connections per upstream
- `--pool-min-idle <N>` - Minimum idle connections
- `--pool-idle-timeout <SECS>` - Idle connection timeout
- `--pool-connection-timeout <SECS>` - Connection timeout

### Health Checks
- `--health-check` - Enable health checks
- `--health-interval <SECS>` - Check interval
- `--health-timeout <SECS>` - Check timeout
- `--health-path <PATH>` - Health check path
- `--health-threshold-healthy <N>` - Healthy threshold
- `--health-threshold-unhealthy <N>` - Unhealthy threshold

### Session Management (existing)
- `--session-timeout-secs <SECS>` - Session timeout
- `--session-cleanup-interval-secs <SECS>` - Cleanup interval
- `--max-sessions <N>` - Maximum concurrent sessions

### Miscellaneous
- `--max-body-size <SIZE>` - Maximum request body size
- `--cors-disable` - Disable CORS
- `--trace-disable` - Disable request tracing
- `--audit-log <FILE>` - Audit log file
- `--audit-level <LEVEL>` - Audit detail level

## Environment Variables

Support for environment variable configuration for sensitive values:

```bash
SHADOWCAT_AUTH_SECRET=secret shadowcat reverse --auth-type jwt
SHADOWCAT_UPSTREAM_1=http://server1:8080
SHADOWCAT_UPSTREAM_2=http://server2:8080
```

## Validation Rules

### Argument Validation
1. At least one upstream must be specified (via CLI or config)
2. Load balancing requires multiple upstreams
3. Auth configuration requires auth type
4. Circuit breaker thresholds must be positive
5. Pool sizes must be reasonable (1-1000)
6. Timeouts must be positive

### Configuration File Validation
1. Schema validation for YAML/JSON
2. Required fields check
3. Type validation
4. Range validation
5. Mutual exclusivity rules

### Error Messages
```
Error: Multiple upstreams specified but no load balancing strategy selected
Hint: Add --load-balancing <strategy> to enable load balancing

Error: Authentication type 'oauth' requires --auth-issuer
Hint: Specify the OAuth issuer URL with --auth-issuer <URL>

Error: Circuit breaker threshold (0) must be positive
Hint: Use --circuit-threshold <N> with N > 0
```

## Implementation Phases

### Phase 1: Core Features (v0.2.0)
- Positional arguments for upstreams
- `--config` support
- `--load-balancing` strategies
- `--enable-recording`

### Phase 2: Enhanced Features (v0.3.0)
- Authentication options
- Circuit breaker options
- Interceptor support
- Connection pooling

### Phase 3: Advanced Features (v0.4.0)
- Health checks
- Audit logging
- Complex interceptor rules
- Performance tuning options

### Phase 4: Documentation (v0.5.0)
- Comprehensive examples
- Configuration guide
- Best practices guide

## Example Commands

### Simple Development Setup
```bash
# Basic reverse proxy (positional argument)
shadowcat reverse http://localhost:3000
```

### Production Setup with HA
```bash
# High availability with load balancing
shadowcat reverse \
  --load-balancing weighted-round-robin \
  --circuit-breaker \
  --health-check \
  --enable-recording \
  "primary=http://server1:8080,weight=2" \
  "secondary=http://server2:8080,weight=1"
```

### Secure API Gateway
```bash
# OAuth-protected API gateway
shadowcat reverse \
  --config production.yaml \
  --auth-type oauth \
  --auth-issuer https://auth.company.com \
  --enable-rate-limit \
  --audit-log /var/log/shadowcat/audit.log
```

### Development with Debugging
```bash
# Development setup with interceptors
shadowcat reverse \
  --interceptor-rules debug-rules.yaml \
  --enable-recording \
  --recording-dir ./debug-sessions \
  stdio "python mcp_server.py"
```

## Configuration File Example

When `--config` is specified, load from YAML/JSON:

```yaml
# reverse-proxy.yaml
bind_address: "0.0.0.0:8080"

upstreams:
  - id: primary
    url: "http://server1:8080"
    weight: 2
    health_check:
      enabled: true
      path: "/health"
      interval: 30
  - id: secondary
    url: "http://server2:8080"
    weight: 1

load_balancing: weighted_round_robin

auth:
  type: oauth
  issuer: "https://auth.example.com"
  audience: "api.example.com"
  jwks_url: "https://auth.example.com/.well-known/jwks.json"

circuit_breaker:
  threshold: 5
  timeout: 30
  half_open_requests: 3

rate_limiting:
  enabled: true
  requests_per_minute: 100
  burst: 20
  per_ip: true

recording:
  enabled: true
  directory: "./tapes"
  format: "tape"

audit:
  enabled: true
  file: "./audit.log"
  level: "detailed"
```

## Help Text Structure

```
shadowcat reverse - Run a reverse proxy server for MCP

USAGE:
    shadowcat reverse [OPTIONS] <UPSTREAMS>...

ARGS:
    <UPSTREAMS>...    One or more upstream servers (URLs or stdio commands)

OPTIONS:
    Core:
        --bind <ADDR>              Bind address [default: 127.0.0.1:8080]
        --config <FILE>            Load configuration from file
        
    Load Balancing:
        --load-balancing <STRAT>   Load balancing strategy
                                  [round-robin, weighted, least-conn, random]
        
    Authentication:
        --auth-type <TYPE>         Authentication type [oauth, jwt, none]
        --auth-config <FILE>       Authentication configuration file
        --auth-issuer <URL>        OAuth issuer URL
        --auth-audience <AUD>      Expected audience
        
    Rate Limiting:
        --enable-rate-limit        Enable rate limiting
        --rate-limit-rpm <N>       Requests per minute [default: 60]
        --rate-limit-burst <N>     Burst size [default: 10]
        
    Circuit Breaker:
        --circuit-breaker          Enable circuit breaker
        --circuit-threshold <N>    Failure threshold [default: 5]
        --circuit-timeout <S>      Recovery timeout [default: 30]
        
    Recording:
        --enable-recording         Enable session recording
        --recording-dir <DIR>      Recording directory [default: ./tapes]
        
    Session Management:
        --session-timeout <S>      Session timeout [default: 300]
        --max-sessions <N>         Max concurrent sessions [default: 1000]
        
    Advanced:
        --max-body-size <SIZE>     Max request body size [default: 10MB]
        --cors-disable             Disable CORS
        --audit-log <FILE>         Enable audit logging

EXAMPLES:
    # Basic reverse proxy
    shadowcat reverse http://localhost:3000
    
    # Multiple upstreams with load balancing
    shadowcat reverse \
        --load-balancing round-robin \
        http://server1:8080 \
        http://server2:8080
    
    # With configuration file
    shadowcat reverse --config reverse-proxy.yaml
    
    # Secure API gateway
    shadowcat reverse \
        --auth-type oauth \
        --auth-issuer https://auth.example.com \
        --enable-rate-limit \
        http://api-server:8080

For more examples and detailed documentation, see:
https://github.com/shadowcat/docs/reverse-proxy
```

## Testing Strategy

### Unit Tests
- Argument parsing validation
- Configuration merging logic
- Validation rule enforcement
- Default value application

### Integration Tests
```rust
#[test]
fn test_backward_compatibility() {
    // Existing commands must work
}

#[test]
fn test_multiple_upstreams() {
    // Multiple upstream parsing
}

#[test]
fn test_config_file_loading() {
    // YAML/JSON parsing
}

#[test]
fn test_config_precedence() {
    // CLI > File > Env > Defaults
}
```

### End-to-End Tests
- Single upstream (backward compat)
- Multiple upstreams with load balancing
- Authentication flow
- Circuit breaker behavior
- Recording and replay

## Implementation Notes

### Code Structure
```rust
// src/cli/reverse.rs
pub struct ReverseCommand {
    // Core
    #[arg(long)]
    pub bind: Option<String>,
    
    #[arg(long)]
    pub config: Option<PathBuf>,
    
    // Upstreams (multiple values)
    #[arg(long, action = Append)]
    pub upstream: Vec<String>,
    
    #[arg(long)]
    pub load_balancing: Option<LoadBalancingStrategy>,
    
    // ... other options
}

impl ReverseCommand {
    pub fn build_config(&self) -> Result<ReverseProxyConfig> {
        let mut config = if let Some(file) = &self.config {
            load_config_file(file)?
        } else {
            ReverseProxyConfig::default()
        };
        
        // Apply CLI overrides
        self.apply_overrides(&mut config)?;
        
        // Validate final configuration
        config.validate()?;
        
        Ok(config)
    }
}
```

### Configuration Builder Pattern
```rust
pub struct ReverseProxyConfigBuilder {
    config: ReverseProxyConfig,
}

impl ReverseProxyConfigBuilder {
    pub fn from_file(path: &Path) -> Result<Self> { ... }
    pub fn from_cli(args: &ReverseCommand) -> Result<Self> { ... }
    pub fn merge(self, other: ReverseProxyConfig) -> Self { ... }
    pub fn validate(self) -> Result<ReverseProxyConfig> { ... }
}
```

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing usage | HIGH | Extensive backward compatibility testing |
| Complex CLI syntax | MEDIUM | Progressive disclosure, good help text |
| Configuration conflicts | MEDIUM | Clear precedence rules, validation |
| Performance overhead | LOW | Lazy initialization, efficient parsing |

## Success Metrics

1. **Clean Interface**: Intuitive positional arguments for upstreams
2. **Feature Coverage**: 100% of module capabilities exposed
3. **User Experience**: Clear help, helpful errors
4. **Performance**: < 100ms configuration parsing
5. **Testing**: > 90% code coverage

## Conclusion

This design enables full access to reverse proxy capabilities while maintaining simplicity for basic use cases. The progressive disclosure approach ensures new users aren't overwhelmed while power users can access all features. Configuration file support enables complex enterprise deployments while CLI arguments provide quick overrides and simple setups.