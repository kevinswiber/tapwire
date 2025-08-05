# Task 010: CLI Updates and Documentation

**Phase:** 5 (Reverse Proxy & Authentication)  
**Week:** 2 (Security & Integration)  
**Day:** 10  
**Priority:** High  
**Estimated Time:** 6-8 hours

## Overview

Complete the reverse proxy implementation with comprehensive CLI management capabilities and production-ready documentation. Update the existing CLI to support reverse proxy operations, configuration management, monitoring, and troubleshooting. Create complete deployment and operational documentation.

## Success Criteria

- [ ] CLI extended with reverse proxy management commands
- [ ] Configuration management commands for all components
- [ ] Monitoring and metrics commands integrated
- [ ] Troubleshooting and debugging CLI utilities
- [ ] Complete deployment documentation and guides
- [ ] API documentation for all public interfaces
- [ ] Performance tuning and optimization guides
- [ ] Security configuration and compliance documentation
- [ ] Operations runbook for production deployment
- [ ] Migration guide from Phase 4 to Phase 5

## Technical Specifications

### CLI Command Structure Extension
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "shadowcat")]
#[command(about = "Shadowcat MCP Proxy - Forward and Reverse Proxy with Authentication")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // Existing Phase 4 commands (unchanged)
    Forward(ForwardArgs),
    Tape(TapeArgs),
    Intercept(InterceptArgs),
    
    // NEW: Reverse proxy commands
    Reverse(ReverseArgs),
    Auth(AuthArgs),
    Config(ConfigArgs),
    Monitor(MonitorArgs),
    Debug(DebugArgs),
}

#[derive(Parser)]
pub struct ReverseArgs {
    #[command(subcommand)]
    pub action: ReverseAction,
}

#[derive(Subcommand)]
pub enum ReverseAction {
    /// Start the reverse proxy server
    Start {
        /// Configuration file path
        #[arg(short, long, default_value = "shadowcat-reverse.toml")]
        config: String,
        
        /// Bind address for the reverse proxy
        #[arg(short, long, default_value = "0.0.0.0:8000")]
        bind: String,
        
        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,
    },
    
    /// Stop the reverse proxy server
    Stop {
        /// PID file path
        #[arg(short, long, default_value = "/var/run/shadowcat-reverse.pid")]
        pid_file: String,
    },
    
    /// Reload configuration without stopping
    Reload {
        /// PID file path
        #[arg(short, long, default_value = "/var/run/shadowcat-reverse.pid")]
        pid_file: String,
    },
    
    /// Show reverse proxy status
    Status {
        /// Detailed status information
        #[arg(short, long)]
        detailed: bool,
    },
}
```

### Authentication Management Commands
```rust
#[derive(Subcommand)]
pub enum AuthAction {
    /// Configure OAuth 2.1 settings
    Setup {
        /// OAuth provider configuration
        #[arg(short, long)]
        provider: String,
        
        /// Client ID
        #[arg(long)]
        client_id: String,
        
        /// Client secret
        #[arg(long)]
        client_secret: String,
        
        /// Authorization URL
        #[arg(long)]
        auth_url: String,
        
        /// Token URL
        #[arg(long)]
        token_url: String,
        
        /// JWKS URL
        #[arg(long)]
        jwks_url: String,
    },
    
    /// Test authentication configuration
    Test {
        /// Test token to validate
        #[arg(short, long)]
        token: Option<String>,
        
        /// Generate test token
        #[arg(short, long)]
        generate: bool,
    },
    
    /// Show authentication metrics
    Metrics {
        /// Time window for metrics (minutes)
        #[arg(short, long, default_value = "60")]
        window: u32,
        
        /// Export format (json, table, prometheus)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Manage authentication policies
    Policy {
        #[command(subcommand)]
        action: AuthPolicyAction,
    },
}

#[derive(Subcommand)]
pub enum AuthPolicyAction {
    /// List authentication policies
    List {
        /// Filter by policy name pattern
        #[arg(short, long)]
        filter: Option<String>,
    },
    
