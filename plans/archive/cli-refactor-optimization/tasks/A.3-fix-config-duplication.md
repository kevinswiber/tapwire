# Task A.3: Fix Configuration Duplication

## Objective
Extract the `ProxyConfig` implementation from the CLI module into the main library, eliminating duplication and establishing a single source of truth for configuration management.

## Background
Currently, configuration logic is duplicated:
- `src/cli/common.rs`: Contains ProxyConfig struct (lines 16-74)
- Configuration parsing logic spread across CLI modules
- No centralized configuration management in the library

This causes:
- Maintenance burden (changes needed in multiple places)
- Risk of configuration drift between CLI and library
- Inability for library users to configure the proxy properly

## Key Questions to Answer
1. Where should the configuration module live in the library?
2. Should we add a builder pattern now or in Phase B?
3. How do we handle CLI-specific config vs library config?
4. Should we support configuration files in this task?

## Step-by-Step Process

### 1. Analyze Current Configuration
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Find all config-related code
rg "ProxyConfig" --type rust
rg "struct.*Config" --type rust
rg "impl.*Config" --type rust

# Check what fields are in ProxyConfig
cat src/cli/common.rs | sed -n '16,74p'
```

### 2. Create Library Configuration Module
```rust
// src/config/mod.rs (new file)
pub mod proxy;
pub mod transport;

pub use proxy::{ProxyConfig, ProxyConfigBuilder};
pub use transport::TransportConfig;

// src/config/proxy.rs (new file)
use serde::{Deserialize, Serialize};

/// Core proxy configuration used by both library and CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    // Rate limiting
    pub enable_rate_limit: bool,
    pub rate_limit_rpm: u32,
    pub rate_limit_burst: u32,
    
    // Session management
    pub session_timeout_secs: u64,
    pub max_sessions: usize,
    
    // Recording
    pub enable_recording: bool,
    pub recording_dir: Option<PathBuf>,
    
    // Interceptor
    pub enable_interceptor: bool,
    pub interceptor_rules: Vec<String>,
    
    // Performance
    pub buffer_size: usize,
    pub max_frame_size: usize,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enable_rate_limit: false,
            rate_limit_rpm: 1000,
            rate_limit_burst: 100,
            session_timeout_secs: 300,
            max_sessions: 1000,
            enable_recording: false,
            recording_dir: None,
            enable_interceptor: false,
            interceptor_rules: Vec::new(),
            buffer_size: 8192,
            max_frame_size: 1024 * 1024, // 1MB
        }
    }
}

impl ProxyConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.rate_limit_rpm == 0 && self.enable_rate_limit {
            return Err(ConfigError::Invalid(
                "Rate limit RPM must be > 0 when enabled".to_string()
            ));
        }
        
        if self.max_sessions == 0 {
            return Err(ConfigError::Invalid(
                "Max sessions must be > 0".to_string()
            ));
        }
        
        Ok(())
    }
}
```

### 3. Add ProxyConfigBuilder (Basic Version)
```rust
// src/config/proxy.rs (continued)
pub struct ProxyConfigBuilder {
    config: ProxyConfig,
}

impl ProxyConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: ProxyConfig::default(),
        }
    }
    
    pub fn rate_limiting(mut self, enabled: bool, rpm: u32) -> Self {
        self.config.enable_rate_limit = enabled;
        self.config.rate_limit_rpm = rpm;
        self
    }
    
    pub fn session_timeout(mut self, secs: u64) -> Self {
        self.config.session_timeout_secs = secs;
        self
    }
    
    pub fn recording(mut self, enabled: bool, dir: Option<PathBuf>) -> Self {
        self.config.enable_recording = enabled;
        self.config.recording_dir = dir;
        self
    }
    
    pub fn build(self) -> Result<ProxyConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}
