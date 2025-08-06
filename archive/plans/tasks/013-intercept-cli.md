# Task 013: CLI Intercept Management

**Phase:** 4 - Interception & Rule Engine  
**Priority:** Medium  
**Estimated Effort:** 2 days  
**Assignee:** Development Team  
**Status:** Not Started

---

## Overview

Create a comprehensive command-line interface for managing interception rules, interactive debugging sessions, and real-time MCP message inspection. This interface provides developers with powerful tools for debugging MCP applications, testing rule effectiveness, and conducting interactive debugging sessions.

## Objectives

- Implement `shadowcat intercept` command group with full functionality
- Create interactive debugging interface with real-time message display
- Add rule management commands with validation and testing
- Provide session attachment and detachment capabilities
- Enable rule testing against recorded tapes
- Support export/import of debugging sessions and rules

## Technical Requirements

### Core Commands

#### 1. Interactive Interception
```bash
# Start interactive interception with new session
shadowcat intercept start [OPTIONS] -- <forward-command>

# Attach to existing running session
shadowcat intercept attach <session-id> [OPTIONS]

# Detach from current session (keep session running)
shadowcat intercept detach

# Stop interception and terminate session
shadowcat intercept stop [<session-id>]

Options:
  --rules <file>              Load rules from file
  --rules-dir <directory>     Load all rules from directory
  --interactive               Enable interactive mode (default)
  --headless                  Run without interactive interface
  --output <format>           Output format: table, json, yaml
  --filter <pattern>          Filter messages by method/pattern
  --save-session <file>       Save session to file on exit
  --auto-resume               Auto-resume paused messages after timeout
  --theme <theme>             UI theme: dark, light, auto
```

#### 2. Rule Management
```bash
# List all rules
shadowcat intercept rules list [OPTIONS]

# Add rule from file
shadowcat intercept rules add <rule-file> [OPTIONS]

# Remove rule by ID
shadowcat intercept rules remove <rule-id>

# Enable/disable rules
shadowcat intercept rules enable <rule-id>
shadowcat intercept rules disable <rule-id>

# Edit rule interactively
shadowcat intercept rules edit <rule-id>

# Test rule against tape
shadowcat intercept rules test <rule-id> <tape-id> [OPTIONS]

# Validate rule syntax
shadowcat intercept rules validate <rule-file>

# Export rules to file
shadowcat intercept rules export [<rule-ids>...] --output <file>

# Import rules from file
shadowcat intercept rules import <file> [OPTIONS]

Options:
  --enabled                   Show only enabled rules
  --format <format>          Output format: table, json, yaml
  --verbose, -v              Show detailed rule information
  --dry-run                  Validate without applying changes
  --force                    Force operation without confirmation
  --tags <tags>              Filter by rule tags
  --priority <min>-<max>     Filter by priority range
```

#### 3. Session Management
```bash
# List active interception sessions
shadowcat intercept sessions list [OPTIONS]

# Show session details
shadowcat intercept sessions show <session-id> [OPTIONS]

# Export session data
shadowcat intercept sessions export <session-id> <output-file> [OPTIONS]

# Replay session with interception
shadowcat intercept sessions replay <session-id> [OPTIONS]

# Clean up terminated sessions
shadowcat intercept sessions cleanup

Options:
  --include-messages         Include message content in export
  --include-rules           Include rule definitions in export
  --format <format>         Output format: json, yaml, tape
  --since <duration>        Show sessions from last duration (e.g., 1h, 30m)
  --status <status>         Filter by session status
```

#### 4. Real-time Debugging
```bash
# Monitor messages in real-time
shadowcat intercept monitor [<session-id>] [OPTIONS]

# Step through messages interactively
shadowcat intercept step <session-id> [OPTIONS]

# Resume all paused messages
shadowcat intercept resume <session-id> [all|<message-id>]

# Pause processing for specific methods
shadowcat intercept pause <session-id> --method <method-pattern>

Options:
  --follow, -f              Follow new messages (like tail -f)
  --filter <pattern>        Filter messages by pattern
  --highlight <pattern>     Highlight matching content
  --show-timing            Show message timing information
  --show-rules             Show which rules matched
  --max-lines <n>          Limit output lines per message
```

### Interactive Interface Components

