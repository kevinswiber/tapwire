# Transport vs Upstream Analysis - TODO

## Context
During the refactor planning, we identified potential overlap between:
- `transport/` module (core transport abstractions)
- `proxy::reverse::upstream/` module (reverse proxy's upstream handling)

## Key Questions to Analyze

### 1. What can proxy::reverse reuse from transport?
- [ ] `transport::stdio::StdioTransport` - Can we use this directly?
- [ ] `transport::http` - HTTP client functionality
- [ ] `transport::sse` - SSE parsing and streaming (already identified for reuse)
- [ ] Buffer pooling from `transport::buffer_pool`
- [ ] Connection pooling abstractions
- [ ] Transport error types and handling

### 2. What might move from proxy::reverse to transport?
- [ ] Generic upstream selection logic (load balancing strategies)
- [ ] Connection health checking patterns
- [ ] Request/response transformation utilities
- [ ] Header manipulation utilities (hop-by-hop removal, etc.)
- [ ] Retry logic and circuit breakers

### 3. Current Duplication to Eliminate
- [ ] Stdio subprocess handling (likely duplicated)
- [ ] HTTP client creation and configuration
- [ ] SSE parsing (should use transport::sse)
- [ ] Message serialization/deserialization
- [ ] Error handling patterns

## Proposed Analysis Approach

### Phase 1: Inventory
1. List all transport-related functionality in `proxy::reverse::legacy.rs`
2. Map to existing `transport/` module capabilities
3. Identify gaps and overlaps

### Phase 2: Refactor Strategy
1. Determine what stays reverse-proxy specific
2. Identify what can be generalized to transport
3. Plan migration order to avoid breaking changes

### Phase 3: Implementation
1. First, make proxy::reverse use transport where possible
2. Then, extract generic patterns from proxy::reverse to transport
3. Finally, remove all duplication

## Design Principles

### What belongs in `transport/`
- Generic transport implementations (stdio, HTTP, SSE, WebSocket)
- Protocol-agnostic message passing
- Connection management primitives
- Buffer and resource pooling
- Transport-level error handling

### What belongs in `proxy::reverse::upstream/`
- Reverse proxy specific logic:
  - Upstream selection strategies
  - Session-aware routing
  - Request forwarding policies
  - Response aggregation
- Thin adapters that compose transport primitives

## Specific Areas to Investigate

### StdioTransport Usage
```rust
// Current in legacy.rs:
process_via_stdio_pooled(...)

// Should potentially be:
let transport = transport::stdio::StdioTransport::new(...);
let response = transport.send(message).await?;
```

### HTTP Client Reuse
```rust
// Current in legacy.rs:
process_via_http_hyper(...)

// Should potentially leverage:
transport::http::HttpTransport or similar
```

### SSE Streaming
```rust
// Current plan:
upstream/http/sse_adapter.rs uses transport::sse::SseParser

// Validate:
- Is SseParser sufficient?
- What reverse-proxy specific logic is needed?
- Can we push any improvements back to transport::sse?
```

## Action Items

1. **Before refactor**: Quick analysis of transport module capabilities
2. **During refactor**: Use transport where possible, note gaps
3. **After refactor**: Propose transport enhancements based on findings

## Notes

- This overlap is expected and healthy - it means our transport abstractions are useful
- The goal is DRY (Don't Repeat Yourself) while maintaining clear boundaries
- Reverse proxy can have proxy-specific logic while leveraging transport primitives
- Some duplication might be intentional for different use cases (forward vs reverse proxy)

## Decision Log

**Date: 2025-01-18**
- Identified overlap during architecture review
- Decided to proceed with refactor but note areas for consolidation
- Will use transport::sse immediately
- Will analyze stdio/http overlap during implementation