    /// Add authentication policy
    Add {
        /// Policy file path
        #[arg(short, long)]
        file: String,
        
        /// Enable immediately
        #[arg(short, long)]
        enable: bool,
    },
    
    /// Remove authentication policy
    Remove {
        /// Policy ID
        policy_id: String,
        
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Test policy against request
    Test {
        /// Policy ID
        policy_id: String,
        
        /// Test request file
        #[arg(short, long)]
        request_file: String,
    },
}
```

### Configuration Management Commands
```rust
#[derive(Subcommand)]
pub enum ConfigAction {
    /// Validate configuration file
    Validate {
        /// Configuration file path
        #[arg(short, long, default_value = "shadowcat-reverse.toml")]
        config: String,
        
        /// Check connectivity to external services
        #[arg(short, long)]
        check_external: bool,
    },
    
    /// Generate default configuration
    Generate {
        /// Output file path
        #[arg(short, long, default_value = "shadowcat-reverse.toml")]
        output: String,
        
        /// Configuration template (basic, production, development)
        #[arg(short, long, default_value = "basic")]
        template: String,
    },
    
    /// Show current configuration
    Show {
        /// Show sensitive values (masked by default)
        #[arg(short, long)]
        show_secrets: bool,
        
        /// Output format (toml, json, yaml)
        #[arg(short, long, default_value = "toml")]
        format: String,
    },
    
    /// Update configuration value
    Set {
        /// Configuration key path (e.g., "auth.oauth2.client_id")
        key: String,
        
        /// New value
        value: String,
        
        /// Apply without confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Get configuration value
    Get {
        /// Configuration key path
        key: String,
    },
    
    /// Compare configurations
    Diff {
        /// First configuration file
        config1: String,
        
        /// Second configuration file
        config2: String,
        
        /// Show only differences
        #[arg(short, long)]
        changes_only: bool,
    },
}
```

### Monitoring and Metrics Commands
```rust
#[derive(Subcommand)]
pub enum MonitorAction {
    /// Show real-time metrics dashboard
    Dashboard {
        /// Refresh interval in seconds
        #[arg(short, long, default_value = "5")]
        refresh: u32,
        
        /// Metrics to display (latency, throughput, errors, connections)
        #[arg(short, long)]
        metrics: Vec<String>,
    },
    
    /// Export metrics in various formats
    Export {
        /// Output format (prometheus, json, csv)
        #[arg(short, long, default_value = "prometheus")]
        format: String,
        
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
        
        /// Time range in minutes
        #[arg(short, long, default_value = "60")]
        range: u32,
    },
    
    /// Show connection pool status
    Connections {
        /// Show detailed connection information
        #[arg(short, long)]
        detailed: bool,
        
        /// Filter by upstream ID
        #[arg(short, long)]
        upstream: Option<String>,
    },
    
    /// Show rate limiting statistics
    RateLimit {
        /// Time window in minutes
        #[arg(short, long, default_value = "60")]
        window: u32,
        
        /// Group by (ip, user, endpoint, session)
        #[arg(short, long, default_value = "user")]
        group_by: String,
    },
    
    /// Show circuit breaker status
    CircuitBreaker {
        /// Show historical state changes
        #[arg(short, long)]
        history: bool,
        
        /// Filter by upstream ID
        #[arg(short, long)]
        upstream: Option<String>,
    },
    