#### 1. Main Dashboard
```rust
pub struct InterceptDashboard {
    session_manager: Arc<SessionManager>,
    rule_engine: Arc<RuleEngine>,
    interceptor_chain: Arc<InterceptorChain>,
    ui_state: DashboardState,
    config: DashboardConfig,
}

#[derive(Debug, Clone)]
pub struct DashboardState {
    pub current_session: Option<SessionId>,
    pub active_rules: Vec<String>,
    pub paused_messages: Vec<PausedMessage>,
    pub message_filter: MessageFilter,
    pub display_mode: DisplayMode,
    pub theme: Theme,
}

#[derive(Debug, Clone)]
pub enum DisplayMode {
    List,           // Message list view
    Detail,         // Single message detail view
    Split,          // Split view with list and detail
    Rules,          // Rules management view
    Sessions,       // Session management view
}

impl InterceptDashboard {
    pub async fn run(&mut self) -> CliResult<()>;
    pub async fn handle_key_input(&mut self, key: KeyEvent) -> CliResult<()>;
    pub async fn update_display(&mut self) -> CliResult<()>;
    pub async fn show_message_detail(&mut self, message_id: &str) -> CliResult<()>;
    pub async fn toggle_rule(&mut self, rule_id: &str) -> CliResult<()>;
    pub async fn resume_message(&mut self, message_id: &str) -> CliResult<()>;
}
```

#### 2. Message Display
```rust
pub struct MessageDisplay {
    formatter: MessageFormatter,
    highlighter: SyntaxHighlighter,
    theme: Theme,
}

impl MessageDisplay {
    pub fn format_message(&self, message: &TransportMessage, context: &MessageContext) -> String {
        match message {
            TransportMessage::Request { id, method, params } => {
                self.format_request(id, method, params, context)
            }
            TransportMessage::Response { id, result, error } => {
                self.format_response(id, result, error, context)
            }
            TransportMessage::Notification { method, params } => {
                self.format_notification(method, params, context)
            }
        }
    }
    
    fn format_request(&self, id: &str, method: &str, params: &Value, context: &MessageContext) -> String {
        let timestamp = format_timestamp(context.timestamp);
        let direction = format_direction(context.direction);
        let session = context.session_id.to_string()[..8].to_string();
        
        format!(
            "┌─ {} {} [{}] {} ─\n\
             │ Method: {}\n\
             │ ID: {}\n\
             │ Session: {}…\n\
             │ Transport: {}\n\
             ├─ Parameters ─\n\
             {}\n\
             └─{}─",
            timestamp,
            direction,
            context.transport_type,
            if context.intercepted { "🛑" } else { "  " },
            self.highlight_method(method),
            self.highlight_id(id),
            session,
            context.transport_type,
            self.format_json_with_syntax_highlighting(params),
            "─".repeat(50)
        )
    }
    
    pub fn format_json_with_syntax_highlighting(&self, value: &Value) -> String {
        let json_str = serde_json::to_string_pretty(value).unwrap_or_default();
        self.highlighter.highlight_json(&json_str, &self.theme)
    }
}

#[derive(Debug, Clone)]
pub struct MessageContext {
    pub timestamp: u64,
    pub direction: Direction,
    pub session_id: SessionId,
    pub transport_type: TransportType,
    pub intercepted: bool,
    pub matched_rules: Vec<String>,
    pub execution_time_us: Option<u64>,
}
```

