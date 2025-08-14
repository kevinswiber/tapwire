# Task D.3: Documentation & Examples

## Objective

Create comprehensive documentation and examples showing how to use the new --help-doc feature, ensuring developers and LLMs can effectively utilize it.

## Background

Documentation needs to cover:
- Feature overview
- Usage instructions
- Format descriptions
- Integration examples
- LLM usage patterns

## Key Questions to Answer

1. What examples best demonstrate the feature?
2. How do we document LLM integration?
3. What format-specific guidance is needed?
4. Where should documentation live?

## Step-by-Step Process

### 1. Documentation Phase (20 min)

#### Update README
```markdown
## LLM-Friendly Documentation

Shadowcat provides comprehensive documentation for LLM consumption:

```bash
# Generate Markdown documentation (default)
shadowcat --help-doc

# Generate JSON documentation
shadowcat --help-doc=json

# Generate man page
shadowcat --help-doc=manpage
```

### Using with LLMs

Provide the generated documentation to your LLM:

```python
# Example with OpenAI
import subprocess
import openai

docs = subprocess.check_output(['shadowcat', '--help-doc=json'])
client.chat.completions.create(
    model="gpt-4",
    messages=[
        {"role": "system", "content": f"CLI Documentation: {docs}"},
        {"role": "user", "content": "How do I forward stdio?"}
    ]
)
```
```

### 2. Examples Phase (10 min)

Create example scripts:
- Python integration example
- Shell script example
- LLM prompt templates

## Expected Deliverables

### Documentation
- Updated README.md
- Examples directory
- Integration guide

### Examples
- `examples/llm_integration.py`
- `examples/doc_usage.sh`
- `examples/prompts.md`

## Success Criteria Checklist

- [ ] README updated
- [ ] Usage examples created
- [ ] LLM integration documented
- [ ] Format descriptions added
- [ ] Examples tested

## Duration Estimate

**Total: 30 minutes**
- Documentation: 20 minutes
- Examples: 10 minutes

## Dependencies

- D.1: Test with LLMs
- D.2: Add Integration Tests

## Notes

- Include real-world use cases
- Show both simple and advanced usage
- Document any limitations

---

**Task Status**: ⬜ Not Started
**Created**: 2025-08-14
**Author**: Shadowcat Team