# Task C.3: Implement Format Handlers

## Objective

Implement formatters that convert the internal documentation representation into the target output formats: Markdown, JSON, and Manpage.

## Background

Each format has specific requirements:
- **Markdown**: Hierarchical headings, code blocks, tables
- **JSON**: Valid JSON with proper escaping
- **Manpage**: ROFF format for man pages

## Key Questions to Answer

1. How do we handle format-specific features?
2. What's the optimal structure for each format?
3. How do we ensure valid output?
4. Should we pretty-print or minimize?

## Step-by-Step Process

### 1. Implementation Phase (1 hour)

#### Markdown Formatter
```rust
// src/cli/doc_gen/formats/markdown.rs
impl Documentation {
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        
        // Title and description
        output.push_str(&format!("# {}\n\n", self.name));
        output.push_str(&format!("{}\n\n", self.description));
        
        // Global options
        output.push_str("## Global Options\n\n");
        // ...
        
        // Commands
        output.push_str("## Commands\n\n");
        for cmd in &self.commands {
            self.format_command(&mut output, cmd, 2);
        }
        
        output
    }
}
```

#### JSON Formatter
```rust
// src/cli/doc_gen/formats/json.rs
impl Documentation {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}
```

#### Manpage Formatter (Basic)
```rust
// src/cli/doc_gen/formats/manpage.rs
impl Documentation {
    pub fn to_manpage(&self) -> String {
        // Basic ROFF format
        format!(
            ".TH {} 1\n.SH NAME\n{}\n.SH SYNOPSIS\n...",
            self.name.to_uppercase(),
            self.description
        )
    }
}
```

### 2. Testing Phase (30 min)

Test each format:
- Valid output generation
- Format-specific features
- Edge cases (special characters, etc.)
- LLM compatibility

## Expected Deliverables

### New Files
- `src/cli/doc_gen/formats/mod.rs` - Format module
- `src/cli/doc_gen/formats/markdown.rs` - Markdown formatter
- `src/cli/doc_gen/formats/json.rs` - JSON formatter
- `src/cli/doc_gen/formats/manpage.rs` - Manpage formatter

### Tests
- Format-specific tests
- Output validation
- Example generation

## Success Criteria Checklist

- [ ] Markdown formatter complete
- [ ] JSON formatter complete
- [ ] Basic manpage support
- [ ] Valid output for all formats
- [ ] Tests passing
- [ ] Examples generated

## Duration Estimate

**Total: 1.5 hours**
- Implementation: 1 hour
- Testing: 30 minutes

## Dependencies

- C.1: Implement Core Generator

## Notes

- Start with Markdown and JSON
- Manpage can be basic initially
- Consider using existing crates for formatting

---

**Task Status**: ⬜ Not Started
**Created**: 2025-08-14
**Author**: Shadowcat Team