#### 3. Rule Editor
```rust
pub struct RuleEditor {
    editor_state: EditorState,
    validator: RuleValidator,
    template_engine: TemplateEngine,
}

impl RuleEditor {
    pub async fn edit_rule(&mut self, rule_id: Option<&str>) -> CliResult<Rule> {
        // Load existing rule or create new one
        let mut rule = if let Some(id) = rule_id {
            self.load_rule(id).await?
        } else {
            self.create_rule_template().await?
        };
        
        loop {
            // Display current rule
            self.display_rule(&rule)?;
            
            // Show menu options
            self.show_edit_menu()?;
            
            // Handle user input
            match self.get_user_choice().await? {
                EditChoice::EditConditions => {
                    rule.conditions = self.edit_conditions(&rule.conditions).await?;
                }
                EditChoice::EditActions => {
                    rule.actions = self.edit_actions(&rule.actions).await?;
                }
                EditChoice::EditMetadata => {
                    self.edit_metadata(&mut rule).await?;
                }
                EditChoice::TestRule => {
                    self.test_rule_interactive(&rule).await?;
                }
                EditChoice::Save => {
                    self.validate_and_save(&rule).await?;
                    break;
                }
                EditChoice::Cancel => {
                    break;
                }
            }
        }
        
        Ok(rule)
    }
    
    async fn edit_conditions(&mut self, current: &RuleCondition) -> CliResult<RuleCondition> {
        // Interactive condition builder
        // Show current conditions in tree format
        // Allow adding/removing/modifying conditions
        // Provide syntax validation and suggestions
        todo!()
    }
    
    async fn test_rule_interactive(&self, rule: &Rule) -> CliResult<()> {
        println!("Testing rule: {}", rule.name);
        
        // Let user choose test data source
        let source = self.choose_test_source().await?;
        
        match source {
            TestSource::Tape(tape_id) => {
                self.test_rule_against_tape(rule, &tape_id).await?;
            }
            TestSource::LiveSession(session_id) => {
                self.test_rule_against_session(rule, &session_id).await?;
            }
            TestSource::ManualInput => {
                self.test_rule_with_manual_input(rule).await?;
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum EditChoice {
    EditConditions,
    EditActions,
    EditMetadata,
    TestRule,
    Save,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum TestSource {
    Tape(String),
    LiveSession(SessionId),
    ManualInput,
}
```

#### 4. Keyboard Shortcuts
```rust
pub struct KeyboardHandler {
    shortcuts: HashMap<KeyEvent, Command>,
    mode: InteractionMode,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        let mut shortcuts = HashMap::new();
        
        // Global shortcuts
        shortcuts.insert(key('q'), Command::Quit);
        shortcuts.insert(key('h'), Command::Help);
        shortcuts.insert(key('r'), Command::Refresh);
        shortcuts.insert(ctrl('c'), Command::Cancel);
        
        // Navigation
        shortcuts.insert(key('j'), Command::MoveDown);
        shortcuts.insert(key('k'), Command::MoveUp);
        shortcuts.insert(key('g'), Command::GoToTop);
        shortcuts.insert(key('G'), Command::GoToBottom);
        shortcuts.insert(key('/'), Command::Search);
        shortcuts.insert(key('n'), Command::SearchNext);
        shortcuts.insert(key('N'), Command::SearchPrevious);
        
        // Message actions
        shortcuts.insert(key(' '), Command::ToggleSelect);
        shortcuts.insert(key('Enter'), Command::ViewDetail);
        shortcuts.insert(key('p'), Command::PauseMessage);
        shortcuts.insert(key('c'), Command::ContinueMessage);
        shortcuts.insert(key('m'), Command::ModifyMessage);
        shortcuts.insert(key('b'), Command::BlockMessage);
        shortcuts.insert(key('M'), Command::MockResponse);
        
        // Rule management
        shortcuts.insert(key('R'), Command::ShowRules);
        shortcuts.insert(key('t'), Command::ToggleRule);
        shortcuts.insert(key('e'), Command::EditRule);
        shortcuts.insert(key('T'), Command::TestRule);
        
        // Session management
        shortcuts.insert(key('S'), Command::ShowSessions);
        shortcuts.insert(key('a'), Command::AttachSession);
        shortcuts.insert(key('d'), Command::DetachSession);
        shortcuts.insert(key('E'), Command::ExportSession);
        
        // Display modes
        shortcuts.insert(key('1'), Command::SetDisplayMode(DisplayMode::List));
        shortcuts.insert(key('2'), Command::SetDisplayMode(DisplayMode::Detail));
        shortcuts.insert(key('3'), Command::SetDisplayMode(DisplayMode::Split));
        shortcuts.insert(key('4'), Command::SetDisplayMode(DisplayMode::Rules));
        shortcuts.insert(key('5'), Command::SetDisplayMode(DisplayMode::Sessions));
        
        Self {
            shortcuts,
            mode: InteractionMode::Normal,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    // Global
    Quit,
    Help,
    Refresh,
    Cancel,
    
    // Navigation
    MoveUp,
    MoveDown,
    GoToTop,
    GoToBottom,
    Search,
    SearchNext,
    SearchPrevious,
    
    // Message actions
    ToggleSelect,
    ViewDetail,
    PauseMessage,
    ContinueMessage,
    ModifyMessage,
    BlockMessage,
    MockResponse,
    
    // Rule management
    ShowRules,
    ToggleRule,
    EditRule,
    TestRule,
    
    // Session management
    ShowSessions,
    AttachSession,
    DetachSession,
    ExportSession,
    
    // Display
    SetDisplayMode(DisplayMode),
}

#[derive(Debug, Clone)]
pub enum InteractionMode {
    Normal,
    Search,
    Edit,
    Command,
}
```

