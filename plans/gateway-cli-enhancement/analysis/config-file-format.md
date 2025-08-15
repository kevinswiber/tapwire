# Configuration File Format Specification

**Date**: 2025-08-15  
**Version**: 1.0  
**Status**: Draft

## Overview

This document specifies the configuration file format for the Shadowcat gateway (reverse proxy), supporting both YAML and JSON formats. The configuration system follows a hierarchical structure with sensible defaults and comprehensive validation.

## File Format Support

### Supported Formats
- **YAML** (.yaml, .yml) - Recommended for human editing
- **JSON** (.json) - For programmatic generation
- **Environment Variables** - For sensitive values and overrides

### File Detection
- Extension-based: `.yaml`, `.yml` → YAML parser, `.json` → JSON parser
- Content-based: Attempt JSON first, fall back to YAML if parsing fails

## Schema Definition

### Root Configuration Object

```yaml
# Complete configuration schema with all options
version: "1.0"  # Configuration format version

# Core settings
bind_address: "127.0.0.1:8080"  # Address to bind the gateway

# Upstream servers configuration
upstreams:
  - id: "primary"
    type: "http"  # http | stdio
    url: "http://server1:8080/api/mcp"
    weight: 2
    enabled: true
    health_check:
      enabled: true
      interval: 30
      timeout: 5
      path: "/health"
      healthy_threshold: 2
      unhealthy_threshold: 3
    connection_pool:
      max_connections: 10
      min_idle: 1
      max_idle_time: 300
      connection_timeout: 30
    circuit_breaker:
      enabled: true
      threshold: 5
      timeout: 30
      half_open_requests: 3

# Load balancing configuration
load_balancing:
  strategy: "weighted_round_robin"  # round_robin | weighted_round_robin | least_connections | random | weighted_random | healthy_first
  sticky_sessions: false
  cookie_name: "shadowcat_session"

# Authentication configuration
auth:
  enabled: true
  type: "oauth"  # oauth | jwt | basic | none
  
  # OAuth specific
  oauth:
    issuer: "https://auth.example.com"
    audience: "api.example.com"
    jwks_url: "https://auth.example.com/.well-known/jwks.json"
    scopes: ["read", "write"]
    cache_ttl: 300
  
  # JWT specific
  jwt:
    secret_file: "/path/to/secret.key"
    algorithm: "RS256"  # HS256 | RS256 | ES256
    issuer: "shadowcat"
    audience: "api"
    expiry_leeway: 60
  
  # Basic auth specific
  basic:
    users_file: "/path/to/users.htpasswd"
    realm: "Shadowcat Proxy"

# Rate limiting configuration
rate_limiting:
  enabled: true
  
  global:
    enabled: true
    requests_per_minute: 1000
    burst: 100
  
  per_ip:
    enabled: true
    requests_per_minute: 100
    burst: 20
    whitelist: ["127.0.0.1", "::1"]
    blacklist: []
  
  per_session:
    enabled: true
    requests_per_minute: 200
    burst: 40
  
  per_user:
    enabled: false
    requests_per_minute: 500
    burst: 50
  
  per_endpoint:
    enabled: false
    endpoints:
      - path: "/api/heavy"
        requests_per_minute: 10
        burst: 2

# Circuit breaker configuration
circuit_breaker:
  enabled: true
  threshold: 5
  timeout: 30
  half_open_requests: 3
  failure_rate_threshold: 0.5
  min_requests: 10

# Interceptor configuration
interceptors:
  enabled: true
  rules_file: "interceptor-rules.yaml"
  
  # Inline rules
  rules:
    - id: "block-dangerous"
      type: "block"
      pattern:
        method: "dangerous/*"
      message: "This method is blocked"
    
    - id: "log-tools"
      type: "log"
      pattern:
        method: "tools/*"
      log_level: "info"
    
    - id: "modify-headers"
      type: "modify"
      pattern:
        path: "/api/*"
      modifications:
        add_headers:
          X-Proxy: "shadowcat"
        remove_headers: ["X-Internal"]

# Recording configuration
recording:
  enabled: true
  directory: "./tapes"
  format: "tape"  # tape | jsonl
  
  # Filtering
  filter:
    include_patterns:
      - method: "tools/*"
      - path: "/api/*"
    exclude_patterns:
      - method: "system/*"
  
  # Storage
  max_size_mb: 1000
  max_files: 100
  rotation: "daily"  # daily | size | never
  compression: true

# Session management
session:
  timeout: 300  # seconds
  max_sessions: 1000
  cleanup_interval: 60
  
  storage:
    type: "memory"  # memory | redis | sqlite
    
    # Redis specific
    redis:
      url: "redis://localhost:6379"
      key_prefix: "shadowcat:"
      
    # SQLite specific
    sqlite:
      path: "./sessions.db"
      pool_size: 5

# Audit logging
audit:
  enabled: true
  file: "./audit.log"
  level: "detailed"  # basic | detailed | verbose
  
  # Log rotation
  rotation:
    enabled: true
    max_size: "100MB"
    max_age: 30  # days
    max_backups: 10
    compress: true
  
  # What to log
  events:
    - authentication
    - authorization
    - rate_limit_exceeded
    - circuit_breaker_triggered
    - session_created
    - session_expired
    - upstream_error
    - interceptor_action

# HTTP server settings
http:
  max_body_size: "10MB"
  timeout:
    read: 30
    write: 30
    idle: 120
  cors:
    enabled: true
    allow_origins: ["*"]
    allow_methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
    allow_headers: ["*"]
    expose_headers: ["X-Request-Id"]
    max_age: 3600
    allow_credentials: false

# Observability
observability:
  tracing:
    enabled: true
    level: "info"  # trace | debug | info | warn | error
    format: "json"  # json | pretty | compact
    
  metrics:
    enabled: true
    port: 9090
    path: "/metrics"
    
  health:
    enabled: true
    path: "/health"
    
  ready:
    enabled: true
    path: "/ready"

# TLS configuration
tls:
  enabled: false
  cert_file: "/path/to/cert.pem"
  key_file: "/path/to/key.pem"
  client_auth:
    enabled: false
    ca_file: "/path/to/ca.pem"
    verify_mode: "required"  # none | optional | required
```

## JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Shadowcat Gateway Configuration",
  "type": "object",
  "properties": {
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+$"
    },
    "bind_address": {
      "type": "string",
      "format": "host-port"
    },
    "upstreams": {
      "type": "array",
      "minItems": 1,
      "items": {
        "$ref": "#/definitions/upstream"
      }
    },
    "load_balancing": {
      "$ref": "#/definitions/load_balancing"
    },
    "auth": {
      "$ref": "#/definitions/auth"
    },
    "rate_limiting": {
      "$ref": "#/definitions/rate_limiting"
    },
    "circuit_breaker": {
      "$ref": "#/definitions/circuit_breaker"
    },
    "interceptors": {
      "$ref": "#/definitions/interceptors"
    },
    "recording": {
      "$ref": "#/definitions/recording"
    },
    "session": {
      "$ref": "#/definitions/session"
    },
    "audit": {
      "$ref": "#/definitions/audit"
    },
    "http": {
      "$ref": "#/definitions/http"
    },
    "observability": {
      "$ref": "#/definitions/observability"
    },
    "tls": {
      "$ref": "#/definitions/tls"
    }
  },
  "required": ["upstreams"],
  "definitions": {
    "upstream": {
      "type": "object",
      "properties": {
        "id": {
          "type": "string",
          "pattern": "^[a-zA-Z0-9_-]+$"
        },
        "type": {
          "type": "string",
          "enum": ["http", "stdio"]
        },
        "url": {
          "type": "string",
          "format": "uri"
        },
        "command": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "weight": {
          "type": "integer",
          "minimum": 1,
          "maximum": 100
        },
        "enabled": {
          "type": "boolean"
        },
        "health_check": {
          "$ref": "#/definitions/health_check"
        },
        "connection_pool": {
          "$ref": "#/definitions/connection_pool"
        },
        "circuit_breaker": {
          "$ref": "#/definitions/circuit_breaker"
        }
      },
      "required": ["id", "type"],
      "oneOf": [
        {
          "properties": {
            "type": { "const": "http" }
          },
          "required": ["url"]
        },
        {
          "properties": {
            "type": { "const": "stdio" }
          },
          "required": ["command"]
        }
      ]
    }
  }
}
```

## Configuration Examples

### Minimal Configuration

```yaml
# minimal.yaml
upstreams:
  - id: default
    type: http
    url: http://localhost:3000