    /// Show audit log summary
    Audit {
        /// Number of recent events to show
        #[arg(short, long, default_value = "100")]
        count: usize,
        
        /// Filter by event type
        #[arg(short, long)]
        event_type: Option<String>,
        
        /// Filter by user
        #[arg(short, long)]
        user: Option<String>,
    },
}
```

### Debug and Troubleshooting Commands
```rust
#[derive(Subcommand)]
pub enum DebugAction {
    /// Health check for all components
    Health {
        /// Include upstream server health
        #[arg(short, long)]
        include_upstreams: bool,
        
        /// Detailed health information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Trace a request through the system
    Trace {
        /// Request ID or session ID
        request_id: String,
        
        /// Follow related requests
        #[arg(short, long)]
        follow: bool,
        
        /// Output format (json, table)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Test connectivity to upstream servers
    Upstream {
        /// Upstream ID to test
        upstream_id: Option<String>,
        
        /// Test all upstreams
        #[arg(short, long)]
        all: bool,
        
        /// Number of test requests per upstream
        #[arg(short, long, default_value = "3")]
        count: u32,
    },
    
    /// Validate JWT token
    Jwt {
        /// JWT token to validate
        token: String,
        
        /// Show token claims
        #[arg(short, long)]
        show_claims: bool,
        
        /// Verify against JWKS
        #[arg(short, long)]
        verify: bool,
    },
    
    /// Test policy evaluation
    Policy {
        /// Policy ID to test
        policy_id: String,
        
        /// Test request JSON file
        #[arg(short, long)]
        request: String,
        
        /// Show evaluation steps
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Show system information
    System {
        /// Include performance metrics
        #[arg(short, long)]
        performance: bool,
        
        /// Include resource usage
        #[arg(short, long)]
        resources: bool,
    },
}
```

### CLI Implementation
```rust
// Implementation of core CLI commands
impl CliHandler {
    pub async fn handle_reverse_start(&self, args: &ReverseStartArgs) -> Result<(), CliError> {
        println!("🚀 Starting Shadowcat Reverse Proxy...");
        
        // Load and validate configuration
        let config = self.load_reverse_proxy_config(&args.config).await?;
        println!("✅ Configuration loaded from {}", args.config);
        
        // Validate external dependencies
        self.validate_external_dependencies(&config).await?;
        println!("✅ External dependencies validated");
        
        // Start the reverse proxy server
        let server = ReverseProxyServer::new(config).await?;
        
        // Setup signal handlers for graceful shutdown
        self.setup_signal_handlers().await?;
        
        // Write PID file
        self.write_pid_file(&args.pid_file).await?;
        
        println!("🌟 Reverse proxy server started successfully");
        println!("   • Listening on: {}", args.bind);
        println!("   • PID file: {}", args.pid_file);
        println!("   • Configuration: {}", args.config);
        
        // Start server and block until shutdown signal
        server.run().await?;
        
        // Cleanup
        self.cleanup_pid_file(&args.pid_file).await?;
        println!("🛑 Reverse proxy server stopped");
        
        Ok(())
    }

    pub async fn handle_auth_metrics(&self, args: &AuthMetricsArgs) -> Result<(), CliError> {
        let metrics = self.collect_auth_metrics(args.window).await?;
        
        match args.format.as_str() {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&metrics)?);
            }
            "prometheus" => {
                self.export_prometheus_metrics(&metrics).await?;
            }
            "table" | _ => {
                self.display_auth_metrics_table(&metrics).await?;
            }
        }
        
        Ok(())
    }

    async fn display_auth_metrics_table(&self, metrics: &AuthMetrics) -> Result<(), CliError> {
        use prettytable::{Table, Row, Cell};
        
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Metric"),
            Cell::new("Value"),
            Cell::new("Trend"),
        ]));
        
        table.add_row(Row::new(vec![
            Cell::new("Successful Authentications"),
            Cell::new(&metrics.successful_auths.to_string()),
            Cell::new(&self.format_trend(metrics.auth_trend)),
        ]));
        
        table.add_row(Row::new(vec![
            Cell::new("Failed Authentications"),
            Cell::new(&metrics.failed_auths.to_string()),
            Cell::new(&self.format_trend(metrics.failure_trend)),
        ]));
        
        table.add_row(Row::new(vec![
            Cell::new("Average Auth Time"),
            Cell::new(&format!("{:.2}ms", metrics.avg_auth_time.as_millis())),
            Cell::new(&self.format_trend(metrics.latency_trend)),
        ]));
        
