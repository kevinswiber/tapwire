# Shadowcat MCP Code Extraction Inventory

## Purpose
Quick reference for what exists in shadowcat/src/mcp/ and how to leverage it during extraction. Keep this lightweight - discover details during implementation.

## What Can Be Extracted As-Is (Good candidates)

### Core Protocol Types
- `messages.rs` - MessageEnvelope, ProtocolMessage, Direction
- `types.rs` - JsonRpcId, SessionId, TransportType, MessageContext
- `constants.rs` - Protocol versions and constants
- `version.rs` - Version negotiation logic
- `validation.rs` - Message validation rules

### Builders & Parsers
- `builder.rs` - Request/Response/Notification builders
- `parser.rs` - McpParser, McpMessage parsing
- `early_parser.rs` - Minimal parsing for routing

### Protocol Logic
- `handshake.rs` - McpHandshake, capability negotiation
- `batch.rs` - Batch message handling
- `encoding.rs` - JSON-RPC encoding/decoding

## What Needs Refactoring (Modify for extraction)

### Handler System
- `handler.rs` - Currently tied to shadowcat's proxy model
- **Action**: Extract interface, make proxy-agnostic

### Correlation Engine  
- `correlation.rs` - Has proxy-specific timeout/cleanup logic
- **Action**: Extract core correlation, make configurable

### Version State
- `version_state.rs` - Stateful version tracking
- **Action**: Simplify for client/server use cases

## What Stays in Shadowcat (Proxy-specific)

### Proxy Features
- `event_id.rs` - EventIdGenerator for dual session tracking
- Proxy-specific error handling
- Session bridging logic
- Interceptor integration points

## Extraction Strategy

### Phase 1: Direct Extraction (Week 1)
Just copy these files to crates/mcp/:
- types.rs, messages.rs, constants.rs
- builder.rs, parser.rs
- Basic error types

### Phase 2: Refactor & Adapt (Week 2)
Extract with modifications:
- handler.rs → Create trait-based handler system
- handshake.rs → Separate client/server handshake logic
- correlation.rs → Make timeout/cleanup pluggable

### Phase 3: Transport Integration (Week 3)
New implementations using extracted core:
- stdio::Transport
- http::Transport with SSE support
- Transport trait abstraction

### Phase 4: Shadowcat Migration (Week 4)
- Replace shadowcat's mcp module with crate dependency
- Keep proxy-specific code in shadowcat
- Add interceptor hooks

## Key Insights

1. **~70% reusable** - Most protocol logic can be extracted
2. **Handler abstraction needed** - Current handler is proxy-coupled
3. **Transport agnostic** - Protocol code doesn't depend on transport
4. **Well-tested** - Existing tests can guide extraction

## Don't Over-Plan These
- Exact module structure (discover during extraction)
- Performance optimizations (measure first)
- Feature flags (add when needed)
- Detailed API design (evolve through use)

## Quick Win Opportunities
1. Start with types/messages - zero dependencies
2. Parser/builder next - only needs types
3. Test extraction early with validator tests
4. Keep shadowcat working throughout

---

*Created: 2025-08-24*
*Purpose: Lightweight guide for extraction, not detailed design*
*Key: Start simple, refactor as needed, don't over-engineer*