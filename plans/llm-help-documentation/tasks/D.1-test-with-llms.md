# Task D.1: Test with LLMs

## Objective

Validate that the generated documentation is actually useful and parseable by real LLMs, ensuring the feature meets its intended purpose.

## Background

Testing with actual LLMs is crucial to verify:
- Documentation completeness
- Format compatibility
- Parsing reliability
- Practical utility

## Key Questions to Answer

1. Can LLMs parse the generated documentation?
2. Do they understand the command structure?
3. Can they generate valid commands from the docs?
4. Are there any parsing errors or ambiguities?
5. Which format works best for each LLM?

## Step-by-Step Process

### 1. Preparation Phase (10 min)

Generate documentation:
```bash
# Generate all formats
cargo run -- --help-doc > shadowcat-help.md
cargo run -- --help-doc=json > shadowcat-help.json
cargo run -- --help-doc=manpage > shadowcat.1
```

### 2. Testing Phase (40 min)

Test with multiple LLMs:

#### Claude Testing
- Provide generated documentation
- Ask to explain available commands
- Request example command generation
- Test command validation

#### GPT-4 Testing
- Same test suite
- Note any differences

#### Other LLMs (if available)
- Gemini
- Local models (Ollama, etc.)

### 3. Analysis Phase (10 min)

Document findings:
- Parsing success rates
- Format preferences
- Common issues
- Improvement suggestions

## Expected Deliverables

### Test Report
- `analysis/llm-testing-results.md` - Testing outcomes

### Findings
- Compatibility matrix
- Format recommendations
- Issue list
- Success examples

## Success Criteria Checklist

- [ ] Tested with Claude
- [ ] Tested with GPT-4
- [ ] Documentation parsed successfully
- [ ] Commands generated correctly
- [ ] Issues documented
- [ ] Recommendations made

## Duration Estimate

**Total: 1 hour**
- Preparation: 10 minutes
- Testing: 40 minutes
- Analysis: 10 minutes

## Dependencies

- C.1-C.3: Implementation complete

## Notes

- Focus on practical use cases
- Test both simple and complex commands
- Verify subcommand understanding

---

**Task Status**: ⬜ Not Started
**Created**: 2025-08-14
**Author**: Shadowcat Team