```

### 4. Create Configuration Error Type
```rust
// src/config/error.rs (new file)
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Configuration file error: {0}")]
    FileError(#[from] std::io::Error),
    
    #[error("Configuration parse error: {0}")]
    ParseError(#[from] toml::de::Error),
}
```

### 5. Update CLI to Use Library Config
```rust
// src/cli/common.rs
use shadowcat::config::{ProxyConfig, ProxyConfigBuilder};

// Remove the duplicate ProxyConfig struct (lines 16-74)

// Update the from_args method to use the builder
impl ProxyArgs {
    pub fn to_config(&self) -> Result<ProxyConfig> {
        let mut builder = ProxyConfigBuilder::new();
        
        if self.rate_limit {
            builder = builder.rate_limiting(true, self.rate_limit_rpm);
        }
        
        builder = builder
            .session_timeout(self.session_timeout)
            .recording(self.record, self.output.clone());
        
        builder.build()
            .context("Failed to build proxy configuration")
    }
}
```

### 6. Update lib.rs
```rust
// src/lib.rs
pub mod config;  // Add public config module

// Other modules...
```

### 7. Test Configuration
```rust
// tests/config.rs
use shadowcat::config::{ProxyConfig, ProxyConfigBuilder};

#[test]
fn test_default_config() {
    let config = ProxyConfig::default();
    assert!(!config.enable_rate_limit);
    assert_eq!(config.session_timeout_secs, 300);
}

#[test]
fn test_config_builder() {
    let config = ProxyConfigBuilder::new()
        .rate_limiting(true, 500)
        .session_timeout(600)
        .build()
        .unwrap();
    
    assert!(config.enable_rate_limit);
    assert_eq!(config.rate_limit_rpm, 500);
    assert_eq!(config.session_timeout_secs, 600);
}

#[test]
fn test_config_validation() {
    let result = ProxyConfigBuilder::new()
        .rate_limiting(true, 0)  // Invalid: 0 RPM
        .build();
    
    assert!(result.is_err());
}
```

## Expected Deliverables

### New Files
- `shadowcat/src/config/mod.rs` - Configuration module
- `shadowcat/src/config/proxy.rs` - ProxyConfig and builder
- `shadowcat/src/config/error.rs` - Configuration errors
- `shadowcat/tests/config.rs` - Configuration tests

### Modified Files
- `shadowcat/src/lib.rs` - Export config module
- `shadowcat/src/cli/common.rs` - Remove duplicate ProxyConfig
- `shadowcat/src/cli/forward.rs` - Use library config
- `shadowcat/src/cli/reverse.rs` - Use library config

### Verification Commands
```bash
# Build and test
cargo build --all-features
cargo test config

# Verify no duplication
rg "struct ProxyConfig" --type rust

# Check that CLI still works
cargo run -- forward stdio -- echo test

# Generate docs to verify API
cargo doc --no-deps --open
```

## Success Criteria Checklist
- [ ] Single ProxyConfig definition in library
- [ ] CLI uses library configuration
- [ ] Configuration validates itself
- [ ] Builder pattern available for library users
- [ ] All tests pass
- [ ] Configuration is documented
- [ ] No configuration logic in CLI module

## Risk Assessment
- **Risk**: Breaking existing CLI argument parsing
  - **Mitigation**: Keep same CLI args structure
  - **Mitigation**: Test all CLI commands

- **Risk**: Missing configuration fields
  - **Mitigation**: Audit current usage first
  - **Mitigation**: Add fields incrementally

## Duration Estimate
**3 hours**
- 30 min: Analyze current configuration
- 1 hour: Create library config module
- 45 min: Update CLI to use library config
- 30 min: Write tests
- 15 min: Documentation

## Dependencies
- A.1: Make CLI Module Private (clean separation)
- A.2: Remove Exit() Calls (proper error handling)

## Notes
- This establishes the pattern for all shared functionality
- Builder pattern is introduced here but will be expanded in Phase B
- Consider adding environment variable support later
- Configuration file support comes in Phase C

## Commands Reference
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Create config module structure
mkdir -p src/config
touch src/config/mod.rs
touch src/config/proxy.rs
touch src/config/error.rs

# Find and remove duplication
rg "ProxyConfig" --type rust

# Test the new configuration
cargo test config --lib

# Verify CLI still works
cargo run -- forward stdio -- echo '{"test": true}'
cargo run -- reverse --bind 127.0.0.1:8080 --upstream http://example.com
```