### Command Implementation

#### 1. Intercept Start Command
```rust
#[derive(Debug, Parser)]
pub struct InterceptStartCommand {
    /// Command to run for forwarding
    #[arg(last = true)]
    forward_command: Vec<String>,
    
    /// Rules file to load
    #[arg(long)]
    rules: Option<PathBuf>,
    
    /// Rules directory to load
    #[arg(long)]
    rules_dir: Option<PathBuf>,
    
    /// Enable interactive mode
    #[arg(long, default_value = "true")]
    interactive: bool,
    
    /// Output format
    #[arg(long, default_value = "table")]
    output: OutputFormat,
    
    /// Message filter pattern
    #[arg(long)]
    filter: Option<String>,
    
    /// Save session on exit
    #[arg(long)]
    save_session: Option<PathBuf>,
    
    /// Auto-resume timeout (ms)
    #[arg(long)]
    auto_resume: Option<u64>,
    
    /// UI theme
    #[arg(long, default_value = "auto")]
    theme: Theme,
}

impl InterceptStartCommand {
    pub async fn execute(&self) -> CliResult<()> {
        // Setup interceptor chain with rules
        let mut interceptor_chain = InterceptorChain::new();
        self.load_rules(&mut interceptor_chain).await?;
        
        // Start forward proxy with interception
        let proxy_config = ProxyConfig {
            interceptor_chain: Some(interceptor_chain),
            ..Default::default()
        };
        
        let mut proxy = ForwardProxy::new(proxy_config);
        
        if self.interactive {
            // Start interactive dashboard
            let dashboard = InterceptDashboard::new(proxy, self.create_dashboard_config());
            dashboard.run().await?;
        } else {
            // Run headless
            proxy.run_headless(&self.forward_command).await?;
        }
        
        // Save session if requested
        if let Some(save_path) = &self.save_session {
            self.save_session_data(save_path).await?;
        }
        
        Ok(())
    }
    
    async fn load_rules(&self, chain: &mut InterceptorChain) -> CliResult<()> {
        // Load rules from file
        if let Some(rules_file) = &self.rules {
            let rules = self.load_rules_from_file(rules_file).await?;
            for rule in rules {
                chain.add_rule_interceptor(rule).await?;
            }
        }
        
        // Load rules from directory
        if let Some(rules_dir) = &self.rules_dir {
            let rules = self.load_rules_from_directory(rules_dir).await?;
            for rule in rules {
                chain.add_rule_interceptor(rule).await?;
            }
        }
        
        Ok(())
    }
}
```

