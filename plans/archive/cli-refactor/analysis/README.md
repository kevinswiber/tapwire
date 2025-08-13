# CLI Refactor Analysis

This directory contains the analysis outputs from the CLI refactoring project for Shadowcat.

## Analysis Documents

### [main-components.md](main-components.md)
Complete inventory of all components in main.rs including:
- Command structure and variants
- Handler functions with line counts
- Helper utilities
- Configuration structures
- Code patterns

### [dependencies.md](dependencies.md)
Detailed dependency analysis showing:
- Shared dependencies across commands
- Command-specific requirements
- Coupling issues
- Common patterns
- Duplication identification

### [opportunities.md](opportunities.md)
Prioritized refactoring opportunities:
- High priority extractions for maximum impact
- Medium priority improvements
- Code quality enhancements
- Migration strategy
- Expected outcomes

## Key Findings

### Current State
- **main.rs size**: 1294 lines (not 1568 as initially estimated)
- **Commands**: 7 top-level (forward, reverse, record, replay, tape, intercept, session)
- **Already modularized**: tape, intercept, session commands
- **Major duplication**: Rate limiter setup (3x), ProxyConfig handling (4x)

### Major Issues
1. **Code duplication**: Rate limiting setup repeated 3 times
2. **Configuration coupling**: ProxyConfig tightly coupled to CLI args
3. **Large functions**: Several handlers over 100 lines
4. **Test implementation**: Forward stdio has incomplete test code
5. **Mixed responsibilities**: main.rs handles parsing, execution, and utilities

### Refactoring Targets
- **Goal**: Reduce main.rs to < 200 lines
- **Extract**: All command execution logic to dedicated modules
- **Centralize**: Shared configuration and utilities
- **Maintain**: Exact CLI compatibility

## Next Steps

1. **Design module boundaries** (Task A.2)
   - Define interfaces for each command module
   - Plan common utilities module
   - Design configuration management

2. **Create migration strategy** (Task A.3)  
   - Plan incremental extraction
   - Define testing approach
   - Set up rollback plan

3. **Begin implementation** (Phase 2)
   - Start with common module
   - Extract commands one by one
   - Add tests throughout

## Usage

These analysis documents serve as the foundation for the refactoring effort. They should be referenced when:
- Designing new module interfaces
- Prioritizing work items
- Making architectural decisions
- Validating refactoring completeness

---

Generated: 2025-01-09
Phase: Analysis (A.1)