```

### Development Configuration

```yaml
# development.yaml
bind_address: "127.0.0.1:8080"

upstreams:
  - id: local
    type: stdio
    command: ["python", "mcp_server.py"]

recording:
  enabled: true
  directory: "./dev-tapes"

interceptors:
  enabled: true
  rules:
    - id: debug-log
      type: log
      pattern:
        method: "*"
      log_level: debug
```

### Production Configuration

```yaml
# production.yaml
version: "1.0"
bind_address: "0.0.0.0:443"

upstreams:
  - id: primary
    type: http
    url: https://api1.internal:8443/mcp
    weight: 3
    health_check:
      enabled: true
      path: /health
      interval: 10
  
  - id: secondary
    type: http
    url: https://api2.internal:8443/mcp
    weight: 1
    health_check:
      enabled: true

load_balancing:
  strategy: weighted_round_robin
  sticky_sessions: true

auth:
  enabled: true
  type: oauth
  oauth:
    issuer: https://auth.company.com
    audience: api.company.com
    jwks_url: https://auth.company.com/.well-known/jwks.json

rate_limiting:
  enabled: true
  per_ip:
    enabled: true
    requests_per_minute: 100

circuit_breaker:
  enabled: true
  threshold: 10
  timeout: 60

tls:
  enabled: true
  cert_file: /etc/shadowcat/cert.pem
  key_file: /etc/shadowcat/key.pem

audit:
  enabled: true
  file: /var/log/shadowcat/audit.log
  level: detailed
  rotation:
    enabled: true
    max_size: 100MB
```

### High Availability Configuration

```yaml
# ha.yaml
upstreams:
  - id: dc1-primary
    type: http
    url: http://dc1-api1.internal:8080
    weight: 2
    health_check:
      enabled: true
      interval: 5
    circuit_breaker:
      enabled: true
      threshold: 3
  
  - id: dc1-secondary
    type: http
    url: http://dc1-api2.internal:8080
    weight: 2
    health_check:
      enabled: true
  
  - id: dc2-primary
    type: http
    url: http://dc2-api1.internal:8080
    weight: 1
    health_check:
      enabled: true
  
  - id: dc2-secondary
    type: http
    url: http://dc2-api2.internal:8080
    weight: 1
    health_check:
      enabled: true

load_balancing:
  strategy: weighted_round_robin

circuit_breaker:
  enabled: true
  threshold: 5
  timeout: 30
  failure_rate_threshold: 0.3
```

## Environment Variable Support

Environment variables can override configuration values:

```bash
# Override bind address
SHADOWCAT_BIND_ADDRESS=0.0.0.0:9000

# Override auth settings (for secrets)
SHADOWCAT_AUTH_JWT_SECRET=supersecret
SHADOWCAT_AUTH_OAUTH_CLIENT_SECRET=clientsecret

# Override upstream URLs
SHADOWCAT_UPSTREAM_PRIMARY_URL=http://new-server:8080
SHADOWCAT_UPSTREAM_SECONDARY_ENABLED=false

# Override rate limits
SHADOWCAT_RATE_LIMIT_GLOBAL_RPM=5000
```

### Environment Variable Naming Convention
- Prefix: `SHADOWCAT_`
- Nested paths: Use `_` separator
- Arrays: Use index or ID
- Examples:
  - `SHADOWCAT_BIND_ADDRESS`
  - `SHADOWCAT_AUTH_OAUTH_ISSUER`
  - `SHADOWCAT_UPSTREAM_PRIMARY_URL`
  - `SHADOWCAT_RATE_LIMIT_PER_IP_RPM`

## Validation Rules

### Required Fields
- At least one upstream must be defined
- Upstream must have either `url` (http) or `command` (stdio)
- If auth is enabled, type must be specified
- If auth type is oauth, issuer and audience are required
- If auth type is jwt, secret_file or secret is required

### Value Constraints
```yaml
# Numeric ranges
weight: 1-100
requests_per_minute: 1-1000000
burst: 1-10000
timeout: 1-3600
max_connections: 1-1000
threshold: 1-100

# String formats
bind_address: "host:port"
url: "valid URL"
id: "alphanumeric + dash + underscore"
path: "valid file path"

