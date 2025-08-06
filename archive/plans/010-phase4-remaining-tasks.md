# Phase 4 Remaining Tasks - Implementation Plan

**Project:** Shadowcat Phase 4 Completion  
**Timeline:** Week 7-8 (August 5-18, 2025)  
**Status:** Planning

---

## Overview

Phase 4 core infrastructure is complete with the interceptor engine and rule engine fully implemented. The remaining tasks focus on completing the user-facing features and integration points to make the interception system fully functional for end users.

**Completed (August 4, 2025):**
- ✅ InterceptorChain with async hooks and priority handling
- ✅ Comprehensive rule engine with JSON-based matching
- ✅ ForwardProxy integration with all action types
- ✅ Action framework with conditional execution
- ✅ Comprehensive test coverage (17 new tests)

**Remaining Tasks:**
- 🔴 Rule-Based Interceptor Integration (bridges RuleEngine → InterceptorChain)
- 🔴 CLI Intercept Management (user interface for interception)
- 🔴 Advanced Message Actions (enhanced modification and mocking)
- 🔴 Persistent Rule Storage (file-based rule management)

---

## Task 1: Rule-Based Interceptor Integration

**Priority:** 🔴 HIGH  
**Estimated Effort:** 2 days  
**File:** `src/interceptor/rules_interceptor.rs`

### Problem Statement
Currently, the RuleEngine exists separately from the InterceptorChain. Users cannot easily apply rule-based interception without writing custom interceptors. We need a bridge that automatically converts rules into interceptor behavior.

### Implementation Plan

#### 1.1 RuleBasedInterceptor Structure
```rust
pub struct RuleBasedInterceptor {
    rule_engine: Arc<RwLock<RuleEngine>>,
    config: RuleInterceptorConfig,
    metrics: Arc<RwLock<RuleInterceptorMetrics>>,
}

pub struct RuleInterceptorConfig {
    pub auto_reload: bool,
    pub rule_files: Vec<PathBuf>,
    pub max_rules: Option<usize>,
    pub evaluation_timeout: Duration,
}
```

#### 1.2 Core Integration
- Implement `Interceptor` trait for `RuleBasedInterceptor`
- `intercept()` method calls `rule_engine.evaluate()` and converts actions
- Handle multiple matching rules with proper priority ordering
- Support rule result aggregation (e.g., multiple Continue actions)

#### 1.3 Dynamic Rule Management
```rust
impl RuleBasedInterceptor {
    pub async fn load_rules_from_file(&self, path: &Path) -> InterceptResult<usize>
    pub async fn reload_rules(&self) -> InterceptResult<()>
    pub async fn add_rule(&self, rule: Rule) -> InterceptResult<()>
    pub async fn remove_rule(&self, rule_id: &str) -> InterceptResult<bool>
    pub async fn list_rules(&self) -> Vec<Rule>
    pub async fn get_rule_metrics(&self) -> RuleInterceptorMetrics
}
```

#### 1.4 Hot-Reloading Support
- File system watching for rule file changes
- Atomic rule reloading without dropping active interceptions
- Validation before applying new rules
- Rollback capability on invalid rule sets

### Success Criteria
- [ ] RuleBasedInterceptor implements Interceptor trait correctly
- [ ] Rule evaluation produces correct InterceptAction results
- [ ] Dynamic rule loading works without interrupting active sessions
- [ ] File watching triggers automatic rule reloads
- [ ] Performance impact < 500μs for typical rule evaluations
- [ ] Integration tests with ForwardProxy work end-to-end

---

## Task 2: CLI Intercept Management Foundation

**Priority:** 🔴 HIGH  
**Estimated Effort:** 2 days  
**File:** `src/cli/intercept.rs`

### Problem Statement
Users need a command-line interface to manage interception rules, start/stop interception sessions, and debug rule behavior. This is critical for developer experience and system usability.

### Implementation Plan

