# LLM Documentation Best Practices

## Key Principles for LLM-Consumable Documentation

### 1. Structure and Hierarchy
- **Clear nesting**: Use consistent indentation and hierarchy markers
- **Explicit relationships**: Show parent-child command relationships
- **Breadcrumbs**: Include full command paths (e.g., `shadowcat tape list`)

### 2. Completeness
- **All options**: Include every flag, option, and argument
- **Types**: Specify data types (string, number, boolean, enum)
- **Defaults**: Always show default values when applicable
- **Constraints**: Document min/max values, regex patterns, valid choices

### 3. Format Considerations

#### JSON Format
```json
{
  "command": "shadowcat",
  "version": "0.1.0",
  "description": "High-performance MCP proxy",
  "global_options": [...],
  "subcommands": [
    {
      "name": "forward",
      "description": "Run forward proxy",
      "options": [...],
      "subcommands": [...]
    }
  ]
}
```

#### Markdown Format
```markdown
# shadowcat

High-performance MCP proxy

## Global Options
- `--config <FILE>`: Configuration file path
- `--verbose`: Enable verbose output (default: false)

## Commands

### shadowcat forward
Run forward proxy

#### Options
- `--rate-limit`: Enable rate limiting
```

### 4. Examples
- **Real-world usage**: Provide practical, copy-paste examples
- **Common scenarios**: Cover 80% use cases
- **Progressive complexity**: Start simple, add complexity
- **Annotations**: Explain what each example does

### 5. Machine-Readable Metadata
- **JSON Schema**: Define structure for validation
- **Type hints**: Use TypeScript-like notation where helpful
- **Enums**: List all possible values for constrained options
- **Required vs Optional**: Clearly mark required arguments

## Optimal Output Characteristics

### For Tool/Function Calling
```json
{
  "name": "shadowcat_forward",
  "description": "Run forward proxy for MCP protocol",
  "parameters": {
    "type": "object",
    "properties": {
      "rate_limit": {
        "type": "boolean",
        "description": "Enable rate limiting",
        "default": false
      },
      "rate_limit_rpm": {
        "type": "integer",
        "description": "Requests per minute",
        "default": 100,
        "minimum": 1
      }
    },
    "required": []
  }
}
```

### For Natural Language Understanding
```markdown
## Command: shadowcat forward stdio

Starts a forward proxy that spawns a local MCP server process and proxies communication through stdio (standard input/output).

**Usage Pattern:**
```
shadowcat forward stdio -- <command> [args...]
```

**Required:**
- `<command>`: The MCP server executable to spawn

**Common Examples:**
```bash
# Basic usage
shadowcat forward stdio -- mcp-server

# With server arguments
shadowcat forward stdio -- mcp-server --port 3000 --debug

# With proxy options
shadowcat forward --rate-limit --rate-limit-rpm 60 stdio -- mcp-server
```
```

## Format-Specific Guidelines

### JSON Output
- Use consistent schema across all commands
- Include `$schema` reference for validation
- Flatten deeply nested structures where possible
- Use arrays for repeated elements
- Include type information inline

### Markdown Output
- Use ATX headings (#) for hierarchy
- Code blocks with language hints
- Tables for option summaries
- Bold for required parameters
- Inline code for literals

### Manpage Output
- Follow traditional man page sections
- Use proper ROFF formatting
- Include SYNOPSIS section
- Group related options
- Provide SEE ALSO references

## Testing with LLMs

### Coverage Tests
1. Can the LLM generate valid command from description?
2. Can it explain what a complex command does?
3. Can it suggest the right flags for a use case?
4. Can it detect invalid option combinations?

### Validation Criteria
- Commands generated are syntactically correct
- All required arguments included
- No conflicting options used
- Appropriate defaults understood

## Anti-Patterns to Avoid

1. **Ambiguous descriptions**: "Does something with the data"
2. **Missing context**: Options without explaining their effect
3. **Inconsistent formatting**: Mixing styles within documentation
4. **Assumed knowledge**: Unexplained technical terms
5. **Hidden dependencies**: Not documenting option relationships
6. **Outdated examples**: Examples that no longer work

## Recommendations

1. **Start with JSON**: Easiest for LLMs to parse reliably
2. **Layer Markdown**: Human-friendly with good LLM comprehension
3. **Test early**: Validate with actual LLM tools during development
4. **Version the schema**: Include version in output for compatibility
5. **Stream-friendly**: Design for incremental output if possible