        table.add_row(Row::new(vec![
            Cell::new("Token Cache Hit Rate"),
            Cell::new(&format!("{:.1}%", metrics.cache_hit_rate * 100.0)),
            Cell::new(&self.format_trend(metrics.cache_trend)),
        ]));
        
        table.printstd();
        Ok(())
    }

    pub async fn handle_monitor_dashboard(&self, args: &DashboardArgs) -> Result<(), CliError> {
        use crossterm::{
            event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
            execute,
            terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        };
        use tui::{
            backend::CrosstermBackend,
            layout::{Constraint, Direction, Layout},
            style::{Color, Modifier, Style},
            symbols,
            text::Span,
            widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, List, ListItem, Paragraph},
            Frame, Terminal,
        };

        // Initialize terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create dashboard loop
        let mut should_quit = false;
        let mut last_update = Instant::now();

        while !should_quit {
            // Check for quit event
            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        should_quit = true;
                    }
                }
            }

            // Update dashboard at specified interval
            if last_update.elapsed() >= Duration::from_secs(args.refresh as u64) {
                let metrics = self.collect_dashboard_metrics().await?;
                
                terminal.draw(|f| {
                    self.render_dashboard(f, &metrics, &args.metrics);
                })?;
                
                last_update = Instant::now();
            }
        }

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        Ok(())
    }

    pub async fn handle_debug_health(&self, args: &HealthArgs) -> Result<(), CliError> {
        println!("🔍 Performing health check...\n");
        
        let mut all_healthy = true;
        
        // Check core components
        let core_health = self.check_core_component_health().await?;
        self.display_health_status("Core Components", &core_health);
        all_healthy &= core_health.is_healthy();
        
        // Check authentication system
        let auth_health = self.check_auth_system_health().await?;
        self.display_health_status("Authentication System", &auth_health);
        all_healthy &= auth_health.is_healthy();
        
        // Check policy engine
        let policy_health = self.check_policy_engine_health().await?;
        self.display_health_status("Policy Engine", &policy_health);
        all_healthy &= policy_health.is_healthy();
        
        // Check upstream servers if requested
        if args.include_upstreams {
            let upstream_health = self.check_upstream_health().await?;
            self.display_health_status("Upstream Servers", &upstream_health);
            all_healthy &= upstream_health.is_healthy();
        }
        
        // Overall status
        println!("\n{}", "=".repeat(50));
        if all_healthy {
            println!("✅ Overall Status: HEALTHY");
        } else {
            println!("❌ Overall Status: UNHEALTHY");
            std::process::exit(1);
        }
        
        Ok(())
    }
}
```

### Configuration Templates
```rust
// Configuration template generation
impl ConfigGenerator {
    pub fn generate_basic_config() -> ReverseProxyConfig {
        ReverseProxyConfig {
            server: ServerConfig {
                bind_address: "0.0.0.0:8000".parse().unwrap(),
                tls: None,
                cors: CorsConfig::default(),
            },
            
            oauth2: OAuth2Config {
                client_id: "your-client-id".to_string(),
                client_secret: "your-client-secret".to_string(),
                auth_url: "https://auth.example.com/oauth2/authorize".to_string(),
                token_url: "https://auth.example.com/oauth2/token".to_string(),
                redirect_url: "http://localhost:8000/auth/callback".to_string(),
                default_scopes: vec!["openid".to_string(), "profile".to_string()],
                token_encryption_key: generate_encryption_key(),
            },
            
            jwt_validation: JwtValidationConfig {
                jwks_uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
                allowed_audiences: vec!["shadowcat".to_string()],
                allowed_issuers: vec!["https://auth.example.com".to_string()],
                cache_ttl: Duration::from_secs(300),
                clock_skew_tolerance: Duration::from_secs(30),
                supported_algorithms: vec!["RS256".to_string(), "ES256".to_string()],
            },
            
            upstreams: vec![
                UpstreamConfig {
                    id: "primary-mcp-server".to_string(),
                    base_url: "http://localhost:9000".to_string(),
                    weight: 100,
                    tls: None,
                    auth: None,
                    health_check_path: Some("/health".to_string()),
                }
            ],
            
            rate_limits: RateLimitConfig {
                global: TierConfig {
                    requests_per_minute: 10000,
                    burst_size: 1000,
                    enabled: true,
                },
                per_user: TierConfig {
                    requests_per_minute: 1000,
                    burst_size: 100,
                    enabled: true,
                },
                per_ip: TierConfig {
                    requests_per_minute: 500,
                    burst_size: 50,
                    enabled: true,
                },
                per_endpoint: TierConfig {
                    requests_per_minute: 2000,
                    burst_size: 200,
                    enabled: true,
                },
                per_session: TierConfig {
                    requests_per_minute: 200,
                    burst_size: 20,
                    enabled: true,
                },
            },
            
            audit: AuditConfig {
                enabled: true,
                log_successful_auth: true,
                log_failed_auth: true,
                log_successful_policy_decisions: false,
                store_rate_limit_violations: true,
                store_policy_decisions: true,
                audit_store: None,
            },
            
            metrics: MetricsConfig {
                enabled: true,
                prometheus_endpoint: "/metrics".to_string(),
                collection_interval: Duration::from_secs(30),
                retention_period: Duration::from_hours(24),
            },
        }
    }

