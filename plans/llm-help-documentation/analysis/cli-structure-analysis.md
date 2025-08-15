# Shadowcat CLI Structure Analysis

## Command Hierarchy

```
shadowcat
├── forward          (Forward proxy)
│   ├── stdio        (Process-based MCP communication)
│   └── http         (HTTP POST + optional SSE)
├── reverse          (Reverse proxy)
├── record           (Record session to tape)
├── replay           (Replay session from tape)
├── tape             (Manage recorded tapes)
│   ├── list
│   ├── show
│   ├── delete
│   └── export
├── intercept        (Manage interception and rules)
│   ├── enable
│   ├── disable
│   ├── list
│   └── add-rule
├── session          (Manage sessions)
│   ├── list
│   ├── show
│   ├── kill
│   └── cleanup
└── completions      (Generate shell completions)
```

## Global Options

- `-c, --config <CONFIG>`: Path to configuration file
- `--log-level <LOG_LEVEL>`: Set log level
- `--verbose`: Enable verbose output
- `--storage-dir <STORAGE_DIR>`: Storage directory for tapes (default: ./tapes)
- `-h, --help`: Print help
- `-V, --version`: Print version

## Command-Specific Options

### Forward Command
- `--rate-limit`: Enable rate limiting
- `--rate-limit-rpm <N>`: Requests per minute (default: 100)
- `--rate-limit-burst <N>`: Burst size (default: 20)
- `--session-timeout <SECS>`: Session timeout (default: 300)
- `--max-sessions <N>`: Maximum concurrent sessions (default: 1000)
- `--cleanup-interval <SECS>`: Cleanup interval (default: 60)

#### Forward Stdio Subcommand
- Trailing arguments: Command and arguments to spawn as MCP server

#### Forward HTTP Subcommand
- `--port <PORT>`: Local port to listen on
- `--bind <ADDR>`: Bind address
- `--target <URL>`: Target MCP server URL

### Reverse Command
- `--bind <ADDR>`: Bind address (default: 127.0.0.1:8080)
- `--upstream <URL>`: Upstream MCP server URL
- `--auth-mode <MODE>`: Authentication mode
- Various OAuth/JWT options

### Record Command
- `--output <FILE>`: Output tape file
- `--format <FORMAT>`: Tape format (json, binary)
- Command arguments to record

### Replay Command
- Tape file path
- `--port <PORT>`: Port for HTTP replay
- `--speed <MULTIPLIER>`: Playback speed

### Tape Subcommands
- `list`: Show all tapes
  - `--format <FORMAT>`: Output format (table, json)
- `show <TAPE_ID>`: Display tape details
- `delete <TAPE_ID>`: Remove tape
- `export <TAPE_ID>`: Export tape to file
  - `--output <FILE>`: Output file path

### Intercept Subcommands
- `enable`: Enable interception
- `disable`: Disable interception
- `list`: Show active rules
- `add-rule`: Add interception rule
  - `--pattern <REGEX>`: Message pattern
  - `--action <ACTION>`: Action to take

### Session Subcommands
- `list`: Show active sessions
  - `--format <FORMAT>`: Output format
- `show <SESSION_ID>`: Display session details
- `kill <SESSION_ID>`: Terminate session
- `cleanup`: Clean expired sessions

## Implementation Files

- Main entry: `src/main.rs`
- Command modules: `src/cli/`
  - `forward.rs`: Forward proxy command
  - `reverse.rs`: Reverse proxy command
  - `record.rs`: Recording command
  - `replay.rs`: Replay command
  - `tape.rs`: Tape management
  - `intercept.rs`: Interception management
  - `session.rs`: Session management
  - `common.rs`: Shared utilities
  - `error_formatter.rs`: Error formatting

## Key Patterns

1. **Subcommand Structure**: Uses Clap's `#[derive(Subcommand)]` for nested commands
2. **Argument Groups**: Related options grouped in structs with `#[derive(Args)]`
3. **Trailing Arguments**: Uses `trailing_var_arg` for process commands
4. **Format Options**: Common pattern for output format selection
5. **Default Values**: Sensible defaults provided for most options

## Documentation Needs

For LLM consumption, we need to capture:
1. Full command hierarchy with relationships
2. All options with types, defaults, and descriptions
3. Required vs optional arguments
4. Mutual exclusions and dependencies
5. Examples for common use cases
6. Environment variable alternatives
7. Configuration file structure

## Examples to Include

```bash
# Forward proxy with stdio
shadowcat forward stdio -- mcp-server --arg value

# Forward proxy with HTTP
shadowcat forward http --port 8080 --target http://localhost:3000

# Reverse proxy with authentication
shadowcat reverse --bind 0.0.0.0:8080 --upstream http://mcp-server --auth-mode oauth

# Record a session
shadowcat record --output session.tape forward stdio -- mcp-server

# Replay a session
shadowcat replay session.tape --port 8080 --speed 2.0

# Manage tapes
shadowcat tape list --format json
shadowcat tape show tape_123
shadowcat tape delete tape_123

# Session management
shadowcat session list
shadowcat session kill session_456
```

## Integration Points for --help-doc

1. Add to root Commands enum as new variant
2. Create handler in main.rs match statement
3. Access Cli::command() for full introspection
4. Generate documentation in requested format
5. Output to stdout for piping/redirection