# Shadowcat CLI Design Proposal

## Executive Summary

Transform Shadowcat's CLI from a technical proxy tool to an intuitive MCP development companion. The new design introduces smart auto-detection for common cases while maintaining explicit control, replaces confusing "reverse proxy" terminology with "gateway", and provides helpful guidance through better error messages.

## Design Principles

1. **Make the common case magical** - One command for 90% of use cases
2. **Keep the complex case possible** - Full control remains available  
3. **Be predictable** - Users should understand what will happen
4. **Guide don't gatekeep** - Helpful errors that suggest solutions
5. **Respect existing users** - Maintain backward compatibility

## Proposed Command Structure

### Smart Auto-Detection Mode (NEW)

```bash
# Auto-detect from first argument
shadowcat <target> [options]
```

**Detection Rules:**

| Pattern | Detection | Action |
|---------|-----------|--------|
| `my-command` | Executable name | `forward stdio -- my-command` |
| `./my-script` | File path | `forward stdio -- ./my-script` |
| `http://...` or `https://...` | URL with protocol | `gateway --upstream <url>` |
| `:8080` or `0.0.0.0:8080` | Port binding | `gateway --bind <addr>` |
| `*.tape` | Tape file extension | `replay <file>` |
| `record` + name | Record verb | `record --output <name>` |

### Explicit Command Mode (CURRENT + ENHANCED)

```bash
# Explicit commands for full control
shadowcat <command> [options] [args]
```

**Commands:**

| Command | New Name | Purpose | Change |
|---------|----------|---------|--------|
| `forward` | `forward` | Client → Proxy → Server | Enhanced with smart transport |
| `reverse` | **`gateway`** | API Gateway mode | **RENAMED** for clarity |
| `record` | `record` | Record session | Unchanged |
| `replay` | `replay` | Replay session | Unchanged |
| `tape` | `tape` | Manage tapes | Unchanged |
| `intercept` | `intercept` | Message interception | Unchanged |
| `session` | `session` | Session management | Unchanged |
| `completions` | `completions` | Shell completions | Unchanged |
| `help-doc` | `help-doc` | LLM documentation | Unchanged |

## Detailed Design

### 1. Auto-Detection Implementation

```rust
// In main.rs
enum DetectedMode {
    ForwardStdio(String),
    ForwardHttp(String),
    Gateway { upstream: Option<String>, bind: Option<String> },
    Replay(String),
    Record(String),
    ExplicitCommand,
}

fn detect_mode(args: &[String]) -> DetectedMode {
    if args.is_empty() { return DetectedMode::ExplicitCommand; }
    
    let first = &args[0];
    
    // URL detection
    if first.starts_with("http://") || first.starts_with("https://") {
        return DetectedMode::Gateway { upstream: Some(first), bind: None };
    }
    
    // Port binding detection
    if first.starts_with(":") || first.contains(":") && !first.contains("/") {
        return DetectedMode::Gateway { bind: Some(first), upstream: None };
    }
    
    // Tape file detection
    if first.ends_with(".tape") {
        return DetectedMode::Replay(first);
    }
    
    // Record verb detection
    if first == "record" && args.len() > 1 {
        return DetectedMode::Record(args[1]);
    }
    
    // Check if it's a known command
    if is_known_command(first) {
        return DetectedMode::ExplicitCommand;
    }
    
    // Default: assume forward stdio
    DetectedMode::ForwardStdio(first)
}
```

### 2. Enhanced Forward Command

**Current:**
```bash
shadowcat forward stdio -- my-server
shadowcat forward streamable-http --target http://localhost:3000
```

**Proposed:**
```bash
# Auto-detect transport from target
shadowcat forward my-server                    # Detects stdio
shadowcat forward http://localhost:3000        # Detects HTTP

# Explicit transport still available
shadowcat forward stdio -- my-server
shadowcat forward http --target http://localhost:3000
```

### 3. Gateway Command (Renamed from Reverse)

**Current:**
```bash
shadowcat reverse --upstream http://backend:3000 --bind 0.0.0.0:8080
```

**Proposed:**
```bash
# Positional argument for upstream
shadowcat gateway http://backend:3000           # Bind defaults to 127.0.0.1:8080
shadowcat gateway http://backend:3000 :9090    # Custom bind port

# Or specify bind first
shadowcat gateway :8080 --upstream http://backend:3000

# Full control with flags
shadowcat gateway --upstream http://backend:3000 --bind 0.0.0.0:8080
```

### 4. Enhanced Help System

