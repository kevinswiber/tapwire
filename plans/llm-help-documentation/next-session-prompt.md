# Next Session: LLM Help Documentation Implementation

## Project Context

Implement a `--help-doc` flag for Shadowcat that outputs comprehensive, LLM-consumable documentation in multiple formats (Markdown, JSON, and Manpage). This feature enables LLMs and agents to understand the complete CLI interface without manual documentation updates.

**Project**: LLM-Friendly Help Documentation
**Tracker**: `plans/llm-help-documentation/llm-help-documentation-tracker.md`
**Status**: Phase A - Research & Analysis (25% Complete)

## Current Status

### What Has Been Completed
- **A.0: Research Clap Capabilities** (✅ Completed)
  - Validated Clap 4.x runtime introspection capabilities
  - Identified Command::get_subcommands() and metadata extraction methods
  - Created prototype demonstrating feasibility

### What's In Progress
- **Phase A: Research & Analysis** (1 of 4 tasks complete)
  - Duration: 3 hours remaining
  - Dependencies: None

## Your Mission

Complete the LLM Help Documentation feature in a single focused session, delivering a working `--help-doc` flag that generates comprehensive CLI documentation in multiple formats.

### Priority 1: Complete Analysis & Design (2 hours)

1. **Analyze CLI Structure** (1h)
   - Map all Shadowcat commands and subcommands
   - Document argument types and options
   - Identify examples to include
   
2. **Design Documentation System** (1h)
   - Create JSON schema for structured output
   - Design Markdown template hierarchy
   - Plan integration with existing CLI

### Priority 2: Implement Core Feature (2 hours)

1. **Build Documentation Generator** (1.5h)
   - Recursive command tree walker
   - Metadata extraction from Clap
   - Format-agnostic internal representation
   
2. **Add Format Handlers** (0.5h)
   - Markdown formatter
   - JSON formatter
   - Basic manpage support

## Essential Context Files to Read

1. **Primary Tracker**: `plans/llm-help-documentation/llm-help-documentation-tracker.md` - Full project context
2. **Research Results**: `plans/llm-help-documentation/analysis/` - Clap capabilities and approach
3. **CLI Implementation**: `shadowcat/src/main.rs` - Current CLI structure
4. **CLI Commands**: `shadowcat/src/cli/` - Command definitions

## Working Directory

```bash
cd shadowcat
```

## Commands to Run First

```bash
# Verify current CLI structure
cargo run -- --help

# Check existing command tree
cargo run -- forward --help
cargo run -- reverse --help

# Run tests to ensure baseline
cargo test --lib
cargo clippy --all-targets -- -D warnings
```

## Implementation Strategy

### Phase 1: Analysis (30 min)
1. Read existing CLI implementation in `src/main.rs` and `src/cli/`
2. Map command hierarchy and options
3. Review analysis documents for Clap approach

### Phase 2: Design (30 min)
1. Define JSON schema for command documentation
2. Create internal representation struct
3. Design integration point in CLI

### Phase 3: Implementation (2 hours)
1. Create `src/cli/doc_gen.rs` module
2. Implement command tree walker using Clap introspection
3. Add format handlers (Markdown, JSON)
4. Integrate `--help-doc` flag in main CLI

### Phase 4: Testing & Polish (30 min)
1. Test with actual LLM tools (Claude, GPT-4)
2. Add integration tests
3. Update README with examples
4. Run formatters and linters

## Success Criteria Checklist

- [ ] `shadowcat --help-doc` generates complete Markdown documentation
- [ ] `shadowcat --help-doc=json` outputs valid JSON structure
- [ ] All commands and subcommands included
- [ ] Each command has description, args, options, examples
- [ ] Tests passing with no warnings
- [ ] Documentation updated in README

## Key Commands

```bash
# Development commands
cargo build
cargo run -- --help-doc
cargo run -- --help-doc=json

# Testing commands
cargo test cli::doc_gen
cargo test --doc

# Validation commands
cargo fmt
cargo clippy --all-targets -- -D warnings
```

## Important Notes

- **Use Clap's runtime APIs** - Command::get_subcommands(), get_arguments(), etc.
- **Keep it simple** - Start with runtime generation, optimize later if needed
- **Test with real LLMs** - Ensure output is actually useful for AI assistants
- **Follow existing patterns** - Match the style of other CLI modules

## Key Design Considerations

1. **Completeness**: Must capture ALL commands, flags, and options
2. **Structure**: Maintain hierarchical command relationships
3. **Compatibility**: Output should work with multiple LLM providers
4. **Performance**: Keep generation fast (<100ms)

## Risk Factors & Blockers

- **Clap API limitations**: Mitigated - prototype shows sufficient capabilities
- **Output verbosity**: Plan filtering options if needed
- **Format compatibility**: Test with multiple LLMs during development

## Next Steps After This Task

This is a self-contained feature that can be completed in one session. After completion:
- Update main README with usage examples
- Consider future enhancements (filtering, caching, MCP tool definitions)
- Move to next priority: Better CLI Interface

## Session Time Management

**Estimated Session Duration**: 4-5 hours
- Setup & Context: 30 min
- Analysis & Design: 1 hour  
- Implementation: 2 hours
- Testing & Polish: 30 min
- Documentation: 30 min

---

**Session Goal**: Deliver working `--help-doc` flag with Markdown and JSON output formats

**Last Updated**: 2025-08-14
**Next Review**: After implementation complete