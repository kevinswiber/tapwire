# Next Session: Batch Support Analysis Phase

## Context
Shadowcat currently has inconsistent batch message support. Some components reject batches while the Phase 3 protocol layer added full batch support. We need to analyze the situation and make an informed decision about whether to fully implement or remove batch support.

## Current Status
- Phase 0: Analysis ⬜ Ready to start
- Conflicting implementations identified
- Plan created for systematic investigation

## Next Tasks (Analysis Phase - 5h total)

### A.0: MCP Specification Analysis (2h)
From file: `plans/full-batch-support/tasks/A.0-mcp-spec-analysis.md`
- Review MCP spec for batch requirements
- Check JSON-RPC 2.0 batch semantics
- Analyze reference implementations
- Document findings

### A.1: Code Inventory (3h)
From file: `plans/full-batch-support/tasks/A.1-code-inventory.md`
- Find all batch-related code
- Document current behavior
- Identify components needing changes
- Assess test coverage

## Key Files to Review
- `src/interceptor/batch_handler.rs` - Current batch rejection
- `src/proxy/handlers.rs:~350` - Proxy batch checking
- `src/transport/protocol/mod.rs:138-193` - New batch support
- `src/transport/stdio.rs` - Transport array handling

## Session Goal
Complete thorough analysis to make an informed decision about batch support. The deliverables should provide clear evidence for whether to:
1. Fully implement batch support (complex but future-proof)
2. Remove batch support entirely (simpler, MCP-compliant)
3. Partial support with limitations (middle ground)

## Deliverables
- `analysis/mcp-batch-requirements.md` - Specification findings
- `analysis/batch-code-inventory.md` - Current code state
- `analysis/batch-decision.md` - Recommendation with rationale

## Commands to Start
```bash
# Check current batch references
rg -i "batch" src/ --stats

# Find array handling
rg "is_array|as_array" src/ --type rust

# Run existing batch tests
cargo test batch
```

## Success Criteria
- [ ] Clear understanding of MCP requirements
- [ ] Complete inventory of batch-related code
- [ ] Well-reasoned recommendation
- [ ] Evidence-based decision document

## Important Context
- MCP v2025-11-05 doesn't explicitly require batch support
- JSON-RPC 2.0 allows but doesn't mandate batches
- Current proxy explicitly rejects batches
- Phase 3 added batch support to protocol layer only

**Last Updated**: 2025-08-14
**Session Time**: 5 hours estimated
**Next Phase**: Either Implementation (20h) or Removal (3h) based on decision