#### 2. Rules Management Commands
```rust
#[derive(Debug, Parser)]
pub struct RulesListCommand {
    /// Show only enabled rules
    #[arg(long)]
    enabled: bool,
    
    /// Output format
    #[arg(long, default_value = "table")]
    format: OutputFormat,
    
    /// Show detailed information
    #[arg(short, long)]
    verbose: bool,
    
    /// Filter by tags
    #[arg(long)]
    tags: Option<Vec<String>>,
    
    /// Filter by priority range
    #[arg(long)]
    priority: Option<String>, // Format: "10-100"
}

impl RulesListCommand {
    pub async fn execute(&self) -> CliResult<()> {
        let rule_engine = RuleEngine::new(RuleEngineConfig::default());
        let rules = rule_engine.list_rules().await?;
        
        let filtered_rules = self.apply_filters(&rules);
        
        match self.format {
            OutputFormat::Table => {
                self.display_rules_table(&filtered_rules)?;
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&filtered_rules)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&filtered_rules)?);
            }
        }
        
        Ok(())
    }
    
    fn display_rules_table(&self, rules: &[Arc<Rule>]) -> CliResult<()> {
        let mut table = Table::new();
        table.set_header(vec!["ID", "Name", "Priority", "Enabled", "Conditions", "Actions"]);
        
        for rule in rules {
            let conditions_summary = self.summarize_conditions(&rule.conditions);
            let actions_summary = format!("{} actions", rule.actions.len());
            
            table.add_row(vec![
                rule.id.clone(),
                rule.name.clone(),
                rule.priority.to_string(),
                if rule.enabled { "✓".to_string() } else { "✗".to_string() },
                conditions_summary,
                actions_summary,
            ]);
        }
        
        println!("{}", table);
        Ok(())
    }
    
    fn apply_filters(&self, rules: &[Arc<Rule>]) -> Vec<Arc<Rule>> {
        rules.iter()
            .filter(|rule| {
                // Filter by enabled status
                if self.enabled && !rule.enabled {
                    return false;
                }
                
                // Filter by tags
                if let Some(filter_tags) = &self.tags {
                    let rule_tags: Vec<String> = rule.metadata
                        .get("tags")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect())
                        .unwrap_or_default();
                    
                    if !filter_tags.iter().any(|tag| rule_tags.contains(tag)) {
                        return false;
                    }
                }
                
                // Filter by priority range
                if let Some(priority_range) = &self.priority {
                    if let Some((min, max)) = self.parse_priority_range(priority_range) {
                        if rule.priority < min || rule.priority > max {
                            return false;
                        }
                    }
                }
                
                true
            })
            .cloned()
            .collect()
    }
}
```

## Implementation Details

### Phase 1: Core Commands (Day 1)
1. Implement basic command structure with clap
2. Add intercept start/stop/attach commands
3. Create rule list/add/remove commands
4. Add session management commands
5. Write unit tests for command parsing

### Phase 2: Interactive Interface (Day 1-2)
1. Implement dashboard with message display
2. Add keyboard shortcuts and navigation
3. Create rule editor with validation
4. Add real-time message monitoring
5. Test interactive features

### Phase 3: Advanced Features (Day 2)
1. Add message filtering and search
2. Implement rule testing against tapes
3. Create session export/import
4. Add syntax highlighting and themes
5. Performance optimization for large message volumes

### Phase 4: Integration and Polish (Day 2)
1. Integrate with interceptor engine and rule engine
2. Add comprehensive error handling
3. Create help system and documentation
4. Add auto-completion support
5. End-to-end testing

## Acceptance Criteria

### Functional Requirements
- [ ] All intercept commands work correctly
- [ ] Interactive dashboard displays messages in real-time
- [ ] Rule management commands validate and apply rules
- [ ] Session attachment/detachment works seamlessly
- [ ] Message filtering and search are responsive
- [ ] Export/import maintains data integrity
- [ ] Keyboard shortcuts provide efficient navigation

### Usability Requirements
- [ ] Interface is intuitive and discoverable
- [ ] Error messages are helpful and actionable
- [ ] Performance is responsive with 1000+ messages
- [ ] Help system provides comprehensive guidance
- [ ] Color themes work in different terminal environments

### Quality Requirements
- [ ] Comprehensive test coverage for all commands
- [ ] Integration tests with interceptor engine
- [ ] Performance tests for high message volumes
- [ ] Error handling for all failure scenarios
- [ ] Documentation for all commands and features

## Dependencies

### Internal Dependencies
- InterceptorChain from interceptor engine (Task 010)
- RuleEngine from rule engine (Task 011)
- ActionExecutor from intercept actions (Task 012)
- Session management and tape storage

### External Dependencies
- `clap` for command-line parsing
- `crossterm` for terminal control
- `tui-rs` or `ratatui` for terminal UI
- `syntect` for syntax highlighting
- `serde_yaml` for YAML support
- `tokio` for async runtime

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Tests passing with > 90% coverage
- [ ] Interactive interface is responsive and intuitive
- [ ] All commands work with proper error handling
- [ ] Documentation complete with examples
- [ ] Code review completed and approved
- [ ] Performance validated with large datasets

## Follow-up Tasks

- **Task 014:** Persistent Rule Storage
- Web-based UI for remote debugging
- IDE integration and language server protocol
- Advanced visualization and analytics
- Collaborative debugging features