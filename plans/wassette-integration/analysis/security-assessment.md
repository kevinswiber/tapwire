# Wassette-Shadowcat Security Assessment

## Wassette Security Model

### WebAssembly Sandbox
- **Runtime**: Wasmtime with browser-grade isolation
- **Memory Model**: Linear memory isolation per instance
- **No Ambient Authority**: Components start with zero permissions
- **Capability-Based**: Explicit grants required for resources

### Permission System
```yaml
# Wassette Policy Format
version: "1.0"
permissions:
  storage:
    - uri: "fs://workspace/**"
      access: [read, write]
  network:
    allow:
      - host: "api.example.com"
      - cidr: "10.0.0.0/8"
  environment:
    - key: "API_KEY"
```

### Enforcement Layers
1. **Parse Time**: Policy validation on load
2. **Link Time**: WASI capabilities configured
3. **Runtime**: Wasmtime enforces boundaries
4. **System Calls**: Trapped and validated

### Component Verification
- **Signing**: Support for Notation and Cosign
- **Registry**: OCI standard with signatures
- **Loading**: Verification before execution
- **Updates**: Policy-controlled component updates

## Shadowcat Security Model

### Authentication Gateway
- **Protocol**: OAuth 2.1 compliance
- **Token Types**: JWT with validation
- **PKCE**: Proof Key for Code Exchange
- **Token Isolation**: Never forwarded upstream

### Authorization Layers
1. **Transport**: TLS termination
2. **Session**: Authenticated session tracking
3. **Message**: Per-request authorization
4. **Resource**: Fine-grained access control

### Proxy Security
- **Origin Validation**: CORS and DNS rebinding protection
- **Rate Limiting**: Configurable per-client limits
- **Audit Trail**: Complete request logging
- **Interception**: Policy-based filtering

## Combined Security Architecture

### Trust Boundaries

```
┌─────────────────────────────────────────────┐
│                   Client                     │
│                                              │
└────────────────────┬────────────────────────┘
                     │ HTTPS/TLS
        ┌────────────▼────────────┐
        │   Shadowcat Proxy       │ Trust Boundary 1
        │   - Authentication      │ (Client Auth)
        │   - Rate Limiting       │
        │   - Audit Logging       │
        └────────────┬────────────┘
                     │ stdio (local)
        ┌────────────▼────────────┐
        │   Wassette Runtime      │ Trust Boundary 2
        │   - Policy Engine       │ (Resource Access)
        │   - WASI Capabilities   │
        └────────────┬────────────┘
                     │ Sandboxed
        ┌────────────▼────────────┐
        │   WebAssembly Component │ Trust Boundary 3
        │   - Memory Isolation    │ (Execution)
        │   - Resource Limits     │
        └─────────────────────────┘
```

### Token Flow Design

#### Principle: Token Isolation
```rust
// Shadowcat strips auth tokens before forwarding
impl SecurityProxy {
    async fn forward_request(&self, req: Request) -> Request {
        let mut forwarded = req.clone();
        forwarded.headers.remove("Authorization");
        forwarded.headers.remove("Cookie");
        forwarded
    }
}
```

#### Token Boundaries
1. **Client → Shadowcat**: OAuth 2.1 tokens
2. **Shadowcat → Wassette**: No tokens (local trust)
3. **Wassette → Component**: Capability tokens only
4. **Component → External**: Per-policy credentials

### Capability Preservation

#### Challenge
Ensuring proxy doesn't bypass Wassette's capabilities

#### Solution
```rust
// Shadowcat respects Wassette policies
impl PolicyAwareProxy {
    async fn check_capability(&self, component: &str, resource: &str) -> bool {
        // Query Wassette for component's capabilities
        let policy = self.get_component_policy(component).await?;
        policy.allows(resource)
    }
}
```

## Threat Analysis

### Attack Vectors

#### 1. Privilege Escalation via Proxy
**Threat**: Proxy grants more permissions than policy allows
**Mitigation**: 
- Read-only policy enforcement
- Wassette validates all requests
- Audit log for policy violations

#### 2. Token Leakage
**Threat**: Client tokens exposed to components
**Mitigation**:
- Strict token stripping at proxy
- Separate auth domains
- Token rotation and expiry

#### 3. Component Escape
**Threat**: Malicious component breaks sandbox
**Mitigation**:
- Wasmtime's proven sandbox
- Regular security updates
- Component signing verification

#### 4. Resource Exhaustion
**Threat**: Component consumes excessive resources
**Mitigation**:
- WASI resource limits
- Proxy-level rate limiting
- Timeout enforcement

