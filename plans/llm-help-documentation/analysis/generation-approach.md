# Documentation Generation Approach Evaluation

## Approach Options

### 1. Runtime Generation (RECOMMENDED) ✅
Generate documentation on-demand when `--help-doc` is invoked.

**Pros:**
- Always up-to-date with current CLI structure
- No build-time complexity
- Works with dynamically configured commands
- Simple implementation using Clap introspection
- No separate build step required

**Cons:**
- Small runtime overhead (~10-50ms)
- Cannot be cached at compile time

**Implementation:**
```rust
// In main.rs
match cli.command {
    Commands::HelpDoc { format } => {
        let cmd = Cli::command();
        let doc = generate_documentation(&cmd, format);
        println!("{}", doc);
    }
}
```

### 2. Build-Time Generation
Generate documentation during build using build.rs.

**Pros:**
- Zero runtime overhead
- Can be included in binary as static string
- Can generate multiple format files

**Cons:**
- Complex build.rs setup
- May get out of sync with runtime changes
- Requires rebuilding for doc updates
- Cannot capture dynamic elements

### 3. Hybrid Approach
Cache generated documentation after first run.

**Pros:**
- Fast after first generation
- Can invalidate based on version

**Cons:**
- Cache management complexity
- Minimal performance benefit for small docs

## Decision: Runtime Generation

Based on analysis, **runtime generation** is the clear winner because:

1. **Simplicity**: Straightforward implementation with no build complexity
2. **Correctness**: Always reflects actual CLI structure
3. **Performance**: <50ms generation time is negligible
4. **Maintenance**: No cache or build script to maintain

## Implementation Strategy

### Phase 1: Core Generator
```rust
// src/cli/doc_gen.rs
pub struct DocGenerator {
    command: Command,
}

impl DocGenerator {
    pub fn new(command: Command) -> Self {
        Self { command }
    }
    
    pub fn generate(&self, format: DocFormat) -> String {
        match format {
            DocFormat::Markdown => self.generate_markdown(),
            DocFormat::Json => self.generate_json(),
            DocFormat::Manpage => self.generate_manpage(),
        }
    }
}
```

### Phase 2: Format Handlers
```rust
impl DocGenerator {
    fn generate_markdown(&self) -> String {
        let mut output = String::new();
        self.write_markdown_command(&self.command, &mut output, 0);
        output
    }
    
    fn generate_json(&self) -> String {
        let doc = self.command_to_json(&self.command);
        serde_json::to_string_pretty(&doc).unwrap()
    }
}
```

### Phase 3: CLI Integration
```rust
// Add to Commands enum
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...
    
    #[command(about = "Generate comprehensive CLI documentation")]
    HelpDoc {
        #[arg(value_enum, default_value = "markdown")]
        format: DocFormat,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum DocFormat {
    Markdown,
    Json,
    Manpage,
}
```

## Performance Considerations

### Expected Performance
- Command tree traversal: ~1ms
- Markdown generation: ~5-10ms
- JSON serialization: ~5-10ms
- Total time: <50ms for full documentation

### Optimization Opportunities
1. **Lazy generation**: Only generate requested format
2. **Streaming output**: Write directly to stdout
3. **Parallel processing**: Generate subcommands concurrently (if needed)

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_markdown_generation() {
    let cmd = create_test_command();
    let gen = DocGenerator::new(cmd);
    let markdown = gen.generate(DocFormat::Markdown);
    assert!(markdown.contains("# shadowcat"));
}
```

### Integration Tests
```rust
#[test]
fn test_help_doc_command() {
    let output = Command::new("shadowcat")
        .arg("--help-doc")
        .output()
        .unwrap();
    assert!(output.status.success());
}
```

### Validation Tests
- Parse JSON output with serde_json
- Validate Markdown structure
- Check for completeness (all commands present)

## Format Specifications

### JSON Schema
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "version": { "type": "string" },
    "description": { "type": "string" },
    "global_options": {
      "type": "array",
      "items": { "$ref": "#/definitions/option" }
    },
    "subcommands": {
      "type": "array",
      "items": { "$ref": "#/definitions/command" }
    }
  }
}
```

### Markdown Structure
```markdown
# {name}

{description}

**Version:** {version}

## Global Options

{options_table_or_list}

## Commands

### {command_name}

{command_description}

#### Usage

```
{usage_pattern}
```

#### Options

{options_list}

#### Examples

{examples}
```

## Conclusion

Runtime generation using Clap's introspection APIs is the optimal approach for this feature. It provides the best balance of simplicity, correctness, and performance while maintaining the codebase's clean architecture.