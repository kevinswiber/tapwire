# Task C.2: Configuration File Support

## Overview

Add comprehensive configuration file support to Shadowcat, allowing users to configure the proxy via TOML/YAML files with environment variable overrides. This enables production deployments without complex command-line arguments.

**Duration**: 3 hours  
**Dependencies**: A.3 (Fix Configuration Duplication)  
**Status**: Not Started

## Objectives

1. Design a unified configuration schema that covers all Shadowcat features
2. Implement TOML/YAML parsing with the `config` crate
3. Support environment variable overrides using standard naming conventions
4. Provide configuration validation and sensible defaults
5. Integrate with existing CLI arguments (CLI args override config files)

## Design Considerations

### Configuration Hierarchy (highest to lowest priority)

1. **CLI Arguments** - Direct command-line flags
2. **Environment Variables** - `SHADOWCAT_*` prefixed vars
3. **Config File** - User-specified or default locations
4. **Built-in Defaults** - Hardcoded sensible defaults

### Configuration File Locations

1. **User-specified**: `--config path/to/config.toml`
2. **Current directory**: `./shadowcat.toml`, `./shadowcat.yaml`
3. **User config**: `~/.config/shadowcat/config.toml`
4. **System config**: `/etc/shadowcat/config.toml`

### Configuration Schema

```toml
# shadowcat.toml - Example configuration

[server]
# Bind address for reverse proxy mode
bind = "127.0.0.1:8080"
# Worker threads (defaults to CPU count)
workers = 4
# Graceful shutdown timeout in seconds
shutdown_timeout = 30

[proxy]
# Default upstream for reverse proxy
upstream = "http://localhost:3000"
# Connection timeout in seconds
connect_timeout = 10
# Request timeout in seconds
request_timeout = 30
# Enable connection pooling
connection_pool = true
# Max connections per upstream
max_connections = 100

[transport]
# Default transport type: "stdio", "http", "sse"
default = "stdio"
# HTTP-specific settings
[transport.http]
max_body_size = "10MB"
compression = true

[session]
# Session storage backend: "memory", "sqlite"
storage = "memory"
# SQLite database path (if storage = "sqlite")
database = "./shadowcat.db"
# Session timeout in seconds
timeout = 3600
# Maximum concurrent sessions
max_sessions = 1000

[recording]
# Enable automatic recording
enabled = false
# Recording output directory
output_dir = "./recordings"
# Recording format: "json", "binary"
format = "json"
# Include timing metadata
include_timing = true

[interceptor]
# Enable interceptor chain
enabled = true
# Rules file location
rules_file = "./interceptor-rules.yaml"
# Default action: "continue", "pause", "block"
default_action = "continue"

[auth]
# OAuth 2.1 settings for reverse proxy
enabled = false
# Token validation endpoint
token_endpoint = "https://auth.example.com/oauth/token"
# Required scopes
required_scopes = ["mcp:read", "mcp:write"]
# Cache validated tokens
cache_tokens = true
cache_ttl = 300

[telemetry]
# Enable OpenTelemetry
enabled = false
# OTLP endpoint
otlp_endpoint = "http://localhost:4317"
# Service name
service_name = "shadowcat"
# Sampling rate (0.0 to 1.0)
sampling_rate = 0.1

[metrics]
# Enable Prometheus metrics
enabled = false
# Metrics endpoint
bind = "127.0.0.1:9090"
# Metrics path
path = "/metrics"

[logging]
# Log level: "error", "warn", "info", "debug", "trace"
level = "info"
# Log format: "json", "pretty", "compact"
format = "pretty"
# Log output: "stdout", "stderr", file path
output = "stderr"
# Include timestamps
timestamps = true

[rate_limiting]
# Enable rate limiting
enabled = false
# Default tier if not specified
default_tier = "standard"

[rate_limiting.tiers.standard]
requests_per_second = 100
burst_size = 200

[rate_limiting.tiers.premium]
requests_per_second = 1000
burst_size = 2000
```

### Environment Variable Mapping

Environment variables follow the pattern `SHADOWCAT_<SECTION>_<KEY>`:

```bash
SHADOWCAT_SERVER_BIND=0.0.0.0:8080
SHADOWCAT_PROXY_UPSTREAM=http://mcp-server:3000
SHADOWCAT_LOGGING_LEVEL=debug
SHADOWCAT_TELEMETRY_ENABLED=true
```

## Implementation Plan

### Step 1: Create Configuration Module Structure (30 min)

```rust
// src/config/mod.rs
pub mod schema;
pub mod loader;
pub mod validator;

// src/config/schema.rs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShadowcatConfig {
    pub server: ServerConfig,
    pub proxy: ProxyConfig,
    pub transport: TransportConfig,
    pub session: SessionConfig,
    pub recording: RecordingConfig,
    pub interceptor: InterceptorConfig,
    pub auth: AuthConfig,
    pub telemetry: TelemetryConfig,
    pub metrics: MetricsConfig,
    pub logging: LoggingConfig,
    pub rate_limiting: RateLimitingConfig,
}
```

