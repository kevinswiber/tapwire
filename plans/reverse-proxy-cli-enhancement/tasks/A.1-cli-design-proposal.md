# Task A.1: CLI Design Proposal

**Status**: Not Started  
**Estimated Duration**: 3 hours  
**Dependencies**: A.0 (Complete)

## Objective
Design a comprehensive CLI interface for the reverse proxy that exposes all module capabilities while maintaining backward compatibility and ease of use.

## Key Questions to Answer
1. How should complex configurations be passed (CLI args vs config files)?
2. What's the right balance between CLI flags and configuration files?
3. How do we group related options for discoverability?
4. What shortcuts/aliases would improve developer experience?

## Deliverables
1. **CLI Design Document** (`analysis/cli-design-proposal.md`)
   - Complete argument specification
   - Option grouping and categories
   - Configuration precedence rules
   - Migration guide for existing users

2. **Example Commands**
   - Basic usage (current compatibility)
   - Medium complexity (multiple upstreams)
   - Advanced usage (full features)
   - Config file examples

## Process

### Step 1: Analyze Existing Patterns (30 min)
- Review other Shadowcat commands for consistency
- Identify common patterns and conventions
- Note any anti-patterns to avoid

### Step 2: Design Argument Structure (1 hour)
- Group related options
- Define short and long forms
- Specify value types and validation
- Consider mutual exclusivity

### Step 3: Config File Integration (1 hour)
- Define when config files are needed vs CLI args
- Design override mechanism
- Specify config file discovery

### Step 4: Document Examples (30 min)
- Create examples for each use case tier
- Show progression from simple to complex
- Include troubleshooting scenarios

## Design Constraints
- Must maintain 100% backward compatibility
- Should follow Unix philosophy (do one thing well)
- Complex configs should be file-based, not CLI
- Help text must be comprehensive

## Proposed Option Categories

### 1. Basic Connection
- `--bind` (existing)
- `--upstream` (existing, enhance for multiple)

### 2. Upstream Management
- `--upstream-file` (new)
- `--load-balancing` (new)
- `--health-check-interval` (new)

### 3. Security
- `--auth-config` (new)
- `--tls-cert` (new)
- `--tls-key` (new)

### 4. Resilience
- `--circuit-breaker` (new)
- `--retry-policy` (new)
- `--timeout` (new)

### 5. Observability
- `--enable-recording` (new)
- `--recording-dir` (new)
- `--audit-log` (new)

### 6. Performance
- `--connection-pool-size` (new)
- `--max-body-size` (new)
- `--enable-compression` (new)

## Example Usage Tiers

### Tier 1: Basic (Backward Compatible)
```bash
shadowcat reverse --bind 127.0.0.1:8080 --upstream http://localhost:3000
```

### Tier 2: Enhanced
```bash
shadowcat reverse \
  --bind 127.0.0.1:8080 \
  --upstream http://primary:3000 \
  --upstream http://secondary:3000 \
  --load-balancing round-robin \
  --enable-recording
```

### Tier 3: Production
```bash
shadowcat reverse \
  --config production.yaml \
  --enable-recording \
  --audit-log /var/log/shadowcat/audit.log
```

## Success Criteria
- [ ] All module capabilities accessible
- [ ] Backward compatibility verified
- [ ] Examples cover 80% of use cases
- [ ] Help text is clear and complete
- [ ] Config file schema defined

## Notes
- Consider using clap's derive API for better type safety
- Look at similar tools (nginx, haproxy, envoy) for inspiration
- Keep basic usage simple - complexity should be opt-in