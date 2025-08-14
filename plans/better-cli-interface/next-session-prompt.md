# Next Session: CLI Interface Analysis & Design

## Project Context

Improving Shadowcat's CLI interface to provide a more intuitive developer experience through smart auto-detection and replacing the confusing "reverse proxy" terminology with the clearer "gateway" concept.

**Project**: Better CLI Interface
**Tracker**: `plans/better-cli-interface/better-cli-interface-tracker.md`
**Status**: Phase A - Analysis & Design (0% Complete)

## Current Status

### What Has Been Completed
- Plan structure created
- Tracker document established
- Task files defined for Phase A

### What's In Progress
- **Phase A**: Analysis & Design (Not Started)
  - Duration: 9 hours total
  - Dependencies: None

## Your Mission

Analyze the current Shadowcat CLI implementation and design an improved interface that makes common cases magical while keeping explicit control available. The key insight: keep "forward" as-is but replace "reverse" with the more intuitive "gateway" terminology.

### Priority 1: Analysis Tasks (4 hours)

1. **A.0: Current CLI Analysis** (2h)
   - Map existing command structure in main.rs
   - Document all command variations and options
   - Trace execution flow from CLI to proxy
   - Success: Complete analysis document in `analysis/current-cli-structure.md`
   
2. **A.1: User Experience Research** (2h)
   - Study CLI patterns from ngrok, docker, git, psql
   - Identify successful auto-detection strategies
   - Extract design principles
   - Success: Research document in `analysis/cli-patterns-research.md`

### Priority 2: Design Proposal (3 hours)

**A.2: Design Proposal** (3h)
- Synthesize findings into cohesive design
- Define command structure and auto-detection logic
- Create help text templates
- Success: Complete proposal in `analysis/cli-design-proposal.md`

## Essential Context Files to Read

1. **Primary Tracker**: `plans/better-cli-interface/better-cli-interface-tracker.md` - Full project context
2. **Task Details**: 
   - `plans/better-cli-interface/tasks/A.0-current-cli-analysis.md`
   - `plans/better-cli-interface/tasks/A.1-user-experience-research.md`
   - `plans/better-cli-interface/tasks/A.2-design-proposal.md`
3. **Implementation**: `shadowcat/src/main.rs` - Current CLI implementation
4. **Architecture**: `shadowcat/src/proxy/` - Proxy implementations

## Working Directory

```bash
cd /Users/kevin/src/tapwire
```

## Commands to Run First

```bash
# Enter shadowcat directory
cd shadowcat

# Verify current state
cargo check

# See current CLI help
cargo run -- --help
cargo run -- forward --help
cargo run -- reverse --help  # This will become "gateway"

# Check for any existing issues
cargo clippy --all-targets -- -D warnings
```

## Implementation Strategy

### Phase 1: Analysis (2 hours)
1. Read and understand current main.rs
2. Map all command enums and structures
3. Document current command flow
4. Identify all hardcoded strings and dependencies

### Phase 2: Research (2 hours)
1. Study other CLI tools for patterns
2. Identify what makes CLIs intuitive
3. Document best practices
4. Extract applicable patterns for Shadowcat

### Phase 3: Design (3 hours)
1. Create command hierarchy design
2. Define auto-detection heuristics
3. Design help system structure
4. Plan migration strategy

### Phase 4: Documentation (30 min)
1. Create all analysis documents
2. Update tracker with findings
3. Prepare implementation plan (A.3) if time permits

## Success Criteria Checklist

- [ ] Current CLI fully documented
- [ ] At least 5 CLI tools researched
- [ ] Design proposal complete
- [ ] Auto-detection logic defined
- [ ] Migration path planned
- [ ] All analysis documents created
- [ ] Tracker updated with progress

## Key Commands

```bash
# Development commands
cd shadowcat
cargo build
cargo run -- forward stdio -- echo hello
cargo run -- reverse --port 8080  # Will become: cargo run -- gateway --port 8080

# Analysis commands
grep -r "enum.*Command" src/
grep -r "clap::" src/
grep -r "forward\\|reverse" src/

# Testing commands
cargo test cli
cargo clippy --all-targets -- -D warnings
```

## Important Notes

- **Always use TodoWrite tool** to track progress through tasks
- **Start with examining existing code** to avoid breaking changes
- **Focus on user experience** over technical elegance
- **Keep auto-detection simple** and predictable
- **Document all findings** in the analysis/ directory
- **Consider backward compatibility** in all decisions

## Key Design Considerations

1. **Simplicity**: Make the common case magical, not clever
2. **Predictability**: Users should understand what will happen
3. **Clarity**: "Gateway" is more intuitive than "reverse proxy"
4. **Discoverability**: Help text should guide users naturally
5. **Extensibility**: Design should accommodate future transports

## Risk Factors & Blockers

- **Complex Dependencies**: Current code might have hidden coupling
- **Test Dependencies**: Many tests might depend on "reverse" command name
- **User Workflows**: This is a breaking change - need clear migration messaging

## Next Steps After This Task

Once Phase A is complete:
- **A.3**: Implementation Plan (2 hours, depends on A.2)
- **Phase B**: Core Implementation (12 hours total)
  - B.1: Refactor CLI Module Structure
  - B.2: Implement Smart Detection
  - B.3: Update to Forward/Gateway Commands
  - B.4: Update Help System

## Model Usage Guidelines

- **IMPORTANT**: This is primarily an analysis and design session. If approaching context limits during implementation phases, create a new session focused on specific implementation tasks.

## Session Time Management

**Estimated Session Duration**: 7-9 hours
- Setup & Context: 15 min
- Analysis (A.0): 2 hours
- Research (A.1): 2 hours
- Design (A.2): 3 hours
- Documentation: 30 min
- Buffer: 1-2 hours

---

**Session Goal**: Complete Phase A analysis and design, producing comprehensive documentation that will guide the implementation of an intuitive CLI interface.

**Last Updated**: 2025-01-14
**Next Review**: After A.2 completion