#### 2.1 Command Structure
```bash
shadowcat intercept start [OPTIONS] -- <COMMAND>
shadowcat intercept rules list [--format=table|json] [--enabled-only]
shadowcat intercept rules add <RULE_FILE>
shadowcat intercept rules remove <RULE_ID>
shadowcat intercept rules enable <RULE_ID>
shadowcat intercept rules disable <RULE_ID>
shadowcat intercept rules validate <RULE_FILE>
shadowcat intercept status
shadowcat intercept stop
```

#### 2.2 Core CLI Structure
```rust
#[derive(Parser)]
pub struct InterceptCli {
    #[command(subcommand)]
    pub command: InterceptCommand,
}

#[derive(Subcommand)]
pub enum InterceptCommand {
    Start(StartCommand),
    Rules(RulesCommand),
    Status,
    Stop,
}

#[derive(Parser)]
pub struct StartCommand {
    /// Rule files to load
    #[arg(short, long)]
    pub rules: Vec<PathBuf>,
    
    /// Enable interactive mode
    #[arg(short, long)]
    pub interactive: bool,
    
    /// Command to proxy
    pub command: Vec<String>,
}
```

#### 2.3 Rule Management Commands
- **List Rules**: Display active rules with status, priority, match count
- **Add Rules**: Load rules from JSON/YAML files with validation
- **Remove Rules**: Remove rules by ID with confirmation
- **Enable/Disable**: Toggle rule status without removing
- **Validate**: Check rule syntax and logic without loading
- **Show Rule**: Display detailed rule information and metrics

#### 2.4 Session Management
- **Start**: Launch proxy with specified rules and target command
- **Status**: Show active interception sessions and statistics
- **Stop**: Gracefully shutdown active interception sessions
- **Interactive Mode**: Real-time rule debugging and modification

### Success Criteria
- [ ] All command structures implemented and functional
- [ ] Rule file validation with clear error messages
- [ ] Rich terminal output with tables and progress indicators
- [ ] Integration with RuleBasedInterceptor for rule management
- [ ] Basic interactive session management
- [ ] Error handling with helpful user guidance

---

## Task 3: Advanced Message Actions

**Priority:** 🟡 MEDIUM  
**Estimated Effort:** 1 day  
**File:** `src/interceptor/actions.rs`

### Problem Statement
Current action system supports basic operations but lacks advanced message transformation, sophisticated mocking, and complex delay patterns needed for realistic testing scenarios.

### Implementation Plan

#### 3.1 Enhanced Message Modification
```rust
pub struct MessageTransformer {
    pub modifications: Vec<MessageModification>,
}

pub enum MessageModification {
    SetField { path: String, value: Value },
    RemoveField { path: String },
    AddField { path: String, value: Value },
    Transform { path: String, transformer: Box<dyn ValueTransformer> },
}

pub trait ValueTransformer {
    fn transform(&self, value: &Value) -> TransformResult<Value>;
}
```

#### 3.2 Template-Based Mock Responses
```rust
pub struct MockResponseTemplate {
    pub response_type: ResponseType,
    pub template: String, // Handlebars template
    pub variables: BTreeMap<String, Value>,
}

pub enum ResponseType {
    Success { result_template: String },
    Error { error_code: i32, message_template: String },
    Custom { template: String },
}
```

#### 3.3 Sophisticated Delay Patterns
```rust
pub enum DelayPattern {
    Fixed(Duration),
    Random { min: Duration, max: Duration },
    ExponentialBackoff { base: Duration, max_attempts: u32 },
    Jitter { base: Duration, jitter_percent: f64 },
}
```

#### 3.4 Fault Injection Scenarios
- Network timeouts and connection failures
- Malformed JSON responses
- Partial message corruption
- Rate limiting simulation
- Authentication failures

### Success Criteria
- [ ] JSONPath-based message modification working correctly
- [ ] Template system for mock responses with variable substitution
- [ ] Multiple delay patterns implemented and tested
- [ ] Fault injection scenarios cover common failure modes
- [ ] Performance impact remains minimal (< 100μs per action)