    pub fn generate_production_config() -> ReverseProxyConfig {
        let mut config = Self::generate_basic_config();
        
        // Production-specific overrides
        config.server.tls = Some(TlsConfig {
            cert_file: "/etc/shadowcat/tls/cert.pem".to_string(),
            key_file: "/etc/shadowcat/tls/key.pem".to_string(),
            ca_file: None,
        });
        
        config.rate_limits.global.requests_per_minute = 50000;
        config.rate_limits.global.burst_size = 5000;
        
        config.audit.audit_store = Some(AuditStoreConfig {
            store_type: "database".to_string(),
            connection_string: "postgresql://user:pass@localhost/shadowcat_audit".to_string(),
            retention_days: 90,
        });
        
        config
    }
}
```

## Implementation Steps

### Step 1: CLI Command Extension
- Extend existing CLI with reverse proxy commands
- Implement authentication management commands
- Add configuration management utilities
- Create monitoring and metrics commands

### Step 2: Configuration Management
- Create configuration templates for different deployment scenarios
- Implement configuration validation and generation
- Add configuration comparison and migration utilities
- Create secure configuration handling for secrets

### Step 3: Monitoring and Debugging
- Implement real-time dashboard functionality
- Create comprehensive health check utilities
- Add debugging and troubleshooting commands
- Implement request tracing and analysis tools

### Step 4: Documentation Creation
- Write comprehensive deployment guides
- Create API documentation for all components
- Develop troubleshooting and operations runbooks
- Create performance tuning guides

### Step 5: Testing and Validation
- Test all CLI commands with various scenarios
- Validate documentation accuracy and completeness
- Create example configurations and use cases
- Perform user acceptance testing of CLI interface

## Dependencies

### Blocked By
- Task 009: Performance Testing and Optimization (performance docs needed)

### Blocks
- None (final task)

### Integrates With
- All Phase 5 components for CLI management
- Existing Phase 4 CLI infrastructure

## Testing Requirements

### CLI Testing
- [ ] All command-line arguments parsed correctly
- [ ] Configuration validation working properly
- [ ] Error handling and user feedback appropriate
- [ ] Help text and documentation accurate
- [ ] Interactive features (dashboard) functional

### Documentation Testing
- [ ] All deployment scenarios documented and tested
- [ ] Configuration examples validated
- [ ] Troubleshooting guides accurate
- [ ] Performance tuning recommendations effective
- [ ] API documentation complete and accurate

### User Experience Testing
- [ ] CLI commands intuitive and consistent
- [ ] Error messages helpful and actionable
- [ ] Configuration process straightforward
- [ ] Monitoring tools provide valuable insights
- [ ] Documentation clear and comprehensive

## Documentation Requirements

### Deployment Documentation
- [ ] Installation and setup guide
- [ ] Configuration reference documentation
- [ ] Security configuration guide
- [ ] Performance tuning recommendations
- [ ] Troubleshooting and operations runbook

### API Documentation
- [ ] Complete API reference for all components
- [ ] Authentication and authorization documentation
- [ ] Policy configuration guide
- [ ] Metrics and monitoring documentation
- [ ] Integration examples and patterns

### User Guides
- [ ] Getting started guide
- [ ] Migration guide from Phase 4
- [ ] Best practices and recommendations
- [ ] Common use cases and examples
- [ ] FAQ and troubleshooting

## Risk Assessment

**Low Risk**: CLI extensions and documentation, well-defined requirements.

**Mitigation Strategies**:
- Comprehensive testing of all CLI functionality
- User feedback collection and iteration
- Documentation review and validation
- Example configurations and use cases

## Completion Checklist

- [ ] CLI extended with all reverse proxy management commands
- [ ] Configuration management commands functional
- [ ] Monitoring and metrics commands working
- [ ] Debug and troubleshooting utilities operational
- [ ] Real-time dashboard implemented
- [ ] Configuration templates created for all scenarios
- [ ] Complete deployment documentation written
- [ ] API documentation generated and validated
- [ ] Performance tuning guide created
- [ ] Security configuration guide complete
- [ ] Operations runbook finalized
- [ ] Migration guide from Phase 4 written
- [ ] All CLI commands tested and validated
- [ ] Documentation accuracy verified
- [ ] User experience validated
- [ ] Example configurations provided

## Files Modified/Created

### New Files
- `src/cli/reverse.rs`: Reverse proxy CLI commands
- `src/cli/auth.rs`: Authentication management commands
- `src/cli/config.rs`: Configuration management commands
- `src/cli/monitor.rs`: Monitoring and metrics commands
- `src/cli/debug.rs`: Debug and troubleshooting commands
- `src/config/templates.rs`: Configuration templates
- `docs/deployment-guide.md`: Comprehensive deployment guide
- `docs/api-reference.md`: Complete API documentation
- `docs/performance-tuning.md`: Performance optimization guide
- `docs/security-guide.md`: Security configuration guide
- `docs/operations-runbook.md`: Operations and troubleshooting guide
- `docs/migration-guide.md`: Phase 4 to Phase 5 migration guide
- `examples/`: Configuration examples and use cases

### Modified Files
- `src/main.rs`: Integrate new CLI commands
- `src/cli/mod.rs`: Export new CLI modules
- `Cargo.toml`: Add CLI dependencies (clap, prettytable, tui, etc.)
- `README.md`: Update with Phase 5 capabilities
- CI/CD configuration for documentation building

## Final Deliverables

Upon completion of this task, Phase 5 (Reverse Proxy & Authentication) will be fully implemented with:

1. **Complete CLI Management**: Full command-line interface for all reverse proxy operations
2. **Production-Ready Configuration**: Templates and management for all deployment scenarios  
3. **Comprehensive Monitoring**: Real-time dashboards, metrics export, and alerting
4. **Debug and Troubleshooting**: Complete toolkit for diagnosing and resolving issues
5. **Complete Documentation**: Deployment guides, API docs, operations runbooks
6. **Migration Support**: Clear path from Phase 4 to Phase 5 with migration utilities

The system will be ready for production deployment with enterprise-grade reverse proxy capabilities including OAuth 2.1 authentication, policy-based authorization, multi-tier rate limiting, connection pooling with circuit breakers, comprehensive audit logging, and high-performance operation meeting all specified targets.