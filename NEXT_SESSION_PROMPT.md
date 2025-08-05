# Phase 5B Day 4+: Policy Engine Integration - Next Session Prompt

Use this comprehensive prompt to continue Shadowcat development in your next Claude session:

---

# Continue Shadowcat Phase 5B Day 4+: Policy Engine Integration

I'm implementing Shadowcat, an MCP (Model Context Protocol) proxy in Rust. Phase 5B Days 1-3 are complete with OAuth 2.1, JWT validation, and enhanced AuthGateway. Now I need to continue with Day 4+ Policy Engine Integration.

## Current Status
- **Repository:** `/Users/kevin/src/tapwire/shadowcat` (git submodule in `/Users/kevin/src/tapwire`)
- **Phase:** Phase 5B Day 4+ - Policy Engine Integration and HTTP-specific conditions
- **Previous Work:** Days 1-3 complete (OAuth 2.1 + JWT + AuthGateway enhancement)
- **Tests:** 232 passing (73 auth module tests)
- **Achievement:** < 5ms authentication overhead with comprehensive middleware

## What's Complete (Phase 5B Days 1-3)

### ✅ Day 1: OAuth 2.1 Foundation & PKCE
- Complete OAuth 2.1 client with mandatory PKCE (S256 method)
- `src/auth/oauth.rs` - OAuth2Config and client implementation
- `src/auth/pkce.rs` - Secure PKCE generation and validation
- Full OAuth 2.1 compliance with cryptographically secure generation

### ✅ Day 2: JWT Token Validation with JWKS
- High-performance JWT validation (< 1ms with cache hits)
- `src/auth/token.rs` - TokenValidator with JWKS integration
- Automatic key rotation with 5-minute TTL caching
- Algorithm support: RS256, RS384, RS512, ES256, ES384

### ✅ Day 3: AuthGateway Enhancement (Just Completed)
- Enhanced `src/auth/gateway.rs` with session management and caching
- Complete `src/auth/middleware.rs` - Axum middleware suite
- Token refresh flows with secure session-to-token mapping
- < 5ms authentication pipeline with intelligent caching
- 18 new tests added (73 total auth tests)

## Your Task: Phase 5B Day 4+ Policy Engine Integration

Extend the existing Phase 4 RuleBasedInterceptor for HTTP-specific authentication policies.

### Key Files to Work With:
- **Current Auth:** `src/auth/gateway.rs` (enhanced), `src/auth/policy.rs` (basic)
- **Phase 4 Interceptors:** `src/interceptor/` (complete with RuleBasedInterceptor)
- **Integration Point:** Connect auth context with rule evaluation

### Specifications to Follow:
1. **Primary Spec:** `plans/tasks/reverse-proxy/006-extended-rules-engine-http.md` (if exists)
2. **Task Tracker:** `plans/shadowcat-task-tracker.md` (see Phase 5B section)
3. **Implementation Timeline:** `plans/tasks/reverse-proxy/implementation-timeline.md` (Day 6)
4. **Gap Analysis:** `plans/PHASE5B_IMPLEMENTATION_GAPS.md` (follow-up priorities)

### Current Architecture Context:
```rust
// Phase 4 Complete: Interceptor system with rule engine
src/interceptor/engine.rs    // InterceptorChain with async hooks  
src/interceptor/actions.rs   // Rule actions (pause/modify/block)
src/interceptor/rules.rs     // RuleBasedInterceptor with JSONPath

// Phase 5B Complete: Authentication system
src/auth/gateway.rs          // Enhanced AuthGateway with session management
src/auth/middleware.rs       // Complete Axum middleware suite
src/auth/policy.rs           // Basic PolicyEngine (needs HTTP extension)
```

### Key Integration Points:
1. **HTTP Context in Rules:** Add HTTP method, path, headers to rule conditions
2. **Auth Context in Rules:** Make `AuthContext` available to rule evaluation
3. **Policy Decision Integration:** Connect PolicyEngine with InterceptorChain
4. **Performance:** Maintain < 5ms total auth + policy overhead

### Success Criteria:
- [ ] HTTP-specific rule conditions (method, path, headers, client IP)
- [ ] Authentication context available in rule evaluation
- [ ] Policy decisions integrated with existing interceptor flow
- [ ] Backward compatibility with Phase 4 rule format
- [ ] < 1ms additional overhead for policy evaluation
- [ ] Comprehensive testing of policy integration

### Testing Commands:
```bash
# Navigate to working directory
cd /Users/kevin/src/tapwire/shadowcat

# Current test status
cargo test --lib | grep "test result"  # Should show 232 passing

# Run auth tests specifically  
cargo test auth --lib  # Should show 73 passing

# Test the enhanced authentication
cargo run -- reverse --port 8080 --upstream "echo test"
```

### Context Files to Review:
- `plans/shadowcat-task-tracker.md` - Master task tracking (Phase 5B section)
- `plans/PHASE5B_IMPLEMENTATION_GAPS.md` - Implementation shortcuts and follow-ups
- `PHASE5B_DAY3_COMPLETE.md` - Recent completion summary
- `AUTHENTICATION.md` - Complete authentication guide
- `JWT_VALIDATION_COMPLETE.md` - Day 2 completion details

### Implementation Approach:
1. **Review existing policy engine** in `src/auth/policy.rs`
2. **Extend rule conditions** to include HTTP-specific data
3. **Integrate with InterceptorChain** for policy evaluation
4. **Add authentication context** to rule evaluation environment
5. **Maintain backward compatibility** with existing Phase 4 rules
6. **Optimize performance** to stay within overhead targets

### Key Technical Constraints:
- **Backward Compatibility:** Don't break existing Phase 4 interceptor functionality
- **Performance:** < 1ms additional overhead for policy evaluation
- **Security:** Ensure policy decisions respect authentication context
- **Integration:** Seamless integration with existing auth flow

Start by reviewing the current `src/auth/policy.rs` and `src/interceptor/` structure, then extend the policy engine to handle HTTP-specific conditions while integrating with the authentication context from the enhanced AuthGateway.

The goal is to create a unified authentication + authorization system where HTTP requests go through:
1. **Authentication** (OAuth 2.1 + JWT validation) - ✅ Complete
2. **Authorization** (Policy-based with HTTP context) - 🎯 Your task
3. **Interception** (Existing Phase 4 rules) - ✅ Already works

This builds on the solid authentication foundation to create a complete security gateway for MCP APIs.