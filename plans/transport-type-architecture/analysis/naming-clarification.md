# Naming Clarification: ProxyCore vs UnifiedProxy

**Created**: 2025-08-16  
**Purpose**: Clarify the distinction between UnifiedProxy (analysis concept) and ProxyCore (implementation)

## The Confusion

During the analysis phase, we used "UnifiedProxy" to describe a long-term vision of unified proxy architecture. During the design phase, we named the actual implementation "ProxyCore". This created confusion about whether these are the same or different concepts.

## The Clarification

**They represent the same concept at different stages:**

### UnifiedProxy (Analysis Phase Concept)
- Appeared in directional-transport-analysis.md
- Represented the "long-term vision" 
- Conceptual idea of unified proxy handling
- Theoretical end state where proxies share logic

### ProxyCore (Design Phase Implementation)
- Appears in architecture-proposal.md
- The actual implementation we're building
- Shared pipeline logic used by both proxy types
- Practical abstraction that eliminates duplication

## The Actual Architecture

What we're **actually building**:

```rust
// Shared core logic (what we call ProxyCore)
pub struct ProxyCore {
    session_manager: Arc<SessionManager>,
    interceptor_chain: Arc<InterceptorChain>,
    transport_factory: Arc<TransportFactory>,
}

// Forward proxy uses the core
pub struct ForwardProxy {
    core: ProxyCore,
    // Forward-specific fields if any
}

// Reverse proxy uses the core
pub struct ReverseProxy {
    core: ProxyCore,
    auth_gateway: Option<AuthGateway>,     // Reverse-specific
    sse_resilience: SseResilience,         // Reverse-specific
}
```

What we're **NOT building**:

```rust
// Single unified proxy class (not our approach)
pub struct UnifiedProxy {
    mode: ProxyMode,  // Forward or Reverse
    // All logic in one class
}
```

## Why ProxyCore is Better

1. **Composition over Inheritance**: Each proxy type composes ProxyCore
2. **Specialization Allowed**: Reverse proxy can have auth gateway, forward doesn't need it
3. **Clear Boundaries**: Shared logic vs specific logic is explicit
4. **Gradual Migration**: Can migrate incrementally

## Key Distinction

- **ProxyCore**: Shared pipeline logic (the "how" of proxying)
- **ForwardProxy/ReverseProxy**: Distinct proxy types (the "what" and "when")

## Implementation Impact

### Phase 3 Tasks
When implementing Phase 3, we will:
1. Create `ProxyCore` struct with shared pipeline
2. Refactor `ForwardProxy` to use `ProxyCore`
3. Refactor `ReverseProxy` to use `ProxyCore`
4. Each proxy keeps its unique features

### What's Shared (in ProxyCore)
- Message pipeline (receive → process → forward)
- Interceptor chain execution
- Session management
- Error handling patterns
- Metrics collection

### What's Specific (in each proxy)
- **ForwardProxy**: Client transport acceptance
- **ReverseProxy**: HTTP server, auth gateway, SSE resilience

## Conclusion

"UnifiedProxy" was an analysis-phase concept that evolved into "ProxyCore" during design. ProxyCore is the better name because it accurately represents what we're building: shared core logic that both proxy types use, not a single unified proxy class.

Going forward, we should:
1. Use "ProxyCore" consistently in documentation
2. Avoid "UnifiedProxy" to prevent confusion
3. Be clear that proxies remain distinct types
4. Emphasize that ProxyCore is shared logic, not a complete proxy