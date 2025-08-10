# Implementation Approach Recommendation

## Recommended Approach: Runtime Generation

After analyzing Clap's capabilities and existing solutions, **runtime generation** is the recommended approach for implementing `--help-doc`.

## Technical Architecture

### 1. Core Components

```rust
// Main trait for documentation generation
trait DocGenerator {
    fn generate(&self, cmd: &Command) -> Result<String>;
    fn format_name(&self) -> &str;
}

// Concrete implementations
struct MarkdownGenerator {
    include_examples: bool,
    include_types: bool,
}

struct JsonGenerator {
    schema_version: String,
    compact: bool,
}

struct ManpageGenerator {
    section: u8,  // Man page section (1 for user commands)
    date: String,
    source: String,
}

// Factory for creating generators
enum DocFormat {
    Markdown,
    Json,
    Manpage,
}

impl DocFormat {
    fn create_generator(&self) -> Box<dyn DocGenerator> {
        match self {
            DocFormat::Markdown => Box::new(MarkdownGenerator::default()),
            DocFormat::Json => Box::new(JsonGenerator::default()),
            DocFormat::Manpage => Box::new(ManpageGenerator::default()),
        }
    }
}
```

### 2. Integration Points

#### CLI Integration
```rust
#[derive(Parser)]
struct Cli {
    /// Generate LLM-friendly help documentation
    #[arg(long, value_enum, value_name = "FORMAT")]
    help_doc: Option<DocFormat>,
    
    #[command(subcommand)]
    command: Option<Commands>,
    
    // ... other fields
}

// In main.rs
if let Some(format) = cli.help_doc {
    let cmd = Cli::command();
    let generator = format.create_generator();
    let doc = generator.generate(&cmd)?;
    println!("{}", doc);
    return Ok(());
}
```

#### Command Traversal
```rust
fn traverse_command(cmd: &Command, visitor: &mut impl CommandVisitor) {
    visitor.visit_command(cmd);
    
    for arg in cmd.get_arguments() {
        visitor.visit_argument(arg);
    }
    
    for subcmd in cmd.get_subcommands() {
        traverse_command(subcmd, visitor);
    }
}
```

### 3. Data Collection

```rust
#[derive(Serialize)]
struct CommandDoc {
    name: String,
    version: Option<String>,
    description: Option<String>,
    long_description: Option<String>,
    usage: String,
    arguments: Vec<ArgumentDoc>,
    subcommands: Vec<CommandDoc>,
    examples: Vec<Example>,
}

#[derive(Serialize)]
struct ArgumentDoc {
    name: String,
    short: Option<char>,
    long: Option<String>,
    description: Option<String>,
    value_name: Option<String>,
    required: bool,
    multiple: bool,
    default_value: Option<String>,
    possible_values: Option<Vec<String>>,
    arg_type: String, // flag, option, positional
}

#[derive(Serialize)]
struct Example {
    description: String,
    command: String,
    output: Option<String>,
}
```

## Why Runtime Generation?

### Advantages
1. **No build complexity** - No build.rs or compile-time generation needed
2. **Always current** - Documentation reflects actual runtime command structure
3. **Feature-aware** - Can include/exclude based on compile features
4. **Dynamic content** - Can include runtime information (version, paths, etc.)
5. **User-friendly** - Users can generate docs on demand without rebuilding

### Disadvantages (and mitigations)
1. **Slight startup overhead** - Mitigated by only running when --help-doc is used
2. **Binary size increase** - Minimal, mostly string formatting code
3. **Can't include in distribution** - Users generate on their system as needed

## Implementation Phases

### Phase 1: Core Infrastructure
1. Create DocGenerator trait and basic implementations
2. Add --help-doc flag to CLI
3. Implement command traversal
4. Basic markdown output

### Phase 2: Enhanced Metadata
1. Add example collection system
2. Include type information extraction
3. Document argument relationships
4. Add usage pattern generation

### Phase 3: Format Support
1. Implement JSON generator
2. Implement manpage generator (using ROFF format)
3. Add schema versioning
4. Support format-specific options
5. Add validation

### Phase 4: Polish
1. Optimize for LLM consumption
2. Add filtering options
3. Performance optimization
4. Comprehensive testing

## Risk Assessment

### Low Risk
- Clap provides all necessary APIs
- Pattern proven in clap_complete
- Runtime generation is straightforward

### Medium Risk
- Example collection may require convention
- Type information extraction limited to Clap metadata
- May need custom attributes for richer documentation

### Mitigation Strategies
1. Start with basic implementation, enhance iteratively
2. Use doc comments and long_about for rich descriptions
3. Consider helper macro for examples if needed
4. Fall back to manual examples in specific commands

## Comparison with Build-time Generation

### Build-time Approach
```rust
// build.rs
fn main() {
    let cmd = build_command();
    let doc = generate_documentation(&cmd);
    fs::write("docs/cli-help.md", doc).unwrap();
}
```

**Pros:**
- Documentation included in distribution
- No runtime overhead
- Can be version controlled

**Cons:**
- Increases build complexity
- May not reflect runtime features
- Requires rebuild for doc updates
- Distribution size increase

### Runtime Approach (Recommended)
```rust
// main.rs
if let Some(format) = cli.help_doc {
    let doc = generate_documentation(&cmd, format);
    println!("{}", doc);
}
```

**Pros:**
- Simple implementation
- Always accurate
- No build complexity
- User-generated on demand

**Cons:**
- Small runtime overhead
- Not included in distribution

## Conclusion

Runtime generation is recommended because:
1. Shadowcat already uses derive macros, making runtime introspection natural
2. The pattern is proven in clap_complete
3. It avoids build complexity
4. LLMs can request documentation on-demand
5. The implementation is straightforward and maintainable