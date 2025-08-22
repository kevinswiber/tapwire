# Traffic Recording - Next Session Prompt

## Session Focus
Begin Phase A: Analysis and Design of the traffic recording feature.

## Tasks for This Session
1. **A.0**: Analyze current recording implementation (2h)
2. **A.1**: Research tape format requirements (2h)
3. **A.2**: Design storage architecture (2h)

## Context
You are implementing a comprehensive traffic recording system for Shadowcat, an MCP proxy. The system needs to capture all MCP traffic flowing through the proxy with minimal performance impact.

## Key Files to Review
- `shadowcat/src/tape.rs` - Current tape implementation
- `shadowcat/src/proxy/mod.rs` - Proxy architecture
- `shadowcat/src/protocol/mcp.rs` - MCP message handling
- `shadowcat/src/storage/` - Storage backends

## Deliverables
1. **Analysis document** (`plans/traffic-recording/analysis/current-state.md`)
   - Current tape.rs capabilities and limitations
   - Performance characteristics
   - Integration points with proxy

2. **Format specification** (`plans/traffic-recording/analysis/tape-formats.md`)
   - JSONL format structure
   - Binary format considerations
   - Compression options evaluation

3. **Storage design** (`plans/traffic-recording/analysis/storage-architecture.md`)
   - Storage backend options
   - Retention and rotation strategies
   - Metadata indexing approach

## Commands to Run
```bash
# Analyze current implementation
rg -A5 "impl.*Tape" shadowcat/src/
rg "record" shadowcat/src/ --type rust

# Check existing tests
cargo test tape -- --nocapture

# Review MCP message structures
rg "MessageFrame" shadowcat/src/
```

## Success Criteria
- [ ] Complete understanding of current tape.rs implementation
- [ ] Clear specification for tape formats
- [ ] Documented storage architecture with trade-offs
- [ ] All analysis documents created in `analysis/` directory
- [ ] Key decisions documented in tracker

## Notes
- Focus on understanding before implementing
- Consider both performance and usability
- Document all findings for future reference
- Update tracker with progress and decisions