---

## Task 4: Persistent Rule Storage

**Priority:** 🟡 MEDIUM  
**Estimated Effort:** 2 days  
**File:** `src/interceptor/storage.rs`

### Problem Statement
Rules are currently managed in memory only. Users need persistent storage, versioning, sharing, and backup capabilities for their rule collections.

### Implementation Plan

#### 4.1 File-Based Storage
```rust
pub struct RuleStorage {
    pub storage_path: PathBuf,
    pub format: StorageFormat,
    pub backup_enabled: bool,
}

pub enum StorageFormat {
    Json,
    Yaml,
}
```

#### 4.2 Rule Collection Management
- Save/load rule collections with metadata
- Automatic backup before modifications
- Rule collection validation and migration
- Import/export between different storage formats

#### 4.3 Versioning System
- Rule collection versioning with timestamps
- Rollback to previous versions
- Change tracking and audit logs
- Diff display between versions

#### 4.4 Rule Templates and Libraries
- Built-in rule templates for common scenarios
- User-defined rule templates
- Rule sharing and import from URLs
- Rule library management

### Success Criteria
- [ ] Rule persistence works reliably across restarts
- [ ] Backup and versioning system prevents data loss
- [ ] Rule templates provide good starting points
- [ ] Import/export enables rule sharing
- [ ] Performance suitable for large rule collections (1000+ rules)

---

## Integration and Testing Plan

### Integration Testing
1. **End-to-End Rule Processing**
   - Load rules via CLI → RuleBasedInterceptor → ForwardProxy
   - Verify all action types work correctly in realistic scenarios
   - Test rule priority and chaining behavior

2. **Dynamic Rule Updates**
   - Hot-reload rules during active proxy sessions
   - Verify no message loss or corruption during rule updates
   - Test rollback scenarios for invalid rule updates

3. **CLI Workflow Testing**
   - Complete rule management workflow via CLI
   - Interactive session management and debugging
   - Rule validation and error reporting

### Performance Testing
- Rule evaluation latency under load
- Memory usage with large rule sets
- Hot-reloading performance impact
- CLI command response times

### User Experience Testing
- CLI command discoverability and help
- Error message clarity and actionability
- Rule file format documentation and examples
- Interactive debugging workflow efficiency

---

## Timeline and Milestones

### Week 7 (August 5-11)
**Day 1-2:** Rule-Based Interceptor Integration
- Implement RuleBasedInterceptor with Interceptor trait
- Add dynamic rule loading and basic hot-reloading
- Integration tests with ForwardProxy

**Day 3-4:** CLI Foundation
- Implement command structure and basic rule management
- Add rule file validation and loading
- Basic session management commands

**Day 5:** Advanced Actions Foundation
- Enhanced message modification framework
- Template-based mock response system
- Integration testing

### Week 8 (August 12-18)
**Day 1-2:** Complete CLI Interface
- Interactive session management
- Rich terminal UI and formatting
- Real-time debugging features

**Day 3-4:** Rule Storage System
- Persistent storage with versioning
- Rule templates and libraries
- Import/export functionality

**Day 5:** Integration and Polish
- End-to-end testing across all components
- Performance validation and optimization
- Documentation and examples

---

## Success Metrics

### Functional Completeness
- [ ] All remaining Phase 4 tasks completed
- [ ] End-to-end rule-based interception workflow functional
- [ ] CLI provides complete rule management capabilities
- [ ] Rule storage and sharing system operational

### Performance Requirements
- [ ] Rule evaluation: < 500μs per message
- [ ] CLI commands: < 200ms response time
- [ ] Hot-reloading: < 1s for typical rule sets
- [ ] Memory usage: < 50MB for 1000 rules

### User Experience
- [ ] Clear and helpful CLI interface
- [ ] Comprehensive error messages and validation
- [ ] Good rule file format documentation
- [ ] Working examples for common scenarios

This completes the Phase 4 implementation, delivering a fully functional interception and rule engine system ready for production use.