#### Main Help
```
Shadowcat - High-performance MCP proxy

USAGE:
    shadowcat [OPTIONS] [TARGET] [COMMAND]

SMART DETECTION:
    shadowcat my-mcp-server              # Forward proxy to stdio server
    shadowcat http://api.example.com     # Gateway to HTTP endpoint
    shadowcat :8080                      # Gateway on port 8080
    shadowcat session.tape               # Replay a recorded session

COMMANDS:
    forward    Run a forward proxy (client → shadowcat → server)
    gateway    Run an API gateway (clients → shadowcat → backends)
    record     Record an MCP session
    replay     Replay a recorded session
    tape       Manage recorded sessions
    intercept  Configure message interception
    session    Manage active sessions
    
Run 'shadowcat <command> --help' for more information on a command.

EXAMPLES:
    # Forward proxy to a local MCP server
    shadowcat my-mcp-server
    
    # API gateway on port 8080
    shadowcat gateway :8080 --upstream http://backend
    
    # Record a session
    shadowcat record my-session forward my-server
```

#### Error Messages
```
Error: 'my-server' is not a recognized command or file

Did you mean one of these?
  shadowcat forward my-server         # Run as forward proxy to stdio server
  shadowcat gateway http://my-server  # Use as gateway upstream
  
If 'my-server' is a command you want to run, try:
  shadowcat forward stdio -- my-server

Run 'shadowcat --help' for more options.
```

### 5. Migration Strategy

#### Phase 1: Soft Migration (v0.3.0)
- Add `gateway` as primary command
- Keep `reverse` as hidden alias
- Add deprecation notice to `reverse` help
- Implement auto-detection as opt-in (`--smart` flag)

#### Phase 2: Transition (v0.4.0)
- Make auto-detection default behavior
- Add `--no-auto` flag for explicit mode
- Show migration warnings for `reverse` usage
- Update all documentation

#### Phase 3: Completion (v0.5.0)
- Remove `reverse` from visible commands
- Keep as hidden compatibility alias
- Auto-detection fully integrated

## Backward Compatibility

### Maintained Compatibility
- All existing commands continue to work
- All existing flags remain unchanged
- Config files remain compatible
- API/Library usage unaffected

### Breaking Changes
- None in Phase 1
- `reverse` command deprecated (but functional) in Phase 2
- Full removal only in major version (v1.0.0)

### Migration Helpers
```bash
# Automatic suggestion on old usage
$ shadowcat reverse --upstream http://backend
Note: 'reverse' is deprecated. Use 'gateway' instead:
  shadowcat gateway --upstream http://backend
  
# Config file migration tool
$ shadowcat migrate-config old.yaml new.yaml
```

## Implementation Priority

### High Priority (Core Experience)
1. Add auto-detection logic in main.rs
2. Rename reverse to gateway (with alias)
3. Enhance forward command with transport detection
4. Improve error messages with suggestions

### Medium Priority (Polish)
1. Update help text with examples
2. Add `--verbose` to show detection logic
3. Create migration warnings
4. Update shell completions

### Low Priority (Nice-to-Have)
1. Config file auto-migration
2. Interactive mode for ambiguous inputs
3. Tutorial command (`shadowcat tutorial`)
4. Performance optimizations for detection

## Testing Strategy

### Unit Tests
- Test detection logic with various inputs
- Verify backward compatibility
- Test error message generation

### Integration Tests
- End-to-end CLI invocation tests
- Test all detection patterns
- Verify explicit commands still work

### User Acceptance Tests
1. New user can run MCP server without reading docs
2. Existing scripts continue to work
3. Error messages lead to solution
4. Help text is sufficient for basic usage

## Success Metrics

### Quantitative
- 90% of new users successful on first try
- < 100ms overhead for auto-detection
- Zero breaking changes for existing users

### Qualitative
- "It just works" feedback
- Reduced support questions about forward vs reverse
- Positive community response to gateway terminology

## Example User Journeys

### New User - First Time
```bash
$ shadowcat my-mcp-server
Starting forward proxy to stdio server 'my-mcp-server'...
Proxy running on stdio. Press Ctrl+C to stop.

$ shadowcat http://localhost:3000
Starting gateway to http://localhost:3000...
Gateway listening on 127.0.0.1:8080. Press Ctrl+C to stop.
```

### Existing User - Migration
```bash
$ shadowcat reverse --upstream http://api
Note: 'reverse' is deprecated. Use 'gateway' instead:
  shadowcat gateway --upstream http://api
  
Starting gateway to http://api...
[continues as normal]
```

### Power User - Full Control
```bash
$ shadowcat forward stdio \
    --rate-limit \
    --rate-limit-rpm 100 \
    --session-timeout 600 \
    -- my-server --debug --verbose

Starting forward proxy with rate limiting...
[full control maintained]
```

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Auto-detection misidentifies intent | `--no-auto` flag for explicit mode |
| Users confused by changes | Comprehensive migration guide |
| Scripts break | Maintain full backward compatibility |
| Performance regression | Benchmark detection logic, < 1ms target |

## Conclusion

This design proposal transforms Shadowcat from a powerful but technical tool into an intuitive MCP development companion. By implementing smart auto-detection, clarifying terminology, and providing helpful guidance, we can dramatically improve the new user experience while maintaining the power features that advanced users rely on.

The phased migration approach ensures existing users are not disrupted while new users benefit immediately from the improvements. The design follows proven patterns from successful CLI tools while respecting Shadowcat's unique requirements.