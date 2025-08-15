# Current CLI Structure Analysis

## Executive Summary

Shadowcat's current CLI follows a traditional subcommand pattern but requires users to understand proxy terminology ("forward" vs "reverse") which creates friction for new users. The interface lacks auto-detection capabilities that could make common use cases more intuitive.

## Current Command Structure

### Top-Level Commands

```
shadowcat [OPTIONS] <COMMAND>
```

| Command | Purpose | User Mental Model Challenge |
|---------|---------|----------------------------|
| `forward` | Client → Proxy → Server | Clear enough, but requires subcommand |
| `reverse` | External → Proxy → Backend | "Reverse proxy" is technical jargon |
| `record` | Capture session to tape | Clear and intuitive |
| `replay` | Play back session from tape | Clear and intuitive |
| `tape` | Manage recorded sessions | Clear management command |
| `intercept` | Manage message interception | Advanced feature, clear naming |
| `session` | Manage active sessions | Management command |
| `completions` | Shell completions | Standard utility |
| `help-doc` | LLM documentation | Specialized utility |

### Global Options
- `-c, --config <CONFIG>`: Configuration file path
- `--log-level <LOG_LEVEL>`: Logging verbosity
- `--verbose`: Enable verbose output
- `--storage-dir <STORAGE_DIR>`: Tape storage location (default: ./tapes)
- `-h, --help`: Help information
- `-V, --version`: Version information

## Command Deep Dive

### Forward Command

**Current Usage:**
```bash
shadowcat forward stdio -- my-mcp-server
shadowcat forward streamable-http --target http://localhost:3000
```

**Structure:**
- Requires explicit transport selection (stdio or streamable-http)
- stdio: Takes trailing command arguments
- streamable-http: Requires --target flag

**Pain Points:**
1. Two-step mental model (command → transport)
2. Different syntax for different transports
3. No auto-detection of intent

### Reverse Command (Should be "Gateway")

**Current Usage:**
```bash
shadowcat reverse --upstream http://backend:3000 --bind 0.0.0.0:8080
```

**Structure:**
- Always requires --upstream flag
- Optional --bind (defaults to 127.0.0.1:8080)
- Authentication options available but complex

**Pain Points:**
1. "Reverse proxy" terminology confusing for non-infrastructure developers
2. Required flags make simple cases verbose
3. No smart defaults based on common patterns

## Execution Flow Analysis

### Current Flow (main.rs)

```rust
1. Parse CLI args → Cli struct
2. Handle special commands early (completions, help-doc)
3. Load configuration
4. Initialize logging/telemetry
5. Match on command enum:
   - Forward → ForwardCommand handler
   - Reverse → ReverseCommand handler
   - Record → RecordCommand handler
   - etc.
6. Execute command with shutdown support where needed
```

### Command Implementation Pattern

Each command follows this pattern:
1. Command struct with clap derive macros
2. `execute()` method that:
   - Validates arguments
   - Creates appropriate transport
   - Initializes proxy/recorder
   - Runs async loop

## Identified Improvement Opportunities

### 1. Smart Auto-Detection

**Opportunity**: Detect intent from first argument pattern
```bash
# Current (verbose)
shadowcat forward stdio -- my-server
shadowcat reverse --upstream http://api.example.com

# Potential (intuitive)
shadowcat my-server                    # Detects: forward stdio
shadowcat http://api.example.com       # Detects: gateway mode
shadowcat :8080                        # Detects: gateway on port 8080
shadowcat session.tape                 # Detects: replay mode
```

### 2. Terminology Improvement

**Current Problem**: "Reverse proxy" is infrastructure jargon

**Solution**: Use "gateway" - more intuitive for API developers
```bash
# Current (confusing)
shadowcat reverse --upstream http://backend

# Improved (clear)
shadowcat gateway --upstream http://backend
# or even better with auto-detection:
shadowcat gateway http://backend
```

### 3. Unified Transport Handling

**Current**: Different syntax for stdio vs HTTP
```bash
shadowcat forward stdio -- cmd args
shadowcat forward streamable-http --target http://...
```

**Improved**: Consistent positional argument
```bash
shadowcat forward cmd args           # stdio detected
shadowcat forward http://localhost   # HTTP detected
```

## Dependencies and Constraints

### Code Dependencies
- `clap` for CLI parsing (v4.x with derive macros)
- Command modules in `src/cli/`
- Each command has dedicated module with struct definition

### Breaking Changes if Modified
1. Scripts using `reverse` command would break
2. Documentation referencing current commands
3. Test suites expecting specific command names
4. Configuration files with command-specific settings

### Backward Compatibility Options
1. Keep old commands as hidden aliases
2. Deprecation warnings for old syntax
3. Migration guide in documentation
4. Auto-migration of config files

## Complexity Assessment

### Low Complexity Changes
- Adding gateway as alias for reverse
- Improving help text
- Adding examples to commands

### Medium Complexity Changes
- Smart detection logic in main.rs
- Unified argument handling
- Better error messages

### High Complexity Changes
- Full command restructure
- Removing subcommands from forward
- Changing core execution flow

## Recommendations

### Phase 1: Quick Wins (2-4 hours)
1. Add "gateway" as alias for "reverse"
2. Improve help text with examples
3. Add smart detection for simple cases

### Phase 2: Core Improvements (8-12 hours)
1. Implement full auto-detection logic
2. Unify transport argument handling
3. Enhance error messages with suggestions

### Phase 3: Polish (4-6 hours)
1. Update all documentation
2. Add migration warnings
3. Create comprehensive examples

## Testing Impact

### Affected Test Categories
1. CLI argument parsing tests
2. Integration tests using command-line
3. Documentation examples
4. Shell completion tests

### Test Migration Strategy
1. Keep tests for old syntax (deprecated)
2. Add parallel tests for new syntax
3. Gradually migrate as old syntax is phased out

## Conclusion

The current CLI structure is functional but creates unnecessary friction for users. The main issues are:

1. **Confusing terminology** ("reverse proxy" vs more intuitive "gateway")
2. **Lack of smart defaults** requiring explicit flags for common cases
3. **Inconsistent syntax** between different transport types

The proposed improvements would make Shadowcat more approachable while maintaining full control for advanced users. The key is to make the common case magical without making the complex case impossible.