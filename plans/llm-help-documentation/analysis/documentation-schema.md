# Documentation Schema Design

## JSON Schema Definition

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://shadowcat.dev/schemas/cli-documentation.json",
  "title": "CLI Documentation Schema",
  "type": "object",
  "required": ["name", "version", "description"],
  "properties": {
    "name": {
      "type": "string",
      "description": "The CLI application name"
    },
    "version": {
      "type": "string",
      "description": "Application version"
    },
    "description": {
      "type": "string",
      "description": "Brief description of the application"
    },
    "long_description": {
      "type": "string",
      "description": "Detailed description of the application"
    },
    "global_options": {
      "type": "array",
      "items": { "$ref": "#/definitions/option" }
    },
    "commands": {
      "type": "array",
      "items": { "$ref": "#/definitions/command" }
    },
    "examples": {
      "type": "array",
      "items": { "$ref": "#/definitions/example" }
    },
    "environment_variables": {
      "type": "array",
      "items": { "$ref": "#/definitions/env_var" }
    }
  },
  "definitions": {
    "command": {
      "type": "object",
      "required": ["name", "description"],
      "properties": {
        "name": {
          "type": "string",
          "description": "Command name"
        },
        "description": {
          "type": "string",
          "description": "Brief command description"
        },
        "long_description": {
          "type": "string",
          "description": "Detailed command description"
        },
        "usage": {
          "type": "string",
          "description": "Usage pattern string"
        },
        "options": {
          "type": "array",
          "items": { "$ref": "#/definitions/option" }
        },
        "arguments": {
          "type": "array",
          "items": { "$ref": "#/definitions/argument" }
        },
        "subcommands": {
          "type": "array",
          "items": { "$ref": "#/definitions/command" }
        },
        "examples": {
          "type": "array",
          "items": { "$ref": "#/definitions/example" }
        },
        "aliases": {
          "type": "array",
          "items": { "type": "string" }
        }
      }
    },
    "option": {
      "type": "object",
      "required": ["name", "description"],
      "properties": {
        "name": {
          "type": "string",
          "description": "Option name (without dashes)"
        },
        "short": {
          "type": "string",
          "description": "Short flag (single character)",
          "pattern": "^[a-zA-Z]$"
        },
        "long": {
          "type": "string",
          "description": "Long flag name"
        },
        "description": {
          "type": "string",
          "description": "Option description"
        },
        "value_name": {
          "type": "string",
          "description": "Placeholder for the value (e.g., FILE, PORT)"
        },
        "type": {
          "type": "string",
          "enum": ["string", "number", "boolean", "array", "path"],
          "description": "Data type of the option value"
        },
        "default": {
          "description": "Default value if not specified"
        },
        "required": {
          "type": "boolean",
          "description": "Whether this option is required",
          "default": false
        },
        "multiple": {
          "type": "boolean",
          "description": "Can be specified multiple times",
          "default": false
        },
        "possible_values": {
          "type": "array",
          "description": "List of valid values (for enums)",
          "items": {
            "type": "object",
            "properties": {
              "value": { "type": "string" },
              "description": { "type": "string" }
            }
          }
        },
        "conflicts_with": {
          "type": "array",
          "description": "Options that conflict with this one",
          "items": { "type": "string" }
        },
        "requires": {
          "type": "array",
          "description": "Options required when this is used",
          "items": { "type": "string" }
        }
      }
    },
    "argument": {
      "type": "object",
      "required": ["name", "description"],
      "properties": {
        "name": {
          "type": "string",
          "description": "Argument name"
        },
        "description": {
          "type": "string",
          "description": "Argument description"
        },
        "type": {
          "type": "string",
          "enum": ["string", "number", "path", "command"],
          "default": "string"
        },
        "required": {
          "type": "boolean",
          "description": "Whether this argument is required",
          "default": true
        },
        "multiple": {
          "type": "boolean",
          "description": "Accepts multiple values",
          "default": false
        },
        "variadic": {
          "type": "boolean",
          "description": "Trailing variadic arguments",
          "default": false
        },
        "default": {
          "description": "Default value if not provided"
        }
      }
    },
    "example": {
      "type": "object",
      "required": ["command", "description"],
      "properties": {
        "command": {
          "type": "string",
          "description": "The example command line"
        },
        "description": {
          "type": "string",
          "description": "What this example demonstrates"
        },
        "output": {
          "type": "string",
          "description": "Expected output or behavior"
        }
      }
    },
    "env_var": {
      "type": "object",
      "required": ["name", "description"],
      "properties": {
        "name": {
          "type": "string",
          "description": "Environment variable name"
        },
        "description": {
          "type": "string",
          "description": "What this variable controls"
        },
        "default": {
          "type": "string",
          "description": "Default value if not set"
        },
        "example": {
          "type": "string",
          "description": "Example value"
        }
      }
    }
  }
}
```

## Rust Structure Definitions

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CliDocumentation {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub global_options: Vec<OptionDoc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub commands: Vec<CommandDoc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub examples: Vec<ExampleDoc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub environment_variables: Vec<EnvVarDoc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandDoc {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub options: Vec<OptionDoc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub arguments: Vec<ArgumentDoc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub subcommands: Vec<CommandDoc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub examples: Vec<ExampleDoc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionDoc {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<char>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_name: Option<String>,
    #[serde(rename = "type")]
    pub value_type: ValueType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub multiple: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub possible_values: Vec<PossibleValue>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub conflicts_with: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub requires: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArgumentDoc {
    pub name: String,
    pub description: String,
    #[serde(rename = "type", default)]
    pub value_type: ValueType,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub multiple: bool,
    #[serde(default)]
    pub variadic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExampleDoc {
    pub command: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvVarDoc {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PossibleValue {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Array,
    Path,
    Command,
}

impl Default for ValueType {
    fn default() -> Self {
        ValueType::String
    }
}

fn default_true() -> bool {
    true
}
```