# Enumerations
strategy: [round_robin, weighted_round_robin, least_connections, random, weighted_random, healthy_first]
auth_type: [oauth, jwt, basic, none]
format: [tape, jsonl]
level: [trace, debug, info, warn, error]
```

### Logical Constraints
1. Load balancing strategy requires multiple upstreams
2. Weighted strategies require weights on upstreams
3. Sticky sessions require session storage configuration
4. Circuit breaker threshold must be less than window size
5. Pool min_idle must be less than max_connections

## Configuration Merging

When multiple configuration sources are present:

### Precedence Order (highest to lowest)
1. Command-line arguments
2. Environment variables
3. Configuration file(s)
4. Default values

### Merge Strategy
- **Scalars**: Higher precedence overwrites
- **Arrays**: Higher precedence replaces entire array
- **Objects**: Deep merge with field-level precedence

### Multiple Configuration Files
```bash
shadowcat gateway --config base.yaml --config overrides.yaml
```

Files are merged left-to-right:
1. Load base.yaml
2. Deep merge overrides.yaml
3. Apply CLI arguments

## Error Handling

### Validation Errors
```yaml
# ERROR: Invalid upstream type
upstreams:
  - id: bad
    type: websocket  # ❌ Not a valid type

# ERROR: Missing required field
auth:
  enabled: true
  # type is required when enabled ❌

# ERROR: Invalid range
rate_limiting:
  global:
    requests_per_minute: -1  # ❌ Must be positive
```

### Error Messages
```
Configuration Error at upstreams[0].type:
  Invalid value 'websocket'. Expected one of: http, stdio

Configuration Error at auth:
  Missing required field 'type' when auth.enabled is true

Configuration Error at rate_limiting.global.requests_per_minute:
  Value -1 is out of range. Must be between 1 and 1000000
```

## Migration Path

### From CLI Arguments to Config File

Current CLI:
```bash
shadowcat gateway \
  --bind 127.0.0.1:8080 \
  --upstream http://server \
  --enable-rate-limit \
  --rate-limit-rpm 100
```

Equivalent config.yaml:
```yaml
bind_address: "127.0.0.1:8080"
upstreams:
  - id: default
    type: http
    url: http://server
rate_limiting:
  enabled: true
  global:
    requests_per_minute: 100
```

### Config Version Migration
```yaml
# Version 1.0 (current)
version: "1.0"
upstreams:
  - id: server
    url: http://server

# Future Version 2.0 (example)
version: "2.0"
services:  # renamed from upstreams
  - name: server  # renamed from id
    endpoint: http://server  # renamed from url
```

## Default Configuration

When no configuration is provided, these defaults apply:

```yaml
# Implicit defaults
version: "1.0"
bind_address: "127.0.0.1:8080"

load_balancing:
  strategy: round_robin

rate_limiting:
  enabled: false
  global:
    requests_per_minute: 60
    burst: 10

session:
  timeout: 300
  max_sessions: 1000
  cleanup_interval: 60
  storage:
    type: memory

http:
  max_body_size: "10MB"
  cors:
    enabled: true

observability:
  tracing:
    enabled: true
    level: info
  health:
    enabled: true
    path: /health
```

## Best Practices

### Security
1. Never commit secrets to config files
2. Use environment variables for sensitive values
3. Use file permissions to protect config files
4. Validate all external inputs

### Organization
1. Use separate files for different environments
2. Keep base configuration minimal
3. Use overlays for environment-specific settings
4. Document custom configurations

### Performance
1. Enable connection pooling for HTTP upstreams
2. Configure appropriate circuit breaker thresholds
3. Set reasonable rate limits
4. Use health checks for automatic failover

### Maintenance
1. Always specify version field
2. Use descriptive IDs for upstreams
3. Comment complex configurations
4. Test configuration changes in staging

## Tooling Support

### Validation Tool
```bash
# Validate configuration file
shadowcat config validate gateway.yaml

# Generate example configuration
shadowcat config generate --type gateway > example.yaml

# Convert between formats
shadowcat config convert config.yaml --output config.json
```

### Schema Export
```bash
# Export JSON Schema
shadowcat config schema --format json > schema.json

# Export YAML Schema
shadowcat config schema --format yaml > schema.yaml
```

### Configuration Testing
```bash
# Dry run with configuration
shadowcat gateway --config test.yaml --dry-run

# Test configuration loading
shadowcat gateway --config test.yaml --validate-only
```