# HTTP Forward Proxy Research Plan for Shadowcat

## Executive Summary

This research plan outlines the approach to extend Shadowcat's forward proxy capabilities to support HTTP-based MCP servers, including both standard HTTP request/response and Server-Sent Events (SSE) transports. Currently, Shadowcat only supports stdio-based MCP servers.

## Current State Analysis

### What Works
- **Stdio Transport**: Fully implemented with `StdioTransport` that spawns subprocesses
- **Forward Proxy Architecture**: Generic `ForwardProxy` that works with any `Transport` trait implementation
- **Session Management**: Complete session tracking and recording infrastructure
- **Interceptor Chain**: Flexible message interception and modification framework

### Gaps Identified
1. **HTTP Transport Limitations**:
   - Current `HttpTransport` only supports simple request/response
   - No SSE support for streaming responses
   - Missing bidirectional communication patterns

2. **CLI Limitations**:
   - `intercept` command only supports stdio servers
   - HTTP forward proxy command exists but is not implemented
   - No support for specifying transport type in intercept mode

3. **Protocol Version Support**:
   - Currently hardcoded to version `2025-11-05`
   - Need to support `2025-06-18` and potentially `2025-03-26`

## Research Objectives

### Phase 1: Protocol Understanding
1. **MCP HTTP Transport Variants**
   - Standard HTTP (request/response)
   - Streamable HTTP with SSE
   - Differences in message flow patterns

2. **Version Compatibility**
   - Differences between protocol versions
   - Header requirements per version
   - Backward compatibility strategies

### Phase 2: Technical Investigation
1. **SSE Implementation Strategy**
   - Rust libraries for SSE (e.g., `axum-sse`, `tokio-stream`)
   - Connection management for long-lived streams
   - Error recovery and reconnection

2. **Bidirectional Communication**
   - How to handle server-initiated requests over HTTP
   - Message correlation between requests and responses
   - Session state management

3. **Transport Abstraction Enhancement**
   - Extending `Transport` trait for streaming
   - Handling different message patterns (sync vs async)
   - Unified interface for stdio and HTTP variants

### Phase 3: Architecture Design
1. **Enhanced Transport Architecture**
   - `HttpTransport` vs `HttpSseTransport` vs unified approach
   - Connection pooling and reuse
   - Load balancing considerations

2. **Proxy Pattern Updates**
   - Handling asymmetric transports (stdio client → HTTP server)
   - Message buffering for SSE streams
   - Timeout and keepalive strategies

3. **CLI Integration**
   - Transport selection in commands
   - Configuration for HTTP endpoints
   - Authentication and security

## Detailed Research Areas

### 1. MCP Streamable HTTP Deep Dive
- [ ] Message format over SSE
- [ ] Event types and structure
- [ ] Connection lifecycle management
- [ ] Error handling and recovery

### 2. Implementation Patterns
- [ ] Existing Rust SSE client/server patterns
- [ ] Integration with Tokio async runtime
- [ ] Message serialization/deserialization
- [ ] Stream multiplexing strategies

### 3. Security Considerations
- [ ] Origin header validation
- [ ] DNS rebinding protection
- [ ] Token forwarding prevention
- [ ] TLS/mTLS support

### 4. Performance Analysis
- [ ] SSE overhead vs stdio
- [ ] Connection pooling benefits
- [ ] Message batching opportunities
- [ ] Latency characteristics

### 5. Testing Strategy
- [ ] Mock HTTP/SSE servers for testing
- [ ] Integration test scenarios
- [ ] Performance benchmarks
- [ ] Chaos testing for network issues

## Research Deliverables

### Week 1 Deliverables
1. **Technical Specification** (`technical-spec.md`)
   - Detailed HTTP/SSE transport design
   - Message flow diagrams
   - API specifications

2. **Implementation Plan** (`implementation-plan.md`)
   - Code changes required
   - Module structure
   - Migration strategy

3. **Testing Plan** (`testing-plan.md`)
   - Test scenarios
   - Mock server design
   - Performance criteria

### Week 2 Deliverables
1. **Proof of Concept**
   - Basic SSE client implementation
   - Integration with existing proxy
   - Performance measurements

2. **Risk Assessment**
   - Technical challenges
   - Performance implications
   - Backward compatibility issues

## Key Questions to Answer

1. **Transport Selection**
   - Should we have separate `HttpTransport` and `HttpSseTransport` or a unified implementation?
   - How to handle transport negotiation?

2. **Message Correlation**
   - How to correlate requests/responses in SSE streams?
   - Session management across transport types?

3. **Error Handling**
   - How to handle SSE reconnection transparently?
   - Fallback strategies for network issues?

4. **Performance**
   - Expected latency overhead?
   - Resource usage for long-lived connections?

5. **Security**
   - Authentication token handling?
   - Proxy authentication vs server authentication?

## Research Methodology

1. **Code Analysis**
   - Review existing transport implementations
   - Analyze rmcp library capabilities
   - Study similar proxy implementations

2. **Prototype Development**
   - Build minimal SSE client
   - Test with real MCP servers
   - Measure performance characteristics

3. **Documentation Review**
   - MCP specification details
   - SSE protocol standards
   - Security best practices

4. **Community Engagement**
   - Check MCP community examples
   - Review existing implementations
   - Gather feedback on approach

## Success Criteria

1. Clear understanding of HTTP/SSE transport requirements
2. Detailed design for implementation
3. Identified risks and mitigation strategies
4. Performance baseline established
5. Testing strategy defined

## Timeline

- **Day 1-2**: Protocol research and analysis
- **Day 3-4**: Technical investigation and prototyping
- **Day 5-6**: Architecture design and documentation
- **Day 7**: Review and finalization

## Next Steps

1. Begin with MCP specification deep dive
2. Create simple SSE client prototype
3. Test against known MCP HTTP servers
4. Document findings iteratively