## Sample JSON Output

```json
{
  "name": "shadowcat",
  "version": "0.1.0",
  "description": "High-performance MCP proxy with recording and interception capabilities",
  "global_options": [
    {
      "name": "config",
      "short": "c",
      "long": "config",
      "description": "Path to configuration file",
      "value_name": "CONFIG",
      "type": "path",
      "required": false
    },
    {
      "name": "verbose",
      "long": "verbose",
      "description": "Enable verbose output",
      "type": "boolean",
      "default": false,
      "required": false
    }
  ],
  "commands": [
    {
      "name": "forward",
      "description": "Run forward proxy",
      "usage": "shadowcat forward [OPTIONS] <SUBCOMMAND>",
      "options": [
        {
          "name": "rate-limit",
          "long": "rate-limit",
          "description": "Enable rate limiting",
          "type": "boolean",
          "default": false
        }
      ],
      "subcommands": [
        {
          "name": "stdio",
          "description": "Forward proxy over stdio",
          "arguments": [
            {
              "name": "command",
              "description": "Command and arguments to spawn as MCP server",
              "type": "command",
              "required": true,
              "variadic": true
            }
          ]
        }
      ]
    }
  ],
  "examples": [
    {
      "command": "shadowcat forward stdio -- mcp-server --debug",
      "description": "Start a forward proxy spawning an MCP server with debug mode"
    }
  ]
}
```

## Markdown Template

```markdown
# {name}

{description}

**Version:** {version}

## Installation

[Installation instructions if available]

## Global Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| --{long} | -{short} | {description} | {default} |

## Commands

### `{command_name}`

{command_description}

**Usage:** `{usage}`

#### Options

- `--{option}`: {description} (default: {default})

#### Arguments

- `{argument}`: {description} {required_marker}

#### Subcommands

[List of subcommands]

#### Examples

```bash
{example_command}
```
{example_description}

## Environment Variables

- `{ENV_VAR}`: {description} (default: {default})

## See Also

- Project repository: https://github.com/shadowcat
- Documentation: https://shadowcat.dev
```

This schema provides a comprehensive structure for documenting the CLI that is both machine-readable and human-friendly.