# LLM Documentation Standards

## Overview

Based on 2024 best practices from OpenAI, Anthropic, and other LLM providers, this document outlines standards for CLI documentation optimized for LLM consumption.

## Core Principles

### 1. Structured and Predictable
- Consistent schema across all commands
- Clear hierarchy and relationships
- Predictable field names and types
- Version tracking for compatibility

### 2. Complete and Self-Contained
- All information needed to use the tool
- No external references required
- Include constraints and validation rules
- Document error conditions

### 3. Token-Efficient
- Concise descriptions while maintaining clarity
- Avoid redundancy
- Use structured formats over prose
- Support filtering for specific commands

## JSON Schema Format

### Root Structure
```json
{
  "schema_version": "1.0.0",
  "tool_name": "shadowcat",
  "version": "0.1.0",
  "description": "High-performance MCP proxy",
  "commands": [...],
  "global_options": [...]
}
```

### Command Schema
```json
{
  "name": "forward",
  "description": "Run forward proxy",
  "long_description": "Detailed explanation...",
  "usage": "shadowcat forward <TRANSPORT> [OPTIONS] -- <COMMAND>",
  "arguments": [...],
  "options": [...],
  "subcommands": [...],
  "examples": [...],
  "relationships": {
    "conflicts_with": [],
    "requires": [],
    "required_by": []
  }
}
```

### Argument Schema
```json
{
  "name": "port",
  "type": "integer",
  "short": "p",
  "long": "port",
  "description": "Port to bind to",
  "required": false,
  "default": "8080",
  "constraints": {
    "min": 1,
    "max": 65535
  },
  "possible_values": null,
  "multiple": false,
  "value_name": "PORT"
}
```

### Example Schema
```json
{
  "description": "Forward stdio to echo server",
  "command": "shadowcat forward stdio -- echo",
  "expected_output": "Proxy started on stdio transport",
  "explanation": "Creates a forward proxy using stdio transport"
}
```

## Markdown Format

### Structure
```markdown
# Shadowcat CLI Documentation

## Overview
High-performance MCP proxy with recording and interception

## Global Options
- `--help` - Display help information
- `--version` - Display version
- `--log-level <LEVEL>` - Set log level (trace|debug|info|warn|error)

## Commands

### `forward` - Run forward proxy

**Usage:** `shadowcat forward <TRANSPORT> [OPTIONS] -- <COMMAND>`

**Description:** Creates a forward proxy that intercepts MCP traffic

**Arguments:**
- `<TRANSPORT>` - Transport type (stdio|http)
- `<COMMAND>` - Command to execute

**Options:**
- `-p, --port <PORT>` - Port to bind (default: 8080)
- `--target <URL>` - Target server URL

**Examples:**
```bash
# Forward stdio to echo server
shadowcat forward stdio -- echo

# Forward HTTP with specific port
shadowcat forward http --port 9000 --target http://localhost:8080
```

**Subcommands:** None
```

## LLM-Specific Enhancements

### 1. Type Information
Always include type information for arguments:
- `string` - Text value
- `integer` - Whole number
- `float` - Decimal number
- `boolean` - True/false flag
- `path` - File system path
- `url` - Valid URL
- `enum` - One of specific values

### 2. Validation Rules
Document all constraints:
```json
{
  "validation": {
    "pattern": "^[a-zA-Z0-9_-]+$",
    "min_length": 1,
    "max_length": 64,
    "min_value": 0,
    "max_value": 100
  }
}
```

### 3. Relationship Documentation
Explicitly document argument relationships:
```json
{
  "relationships": {
    "conflicts_with": ["--quiet"],
    "requires": ["--output-dir"],
    "required_by": ["--format"],
    "groups": ["output_options"]
  }
}
```

### 4. Examples with Context
Each example should include:
- Description of what it does
- Complete command line
- Expected output (if predictable)
- Common variations
- Error cases

### 5. Progressive Disclosure
Support different detail levels:
- `--help-doc` - Standard documentation
- `--help-doc --verbose` - Include all details
- `--help-doc --command <name>` - Specific command only

## Tool Use Format (Anthropic/OpenAI Compatible)

### Tool Definition Format
```json
{
  "name": "shadowcat_forward",
  "description": "Run forward proxy to intercept MCP traffic",
  "input_schema": {
    "type": "object",
    "properties": {
      "transport": {
        "type": "string",
        "enum": ["stdio", "http"],
        "description": "Transport protocol to use"
      },
      "port": {
        "type": "integer",
        "description": "Port to bind to",
        "default": 8080
      },
      "command": {
        "type": "array",
        "items": {"type": "string"},
        "description": "Command and arguments to execute"
      }
    },
    "required": ["transport", "command"]
  }
}
```

## Best Practices

### 1. Descriptions
- First sentence: What it does
- Second sentence: When to use it
- Third sentence: Important details or caveats

### 2. Error Documentation
Include common error scenarios:
```json
{
  "common_errors": [
    {
      "condition": "Port already in use",
      "message": "Error: Port 8080 is already in use",
      "solution": "Use --port to specify a different port"
    }
  ]
}
```

### 3. Output Format Documentation
Document output formats when relevant:
```json
{
  "output": {
    "format": "json",
    "schema": {...},
    "examples": [...]
  }
}
```

### 4. Environment Variables
Document relevant environment variables:
```json
{
  "environment": [
    {
      "name": "RUST_LOG",
      "description": "Set log level",
      "default": "info",
      "values": ["trace", "debug", "info", "warn", "error"]
    }
  ]
}
```

## Schema Versioning

Use semantic versioning for the documentation schema:
- **Major**: Breaking changes to schema structure
- **Minor**: New fields added (backward compatible)
- **Patch**: Documentation improvements, typo fixes

Include version in output:
```json
{
  "schema_version": "1.0.0",
  "generated_at": "2024-01-15T10:30:00Z",
  "generator_version": "shadowcat-0.1.0"
}
```

## Testing with LLMs

Documentation should be tested with actual LLMs:

1. **Comprehension Test**: Can the LLM understand all commands?
2. **Generation Test**: Can the LLM generate valid commands?
3. **Error Handling**: Can the LLM identify invalid usage?
4. **Completeness**: Are there questions the LLM can't answer?

## Token Optimization

### Strategies for Reducing Token Count
1. Use consistent field names across all commands
2. Abbreviate where unambiguous (desc vs description)
3. Omit null/empty fields
4. Use references for repeated content
5. Support command-specific queries

### Example Optimization
```json
// Verbose
{
  "argument_name": "port",
  "argument_type": "integer",
  "argument_required": false,
  "argument_default": 8080
}

// Optimized
{
  "name": "port",
  "type": "int",
  "required": false,
  "default": 8080
}
```

## Conclusion

Following these standards ensures:
1. LLMs can reliably understand and use the CLI
2. Documentation is comprehensive yet efficient
3. Format is extensible for future needs
4. Consistency with industry standards (OpenAI, Anthropic)