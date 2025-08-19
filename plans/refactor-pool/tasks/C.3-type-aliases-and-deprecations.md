# Task {ID}: {Task Name}

## Objective

{Clear, concise statement of what this task aims to accomplish and why it's important}

## Background

{Context about the current state and why this task is needed. Include:
- Current problems or limitations
- How this task fits into the larger project
- What benefits this will provide}

## Key Questions to Answer

1. {Important question that needs resolution during this task}
2. {Design decision that needs to be made}
3. {Technical challenge to solve}
4. {Integration consideration}

## Step-by-Step Process

### 1. Analysis Phase ({X} min)
{Description of initial analysis needed}

```bash
# Commands to understand current state
cd {working_directory}
{command to explore codebase}
{command to find relevant patterns}
```

### 2. Design Phase ({X} min)
{Description of design decisions to make}

Key design considerations:
- {Consideration 1}
- {Consideration 2}
- {Consideration 3}

### 3. Implementation Phase ({X} hours)

#### 3.1 {First Component}
```rust
// Example code structure or pseudo-code
{code_example}
```

#### 3.2 {Second Component}
```rust
// Example code structure or pseudo-code
{code_example}
```

### 4. Testing Phase ({X} min)
```bash
# Commands to test implementation
cargo test {specific_tests}
cargo clippy --all-targets -- -D warnings
cargo fmt
```

Test cases to implement:
- [ ] {Test case 1}
- [ ] {Test case 2}
- [ ] {Test case 3}

### 5. Documentation Phase ({X} min)
- Update module documentation
- Add usage examples
- Update tracker with completion status

## Expected Deliverables

### New Files
- `{path/to/new/file.rs}` - {Description of what this file contains}
- `{path/to/another/file.rs}` - {Description}

### Modified Files
- `{path/to/existing/file.rs}` - {What changes are made}
- `{path/to/another/existing.rs}` - {What changes}

### Tests
- `{tests/test_file.rs}` - {What is tested}
- Minimum {X}% code coverage for new code
- All tests passing

### Documentation
- Rustdoc comments for all public APIs
- Usage examples in module documentation
- Updated README if applicable

## Success Criteria Checklist

- [ ] {Primary functional requirement met}
- [ ] {Secondary functional requirement met}
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Code formatted with cargo fmt
- [ ] Documentation complete
- [ ] Performance targets met (if applicable)
- [ ] Backward compatibility maintained (if applicable)
- [ ] Tracker updated with completion status

## Risk Assessment

| Risk | Impact | Mitigation | 
|------|--------|------------|
| {Risk description} | {HIGH/MEDIUM/LOW} | {How to mitigate} |
| {Another risk} | {Impact level} | {Mitigation strategy} |

## Duration Estimate

**Total: {X} hours**
- Analysis: {X} minutes
- Design: {X} minutes
- Implementation: {X} hours
- Testing: {X} minutes
- Documentation: {X} minutes

## Dependencies

- {Dependency 1 - must be completed first}
- {Dependency 2 - required component}
- None (if no dependencies)

## Integration Points

- **{Component A}**: {How this task integrates}
- **{Component B}**: {Integration considerations}

## Performance Considerations

- {Performance requirement or consideration}
- {Memory usage consideration}
- {Latency requirement}

## Notes

- {Important implementation note}
- {Design decision rationale}
- {Future consideration}

## Commands Reference

```bash
# Quick reference of useful commands for this task
cd {working_directory}

# Development
{dev_command_1}
{dev_command_2}

# Testing
{test_command_1}
{test_command_2}

# Validation
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo test --quiet
```

## Example Implementation

```rust
// Optional: Provide a concrete example of the expected implementation
// This helps guide the implementation and serves as documentation
{example_code}
```

## Follow-up Tasks

After completing this task, consider:
- {Potential improvement}
- {Related task that could be done}
- {Optimization opportunity}

---

**Task Status**: ⬜ Not Started
**Created**: {Date}
**Last Modified**: {Date}
**Author**: {Author}