# LLM-Friendly Help Documentation Feature

## Problem Statement
The current CLI help output is designed for human consumption in terminal environments. LLMs and agents need structured, comprehensive documentation that includes:
- Complete command hierarchy
- All available options and flags
- Usage patterns and examples
- Output formats and expectations
- Context about what each command does

## Goals
1. **Primary**: Add `--help-doc` flag that outputs comprehensive, LLM-consumable documentation
2. **Support multiple formats**: Markdown (default), JSON, and Manpage
3. **Automation**: Generate documentation automatically from Clap command definitions
4. **Maintainability**: Minimize manual updates when CLI changes
5. **Completeness**: Include all information an LLM would need to use the CLI effectively

## Success Criteria
- [ ] `shadowcat --help-doc` outputs complete markdown documentation
- [ ] `shadowcat --help-doc=json` outputs structured JSON documentation
- [ ] `shadowcat --help-doc=manpage` outputs ROFF-formatted man page
- [ ] Documentation includes full command tree with all subcommands
- [ ] Each command includes: description, arguments, options, examples
- [ ] Documentation is generated automatically from Clap definitions
- [ ] No manual updates required when adding/changing commands
- [ ] Output is optimized for LLM consumption (clear structure, complete context)

## Phases

### Phase 1: Research & Analysis
- Investigate Clap's capabilities for introspection and documentation generation
- Research existing solutions (clap_mangen, clap_complete, custom approaches)
- Evaluate build-time vs runtime generation
- Design documentation schema for both formats

### Phase 2: Design & Architecture
- Design the documentation generation system
- Plan integration points with existing CLI structure
- Create schema for JSON output
- Design markdown template structure

### Phase 3: Implementation
- Implement core documentation generation logic
- Add `--help-doc` flag to root command
- Support both markdown and JSON formats
- Add examples to existing commands

### Phase 4: Testing & Refinement
- Test with various LLMs for usability
- Ensure all commands are documented
- Validate JSON schema
- Add integration tests

## Task Breakdown

| ID | Task | Duration | Status | Dependencies |
|----|------|----------|--------|--------------|
| A.0 | Research Clap documentation capabilities | 1-2h | Pending | None |
| A.1 | Analyze existing Shadowcat CLI structure | 1h | Pending | None |
| A.2 | Research LLM documentation best practices | 1h | Pending | None |
| A.3 | Evaluate build-time vs runtime generation | 1h | Pending | A.0 |
| B.0 | Design documentation schema | 2h | Pending | A.0-A.3 |
| B.1 | Design integration approach | 1h | Pending | A.1, B.0 |
| C.0 | Implement documentation generator | 3h | Pending | B.0, B.1 |
| C.1 | Add --help-doc flag | 1h | Pending | C.0 |
| C.2 | Implement markdown formatter | 2h | Pending | C.0 |
| C.3 | Implement JSON formatter | 2h | Pending | C.0 |
| C.4 | Implement manpage formatter | 2h | Pending | C.0 |
| C.5 | Add examples to commands | 2h | Pending | A.1 |
| D.0 | Test with LLMs | 2h | Pending | C.0-C.5 |
| D.1 | Add integration tests | 1h | Pending | C.0-C.4 |
| D.2 | Documentation and examples | 1h | Pending | D.0, D.1 |

## Risks & Mitigation
- **Risk**: Clap may not provide sufficient introspection capabilities
  - **Mitigation**: Fall back to macro-based generation or manual registration
- **Risk**: Build-time generation may complicate build process
  - **Mitigation**: Start with runtime generation, optimize later if needed
- **Risk**: Documentation may become too verbose for LLM context windows
  - **Mitigation**: Support filtering by command/subcommand

## Key Questions to Answer
1. Does Clap provide runtime introspection of command structure?
2. Can we extract examples and long descriptions from Clap?
3. Should documentation be generated at build-time (build.rs) or runtime?
4. What schema should the JSON format follow?
5. How do we handle dynamic commands or conditionally available features?

## References
- [Clap Documentation](https://docs.rs/clap/latest/clap/)
- [clap_mangen](https://docs.rs/clap_mangen/latest/clap_mangen/) - Man page generation
- [clap_complete](https://docs.rs/clap_complete/latest/clap_complete/) - Shell completion generation
- Build.rs documentation generation examples

## Notes
- Consider compatibility with common LLM tool-use patterns
- May want to include schema version for future compatibility
- Could potentially integrate with MCP tool definitions