### Step 2: Implement Configuration Loader (1 hour)

```rust
// src/config/loader.rs
use config::{Config, Environment, File};

pub struct ConfigLoader {
    sources: Vec<ConfigSource>,
}

impl ConfigLoader {
    pub fn new() -> Self { ... }
    pub fn with_file(mut self, path: &Path) -> Self { ... }
    pub fn with_env(mut self) -> Self { ... }
    pub fn with_defaults(mut self) -> Self { ... }
    pub fn load(self) -> Result<ShadowcatConfig> { ... }
}
```

### Step 3: Add Validation Layer (30 min)

```rust
// src/config/validator.rs
pub trait Validate {
    fn validate(&self) -> Result<()>;
}

impl Validate for ShadowcatConfig {
    fn validate(&self) -> Result<()> {
        // Validate bind addresses
        // Check file paths exist
        // Verify URL formats
        // Ensure valid ranges (timeouts, rates, etc.)
    }
}
```

### Step 4: Integrate with CLI (45 min)

```rust
// src/main.rs updates
#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    
    #[arg(long, env = "SHADOWCAT_LOG_LEVEL")]
    log_level: Option<String>,
    
    // ... existing args
}

// Load config before processing commands
let config = load_configuration(cli.config)?;
```

### Step 5: Update Builders to Use Config (15 min)

```rust
// src/api.rs
impl ShadowcatBuilder {
    pub fn from_config(config: &ShadowcatConfig) -> Self {
        let mut builder = Self::new();
        
        if let Some(timeout) = config.proxy.connect_timeout {
            builder = builder.with_timeout(Duration::from_secs(timeout));
        }
        
        // Apply other config settings
        builder
    }
}
```

## Testing Strategy

### Unit Tests

1. Test configuration loading from TOML
2. Test configuration loading from YAML
3. Test environment variable overrides
4. Test validation rules
5. Test default values

### Integration Tests

1. Test CLI args override config file
2. Test config file discovery in standard locations
3. Test invalid configuration handling
4. Test partial configuration with defaults

### Example Test

```rust
#[test]
fn test_config_hierarchy() {
    // Set env var
    env::set_var("SHADOWCAT_SERVER_BIND", "0.0.0.0:9000");
    
    // Load config with file
    let config = ConfigLoader::new()
        .with_file("test-config.toml")
        .with_env()
        .load()
        .unwrap();
    
    // Env var should override file
    assert_eq!(config.server.bind, "0.0.0.0:9000");
}
```

## Success Criteria

- [ ] TOML configuration files parse correctly
- [ ] YAML configuration files parse correctly
- [ ] Environment variables override file settings
- [ ] CLI arguments override all other sources
- [ ] Invalid configurations are rejected with clear errors
- [ ] Default configuration works out of the box
- [ ] Configuration is validated before use
- [ ] All existing tests still pass
- [ ] Documentation includes configuration examples

## Risk Mitigation

1. **Breaking Changes**: Use feature flag initially (`config-file` feature)
2. **Complex Schema**: Start with essential fields, expand incrementally
3. **Performance**: Lazy-load configuration only when needed
4. **Security**: Validate all file paths and URLs, never log sensitive config

## Example Usage

```bash
# Use default config file location
shadowcat forward stdio -- mcp-server

# Specify custom config
shadowcat --config production.toml reverse

# Override with env vars
SHADOWCAT_LOGGING_LEVEL=debug shadowcat forward stdio -- server

# Override with CLI args (highest priority)
shadowcat --config prod.toml forward stdio --timeout 60 -- server
```

## Files to Create/Modify

1. **Create**: `src/config/mod.rs` - Module exports
2. **Create**: `src/config/schema.rs` - Configuration structures
3. **Create**: `src/config/loader.rs` - Loading logic
4. **Create**: `src/config/validator.rs` - Validation logic
5. **Create**: `examples/shadowcat.toml` - Example config file
6. **Modify**: `src/main.rs` - Add config loading
7. **Modify**: `src/api.rs` - Add `from_config` builder method
8. **Create**: `tests/config_test.rs` - Configuration tests

## Dependencies

Already available in `Cargo.toml`:
- `config = "0.14"` - Configuration management
- `serde_yaml = "0.9"` - YAML support
- `serde`, `serde_derive` - Serialization

## Documentation to Add

1. Configuration guide in `docs/configuration.md` (already exists, needs updating)
2. Example configurations for common scenarios
3. Environment variable reference
4. Migration guide from CLI args to config file