# Task B.1: Multiple Upstream Support

**Status**: Not Started  
**Estimated Duration**: 4 hours  
**Dependencies**: A.2 (Configuration File Format Design)

## Objective
Implement support for multiple upstream servers in the reverse proxy CLI, enabling high availability and load distribution scenarios.

## Deliverables
1. **Enhanced CLI Arguments**
   - Support multiple `--upstream` flags
   - Add `--upstream-file` for complex configurations
   - Maintain backward compatibility with single upstream

2. **Code Changes**
   - Update `src/cli/reverse.rs` to parse multiple upstreams
   - Modify configuration building logic
   - Add validation for upstream configurations

3. **Tests**
   - Unit tests for parsing multiple upstreams
   - Integration test with multiple backends
   - Backward compatibility test

## Implementation Plan

### Step 1: Update CLI Structure (1 hour)
```rust
#[derive(Debug, Args)]
pub struct ReverseCommand {
    // Change from String to Vec<String>
    #[arg(long, action = Append)]
    pub upstream: Vec<String>,
    
    // Add file-based configuration
    #[arg(long, conflicts_with = "upstream")]
    pub upstream_file: Option<PathBuf>,
    
    // Keep existing fields...
}
```

### Step 2: Parse Upstream Configurations (1.5 hours)
- Parse multiple `--upstream` arguments
- Load and validate `--upstream-file` if provided
- Convert to `Vec<ReverseUpstreamConfig>`
- Handle both stdio and HTTP upstreams

### Step 3: Update Configuration Building (1 hour)
- Modify `run_reverse_proxy` to handle multiple upstreams
- Update `ReverseProxyConfig` builder calls
- Ensure proper error handling

### Step 4: Testing (30 minutes)
- Test single upstream (backward compatibility)
- Test multiple upstreams via CLI
- Test upstream file loading
- Test validation errors

## CLI Examples

### Multiple Upstreams via CLI
```bash
shadowcat reverse \
  --bind 127.0.0.1:8080 \
  --upstream http://server1:3000 \
  --upstream http://server2:3000 \
  --upstream http://server3:3000
```

### Upstreams via File
```bash
shadowcat reverse \
  --bind 127.0.0.1:8080 \
  --upstream-file upstreams.yaml
```

### Upstream File Format
```yaml
upstreams:
  - id: primary
    type: http
    url: http://server1:3000
    weight: 2
    enabled: true
  - id: secondary
    type: http
    url: http://server2:3000
    weight: 1
    enabled: true
  - id: backup
    type: stdio
    command: ["/usr/local/bin/mcp-server"]
    weight: 1
    enabled: false
```

## Code Locations
- **CLI Definition**: `src/cli/reverse.rs`
- **Configuration Types**: `src/proxy/reverse.rs`
- **Tests**: `src/cli/reverse.rs` (unit), `tests/integration_reverse_proxy.rs`

## Validation Rules
1. At least one upstream must be specified
2. Upstream IDs must be unique (when using file)
3. URLs must be valid HTTP/HTTPS
4. Stdio commands must have at least one argument
5. Weights must be positive integers

## Error Handling
- Clear error for no upstreams specified
- Validation errors for malformed URLs
- File parsing errors with line numbers
- Duplicate ID detection

## Testing Checklist
- [ ] Single upstream still works (backward compatibility)
- [ ] Multiple HTTP upstreams
- [ ] Mixed HTTP and stdio upstreams
- [ ] Upstream file parsing
- [ ] Validation error messages
- [ ] Weight-based distribution (if load balancing enabled)

## Success Criteria
- [ ] Multiple upstreams can be specified via CLI
- [ ] Upstream file format is documented and works
- [ ] Backward compatibility maintained
- [ ] All tests passing
- [ ] Help text updated

## Notes
- Consider adding `--upstream-check` to validate without starting
- May want to add `--upstream-timeout` per upstream
- Future: Support for upstream templates/groups