#### 5. Supply Chain Attack
**Threat**: Malicious components from registries
**Mitigation**:
- Signature verification
- Registry authentication
- Policy-controlled sources

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Sandbox Escape | Low | Critical | Wasmtime updates, monitoring |
| Token Leakage | Medium | High | Token isolation, audit |
| Policy Bypass | Low | High | Dual enforcement |
| Resource DoS | Medium | Medium | Limits and quotas |
| Supply Chain | Medium | High | Signing, verification |

## Security Requirements Matrix

| Requirement | Wassette | Shadowcat | Combined System | Implementation |
|-------------|----------|-----------|-----------------|----------------|
| **Authentication** | N/A | OAuth 2.1 | OAuth at proxy | Shadowcat handles auth |
| **Authorization** | Policy-based | Token-based | Layered | Both systems enforce |
| **Sandboxing** | Wasmtime | N/A | Preserved | Direct passthrough |
| **Token Management** | N/A | JWT validation | Isolated | Strip at boundary |
| **Audit Logging** | Component calls | All traffic | Comprehensive | Unified log stream |
| **Rate Limiting** | N/A | Per-client | Applied | Proxy enforcement |
| **Component Verification** | Signatures | N/A | End-to-end | Wassette verifies |
| **Network Isolation** | WASI caps | TLS/Origin | Defense-in-depth | Both layers |
| **Resource Limits** | WASI limits | Timeouts | Combined | Dual enforcement |

## Audit and Monitoring

### Audit Requirements

#### Events to Log
1. **Authentication**: All auth attempts
2. **Component Load**: Source, signature, policy
3. **Policy Changes**: Who, what, when
4. **Tool Invocations**: Component, params, results
5. **Security Violations**: Denied requests, policy violations

#### Log Format
```json
{
  "timestamp": "2025-01-10T10:00:00Z",
  "event_type": "tool_invocation",
  "session_id": "uuid",
  "component": "fetch-rs",
  "tool": "fetch",
  "policy_check": "passed",
  "duration_ms": 45,
  "result": "success"
}
```

### Monitoring Metrics
- Authentication success/failure rate
- Component invocation frequency
- Policy violation attempts
- Resource usage per component
- Latency by operation type

## Best Practices

### Deployment Guidelines

1. **Network Isolation**
   - Run Wassette on localhost only
   - Use Shadowcat for network exposure
   - Separate auth and execution domains

2. **Policy Management**
   - Version control all policies
   - Review before deployment
   - Principle of least privilege
   - Regular audit of permissions

3. **Component Vetting**
   - Verify signatures always
   - Test in sandbox first
   - Monitor behavior patterns
   - Regular security scanning

4. **Token Hygiene**
   - Short-lived tokens
   - Rotation on schedule
   - Never log tokens
   - Separate token stores

### Configuration Hardening

```yaml
# Shadowcat Security Config
security:
  auth:
    require_authentication: true
    token_expiry: 3600
    refresh_enabled: true
  
  transport:
    tls_required: true
    min_tls_version: "1.3"
    
  limits:
    max_request_size: 10MB
    request_timeout: 30s
    rate_limit: 100/min

# Wassette Security Config
permissions:
  default_policy: "deny-all"
  require_signed_components: true
  allow_unsigned_local: false
```

### Incident Response

1. **Detection**
   - Anomaly detection in audit logs
   - Resource usage alerts
   - Policy violation notifications

2. **Response**
   - Automatic component unload
   - Session termination
   - Rate limit enforcement

3. **Recovery**
   - Component rollback
   - Policy restoration
   - Session cleanup

## Security Recommendations

### Critical
1. ✅ Always strip auth tokens at proxy boundary
2. ✅ Never weaken Wassette's capability model
3. ✅ Implement comprehensive audit logging
4. ✅ Verify component signatures

### Important
1. ⚠️ Regular security updates for both systems
2. ⚠️ Monitor for abnormal behavior patterns
3. ⚠️ Implement rate limiting at proxy
4. ⚠️ Use TLS for all network communication

### Nice to Have
1. 💡 Implement component behavior analysis
2. 💡 Add honey token detection
3. 💡 Create security dashboards
4. 💡 Automated policy compliance checks

## Conclusion

The combined Wassette-Shadowcat system provides defense-in-depth security through:
- **Layer 1**: Authentication and rate limiting at proxy
- **Layer 2**: Policy-based capability control
- **Layer 3**: WebAssembly sandbox isolation

Key principles for secure integration:
1. Maintain strict token isolation
2. Preserve Wassette's capability model
3. Implement comprehensive auditing
4. Follow least-privilege principle

The architecture provides strong security guarantees suitable for both development and production deployments.