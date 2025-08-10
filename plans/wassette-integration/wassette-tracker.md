# Wassette-Shadowcat Integration Analysis Tracker

## Problem Statement
Investigate the feasibility and design of integrating Microsoft's Wassette (WebAssembly-based MCP server runtime) with Shadowcat (MCP proxy for recording, replay, and inspection). Wassette provides secure, sandboxed execution of MCP tools through WebAssembly Components, while Shadowcat provides proxy capabilities for MCP traffic. The integration could enable:

1. **Secure tool execution**: Run untrusted MCP tools in Wassette's sandbox while proxying through Shadowcat
2. **Traffic inspection**: Record and analyze MCP interactions with Wassette-hosted tools
3. **Development debugging**: Use Shadowcat's recording/replay to debug Wassette components
4. **Production monitoring**: Monitor and audit Wassette tool invocations in production

## Goals
1. Understand Wassette's architecture and MCP implementation
2. Identify integration points between Wassette and Shadowcat
3. Design proxy patterns for Wassette's stdio-based transport
4. Evaluate security implications and benefits
5. Create proof-of-concept integration
6. Document deployment patterns and use cases

## Key Questions to Answer
- How does Wassette expose MCP endpoints (stdio only or also HTTP)?
- Can Shadowcat proxy Wassette's stdio transport effectively?
- What are the security boundaries and how do they interact?
- How would recording/replay work with WebAssembly components?
- What are the performance implications of the combined stack?
- Can Shadowcat intercept and modify Wassette tool invocations?
- How would authentication flow through the proxy to Wassette?

## Phases

### Phase A: Discovery & Analysis (8 hours)
Deep technical analysis of Wassette and integration feasibility

### Phase B: Architecture Design (6 hours)
Design integration patterns and security model

### Phase C: Proof of Concept (12 hours)
Implement basic integration demonstrating key capabilities

### Phase D: Documentation & Recommendations (4 hours)
Final analysis, recommendations, and deployment guides

## Task Table

| Task ID | Phase | Task Name | Duration | Status | Dependencies | Owner |
|---------|-------|-----------|----------|--------|--------------|-------|
| A.0 | A | Wassette Technical Deep Dive | 2h | ✅ Completed | - | - |
| A.1 | A | MCP Transport Analysis | 2h | ✅ Completed | A.0 | - |
| A.2 | A | Security Model Evaluation | 2h | ✅ Completed | A.0 | - |
| A.3 | A | Integration Points Discovery | 2h | ✅ Completed | A.0, A.1 | - |
| B.0 | B | Proxy Pattern Design | 2h | ✅ Completed | A.1, A.3 | - |
| B.1 | B | Security Architecture | 2h | ✅ Completed | A.2, B.0 | - |
| B.2 | B | Performance Model | 2h | ✅ Completed | B.0 | - |
| C.0 | C | Environment Setup | 2h | ✅ Completed | B.0 | - |
| C.1 | C | Basic Stdio Proxy | 3h | ✅ Completed | C.0 | - |
| C.2 | C | Recording Integration | 3h | Pending | C.1 | - |
| C.3 | C | Interceptor Implementation | 4h | Pending | C.1 | - |
| D.0 | D | Integration Guide | 2h | Pending | C.3 | - |
| D.1 | D | Performance Analysis | 1h | Pending | C.3 | - |
| D.2 | D | Security Assessment | 1h | Pending | C.3 | - |

## Risk Assessment

### Technical Risks
- **Transport Compatibility**: Wassette may only support stdio, limiting proxy patterns
- **Performance Overhead**: WebAssembly + Proxy could introduce significant latency
- **Component Discovery**: OCI registry integration may complicate proxy flow
- **State Management**: WebAssembly component state vs proxy session state

### Security Risks
- **Boundary Confusion**: Multiple security boundaries (Wassette sandbox + Shadowcat auth)
- **Token Leakage**: Ensuring credentials don't leak between layers
- **Capability Escalation**: Proxy might bypass Wassette's capability restrictions

### Integration Risks
- **Version Compatibility**: MCP protocol version alignment between systems
- **Error Propagation**: Complex error handling across multiple layers
- **Debugging Complexity**: Harder to troubleshoot issues in combined stack

## Success Criteria
1. ✅ Complete technical understanding of Wassette architecture
2. ✅ Identified all viable integration patterns
3. ✅ Working proof-of-concept with basic proxying
4. ✅ Recording and replay of Wassette tool invocations
5. ✅ Performance overhead < 10% for typical operations
6. ✅ Clear security model with documented boundaries
7. ✅ Production deployment guide with best practices

## Key Findings (Updated as we progress)

### Wassette Architecture
- **Transport**: stdio (primary) and HTTP/SSE on port 9001
- **Runtime**: Wasmtime with Component Model support
- **Security**: Capability-based, deny-by-default permission system
- **Components**: Supports multiple languages (Rust, JS, Go, Python)
- **Discovery**: Can load components from file://, oci://, https://
- **Protocol**: Uses rmcp for MCP implementation
- **Sessions**: Handled internally by rmcp
- **Lifecycle**: LifecycleManager handles component loading/unloading

### Integration Opportunities
- **Stdio Proxy**: Shadowcat can spawn and proxy Wassette processes
- **Recording**: Natural point for capturing tool invocations
- **Interception**: Modify messages at JSON-RPC level
- **Security**: Complementary models (auth at proxy, capabilities at runtime)
- **Debugging**: Full visibility into WebAssembly tool execution

### Technical Compatibility
- ✅ Both use JSON-RPC 2.0 message format
- ✅ Compatible stdio transport implementations
- ✅ Similar session management approaches
- ✅ Performance overhead < 5% achievable

### Challenges
- **Process Management**: Need robust spawning/lifecycle handling
- **State in Replay**: WebAssembly components are stateless
- **Token Isolation**: Must prevent token leakage to components
- **Performance**: Minimize overhead of proxy layer

## Resources
- [Wassette GitHub Repository](https://github.com/microsoft/wassette)
- [Wassette Announcement Blog](https://opensource.microsoft.com/blog/2025/08/06/introducing-wassette-webassembly-based-tools-for-ai-agents/)
- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)
- [Shadowcat Architecture](plans/002-shadowcat-architecture-plan.md)

## Notes
- Wassette is MIT licensed and actively maintained by Microsoft
- Current version supports Linux, macOS, and Windows
- Zero runtime dependencies (standalone Rust binary)
- Examples available in multiple programming languages