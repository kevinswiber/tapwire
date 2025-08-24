# MCP Compliance Analysis Documents

## 🔥 Active Documents (Use These)

### Core Architecture & Decisions
- **[../TRANSPORT-ARCHITECTURE-FINAL.md](../TRANSPORT-ARCHITECTURE-FINAL.md)** - ⭐ **FINAL transport architecture decision**
- **[../CURRENT-ARCHITECTURE.md](../CURRENT-ARCHITECTURE.md)** - Overall system architecture
- **[../DECISION-LOG.md](../DECISION-LOG.md)** - Historical decision record
- **[websocket-separation-decision.md](websocket-separation-decision.md)** - WebSocket as separate transport
- **[gpt-findings-analysis.md](gpt-findings-analysis.md)** - Critical bugs and fixes needed

### Implementation Guides
- **[mcp-core-extraction-architecture.md](mcp-core-extraction-architecture.md)** - How to build the MCP library
- **[shadowcat-mcp-extraction-inventory.md](shadowcat-mcp-extraction-inventory.md)** - What code to extract
- **[shadowcat-transport-session-inventory.md](shadowcat-transport-session-inventory.md)** - Transport/session code
- **[implementation-considerations.md](implementation-considerations.md)** - Risks and mitigation
- **[compliance-independence-design.md](compliance-independence-design.md)** - Testing independence

### Testing & Compliance
- **[mcp-compliance-checklist.md](mcp-compliance-checklist.md)** - 233 spec requirements
- **[proxy-specific-test-scenarios.md](proxy-specific-test-scenarios.md)** - 50 proxy-specific tests
- **[test-requirement-coverage-matrix.md](test-requirement-coverage-matrix.md)** - Gap analysis (12% coverage!)
- **[validator-test-catalog.md](validator-test-catalog.md)** - 54 validator tests + 28 gaps
- **[client-server-proxy-separation.md](client-server-proxy-separation.md)** - Test separation strategy

### Research & Analysis
- **[mcp-validator-findings.md](mcp-validator-findings.md)** - Critical bugs in Python validator
- **[shadowcat-proxy-validation.md](shadowcat-proxy-validation.md)** - Proof Shadowcat works
- **[protocol-version-matrix.md](protocol-version-matrix.md)** - Version differences
- **[version-agnostic-architecture.md](version-agnostic-architecture.md)** - Pluggable version design

## 📦 Deprecated Documents (Historical Reference Only)

These documents trace our journey to the final architecture. They're preserved for context but **should NOT be used for implementation**.

### Transport Architecture Evolution
- ~~`transport-architecture-investigation.md`~~ → Initial exploration
- ~~`transport-architecture-final.md`~~ → First decision (AsyncRead/Write)
- ~~`transport-architecture-final-v2.md`~~ → Second iteration (early Sink/Stream)
- ~~`transport-decision-summary.md`~~ → Partial summary
- ~~`transport-investigation-summary.md`~~ → Investigation notes
- ~~`transport-patterns-analysis.md`~~ → RMCP pattern exploration
- ~~`transport-deviation-analysis.md`~~ → Deviation analysis
- ~~`framed-sink-stream-architecture.md`~~ → Merged into FINAL
- ~~`rmcp-vs-framed-comparison.md`~~ → Analysis complete
- ~~`http-sse-unified-transport.md`~~ → Merged into FINAL
- ~~`http-transport-unified-architecture.md`~~ → Merged into FINAL
- ~~`websocket-transport-design.md`~~ → Superseded by separation decision

### Other Deprecated
- ~~`build-vs-buy-analysis.md`~~ → Decision made (build our own)
- ~~`architectural-decisions.md`~~ → Merged into FINAL
- ~~`library-architecture-design.md`~~ → Implemented in Phase B

## 📝 Quick Reference

### For New Developers - Start Here
1. **[../TRANSPORT-ARCHITECTURE-FINAL.md](../TRANSPORT-ARCHITECTURE-FINAL.md)** - Current architecture
2. **[gpt-findings-analysis.md](gpt-findings-analysis.md)** - Critical bugs to fix
3. **[mcp-core-extraction-architecture.md](mcp-core-extraction-architecture.md)** - How to build

### For Implementers - Code Locations
- **Transport code**: `crates/mcp/src/transport/`
- **Client/Server**: `crates/mcp/src/{client,server}.rs`
- **Session management**: `src/session/`
- **Existing transport traits**: `src/transport/`

### For Testers - Test Strategy
- [mcp-compliance-checklist.md](mcp-compliance-checklist.md) - What to test
- [proxy-specific-test-scenarios.md](proxy-specific-test-scenarios.md) - Proxy tests
- [test-requirement-coverage-matrix.md](test-requirement-coverage-matrix.md) - Coverage gaps

## 🚨 Critical Issues (from GPT-5 Review)

1. **Client Deadlock** - `request()` blocks forever without `run()`, but `run()` consumes self
2. **HTTP Transport Broken** - Doesn't send actual HTTP requests, just shuffles queues
3. **WebSocket Needs Separation** - Must be separate transport, not HTTP sub-mode
4. **Codec Needs Hardening** - JsonLineCodec needs CRLF handling, overlong line support

See [gpt-findings-analysis.md](gpt-findings-analysis.md) for details.

## Key Findings Summary

### Architecture Decision ✅
- **Sink/Stream at message level** - Correct abstraction
- **Framed for line protocols** - stdio, subprocess, TCP, Unix
- **HTTP adaptive** - JSON response or SSE stream
- **WebSocket separate** - Different lifecycle and requirements

### What We Have ✅
- Basic transport structure implemented
- JsonLineCodec working (needs hardening)
- StdioTransport and SubprocessTransport functional
- Good session infrastructure in `src/session/`

### What's Broken 🔴
- Client concurrency deadlock
- HTTP transport doesn't work
- No WebSocket transport yet
- Codec not robust enough

### Next Steps 📋
1. Fix Client deadlock (CRITICAL)
2. Implement HTTP worker pattern (CRITICAL)
3. Create WebSocket transport
4. Harden JsonLineCodec
5. Wire version negotiation

---

*Last Updated: 2025-08-24*  
*Status: Architecture finalized, critical bugs identified*