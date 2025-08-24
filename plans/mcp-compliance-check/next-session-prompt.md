# Next Session: Begin MCP Library Extraction

## Session Goal
Start Phase B by extracting core MCP protocol types from shadowcat into a reusable library.

## Context
- **Tracker**: `plans/mcp-compliance-check/mcp-compliance-check-tracker.md`
- **Current Phase**: B - MCP Library Extraction (Ready to start)
- **Previous Work**: Completed all analysis and architecture (Phase A)
- **Existing Code**: ~70% of MCP protocol already in shadowcat/src/mcp/

## What We've Learned
1. **Most code is reusable** - shadowcat has mature MCP implementation
2. **Architecture decided** - Single MCP crate, hybrid Client/Server design
3. **Transport organization** - `mcp::transports::http::streaming::sse`
4. **Extraction strategy** - Start with types, then builders/parsers, then transports

## Primary Task: B.0 - Extract Core Types and Messages (2 hours)

### Extraction Strategy: Copy-First Approach ✨
**Important**: We're copying code to create a clean MCP crate. Shadowcat stays unchanged for now.
- **Copy** files from shadowcat/src/mcp/ to crates/mcp/src/
- **Clean** the API without worrying about backward compatibility  
- **Design** for ideal usage, not current shadowcat patterns
- **Later** (Phase H) we'll integrate shadowcat with the new crate

### What to Copy
From `shadowcat/src/mcp/`:
- `types.rs` → Core types (JsonRpcId, SessionId, MessageContext)
- `messages.rs` → Protocol messages (MessageEnvelope, ProtocolMessage)
- `constants.rs` → Protocol constants and versions
- `version.rs` → Version negotiation logic

### Steps
1. **Create crate structure**:
   ```bash
   mkdir -p shadowcat/crates/mcp/src
   cd shadowcat/crates/mcp
   ```

2. **Set up Cargo.toml**:
   ```toml
   [package]
   name = "mcp"
   version = "0.1.0"
   edition = "2021"
   
   [dependencies]
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   thiserror = "1.0"
   ```

3. **Copy and refactor files**:
   - Copy files from shadowcat/src/mcp/
   - Remove shadowcat-specific code
   - Simplify APIs where possible
   - Add clear documentation
   - Keep protocol-pure functionality

4. **Add to workspace** in shadowcat/Cargo.toml:
   ```toml
   [workspace]
   members = [".", "crates/mcp"]  # Note: Don't need crates/compliance yet
   ```

5. **Test standalone**:
   ```bash
   cd crates/mcp
   cargo check
   cargo test
   ```

### If Time Permits: Start B.1 - Extract Builders and Parsers (3 hours)
From `shadowcat/src/mcp/`:
- `builder.rs` → Message builders (RequestBuilder, ResponseBuilder)
- `parser.rs` → Message parsing (McpParser)
- `validation.rs` → Message validation

These depend on B.0 types but are otherwise independent.

### Success Criteria for This Session
- [ ] MCP crate created and added to workspace
- [ ] Core types extracted and compile standalone
- [ ] No dependencies on shadowcat internals
- [ ] Basic unit tests pass
- [ ] Can import types from MCP crate in shadowcat

## Key References
- **Architecture Decisions**: `analysis/architectural-decisions.md` - WHY we made choices
- **Extraction Guide**: `analysis/mcp-core-extraction-architecture.md` - HOW to build
- **MCP Inventory**: `analysis/shadowcat-mcp-extraction-inventory.md` - WHAT to extract
- **Transport Inventory**: `analysis/shadowcat-transport-session-inventory.md` - Infrastructure code

## Target Structure (Single MCP Crate)

```
shadowcat/
├── src/                    # Shadowcat (will use MCP crate)
├── crates/
│   ├── mcp/               # Extracted MCP library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs         # This session: Extract
│   │       ├── messages.rs      # This session: Extract
│   │       ├── constants.rs     # This session: Extract
│   │       ├── version.rs       # This session: Extract
│   │       ├── builder.rs       # Next: If time
│   │       ├── parser.rs        # Next: If time
│   │       ├── client.rs        # Future: B.3
│   │       ├── server.rs        # Future: B.4
│   │       ├── interceptor.rs   # Future: C.1
│   │       └── transports/      # Future: B.2, C.0
│   │           ├── mod.rs
│   │           ├── stdio.rs
│   │           └── http/
│   │               └── streaming/
│   │                   └── sse.rs
│   └── compliance/        # Future: Phase D
│       └── Cargo.toml
```

## Important Notes
- **Copy, don't refactor shadowcat** - Leave shadowcat unchanged
- **Design freedom** - Create the API you wish shadowcat had
- **Start minimal** - Just get types compiling first
- **No over-engineering** - Simple extraction, refactor later
- **Test standalone** - MCP crate should work independently
- **Clear commits** - One commit per extraction step

## Benefits of Copy-First Approach
- ✅ **No risk** to shadowcat - it keeps working
- ✅ **Clean API** - design without legacy constraints
- ✅ **Faster progress** - no simultaneous refactoring
- ✅ **Better testing** - validate MCP crate independently
- ✅ **Flexibility** - can evolve API based on compliance needs

## Definition of Done
- [ ] Core types extracted and compile in MCP crate
- [ ] MCP crate added to workspace
- [ ] No shadowcat dependencies in extracted code  
- [ ] Shadowcat can import from MCP crate
- [ ] Basic tests demonstrate functionality
- [ ] Tracker updated with completion status

## Next Steps After This Session
- B.1: Extract builders and parsers (3h)
- B.2: Create Transport trait and stdio (4h)
- B.3: Build Client struct (3h)
- B.4: Build Server struct (3h)
- Then Phase C: Additional components (HTTP/SSE, interceptors, etc.)

---

**Duration**: 2-3 hours for B.0
**Focus**: Extract core MCP types
**Deliverables**: Standalone MCP crate with core types

*Last Updated: 2025-08-24*
*Ready for: MCP library extraction*