# Task B.1: Design Documentation Schema

## Objective

Design comprehensive schemas for the documentation output formats, ensuring they capture all necessary information while remaining compatible with LLM consumption patterns.

## Background

We need well-defined schemas for:
- JSON format (structured data)
- Markdown format (hierarchical text)
- Internal representation (format-agnostic)

These schemas must be complete, consistent, and optimized for LLM parsing.

## Key Questions to Answer

1. What fields are required vs optional?
2. How do we represent command hierarchies?
3. What metadata should be included?
4. How do we handle examples and defaults?
5. Should we version the schema?

## Step-by-Step Process

### 1. Design Phase (30 min)

Define JSON schema:
```json
{
  "version": "1.0",
  "name": "shadowcat",
  "description": "...",
  "commands": [
    {
      "name": "forward",
      "description": "...",
      "arguments": [...],
      "options": [...],
      "subcommands": [...],
      "examples": [...]
    }
  ],
  "global_options": [...]
}
```

### 2. Validation Phase (20 min)

Ensure schema:
- Captures all CLI information
- Compatible with OpenAPI/JSON Schema
- Parseable by major LLMs
- Supports future extensions

### 3. Documentation Phase (10 min)

Document:
- Schema specification
- Field descriptions
- Example outputs
- Version strategy

## Expected Deliverables

### Schema Files
- `src/cli/doc_gen/schema.rs` - Rust structs for internal representation
- `analysis/json-schema.json` - JSON Schema specification

### Documentation
- Schema field descriptions
- Example outputs for each format
- Migration/versioning strategy

## Success Criteria Checklist

- [ ] JSON schema defined
- [ ] Markdown structure defined
- [ ] Internal representation designed
- [ ] Examples created
- [ ] Compatibility verified
- [ ] Extensibility considered

## Duration Estimate

**Total: 1 hour**
- Design: 30 minutes
- Validation: 20 minutes
- Documentation: 10 minutes

## Dependencies

- A.0-A.3: Analysis tasks

## Notes

- Consider MCP tool definition compatibility
- Plan for schema evolution
- Keep output size reasonable

---

**Task Status**: ⬜ Not Started
**Created**: 2025-08-14
**Author**: Shadowcat Team