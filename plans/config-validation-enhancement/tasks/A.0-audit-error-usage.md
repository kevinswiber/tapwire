# Task A.0: Audit Current Error Usage

## Objective
Identify all patterns of error construction in the config validation code to understand what specific error variants we need.

## Key Questions
1. What validation errors are most common?
2. Which errors would benefit most from specific variants?
3. What contextual information is currently being lost?
4. Are there patterns we can group together?

## Process

### Step 1: Find All Error Constructions
```bash
# Find all Error::Invalid patterns
grep -r "Error::Invalid" src/config --include="*.rs" -n

# Find all Error::MissingField patterns
grep -r "Error::MissingField" src/config --include="*.rs" -n

# Find all error construction patterns
grep -r "return Err(" src/config --include="*.rs" -B1 -A1
```

### Step 2: Categorize Error Patterns
Group errors into categories:
- Port/Address validation
- Resource limits
- Rate limiting configuration
- TLS/Security settings
- Session/timeout configuration
- Compatibility checks

### Step 3: Identify Lost Context
For each error, note what context is being converted to strings:
- Numeric values (ports, limits)
- Validation ranges
- Conflicting settings
- System requirements

### Step 4: Document Frequency
Count how often each pattern appears to prioritize which variants to create first.

## Deliverables

### Location: `analysis/current-error-patterns.md`

Document should contain:
```markdown
# Current Error Patterns Analysis

## Summary Statistics
- Total error constructions: X
- Error::Invalid uses: Y
- Error::MissingField uses: Z
- Other patterns: W

## Common Patterns

### Port Validation (X occurrences)
```rust
// Current pattern
Error::Invalid(format!("Invalid port in server bind address '{}': {}", addr, e))

// Lost context: port number, specific error type
```

### Address Validation (Y occurrences)
...

## Categorization

### High-Priority Patterns (>5 occurrences)
1. Port validation
2. Address parsing
3. ...

### Medium-Priority Patterns (2-5 occurrences)
1. Rate limit configuration
2. ...

### Low-Priority Patterns (1 occurrence)
1. ...

## Recommendations
Based on frequency and impact, create specific variants for:
1. InvalidPort
2. InvalidAddress
3. RateLimitError
4. ...
```

## Success Criteria
- [ ] All error constructions in config module identified
- [ ] Patterns categorized by type and frequency
- [ ] Recommendations for specific error variants documented
- [ ] Context loss documented for each pattern

## Time Estimate
1 hour

## Dependencies
None - this is the starting task

## Notes
- Focus on src/config/validator.rs as it contains most validation
- Also check loader.rs for parsing errors
- Look for places where numeric errors are converted to strings