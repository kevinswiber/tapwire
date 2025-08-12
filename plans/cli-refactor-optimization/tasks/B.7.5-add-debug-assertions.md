# Task B.7.5: Add Debug Assertions to Builders

**Status**: ⬜ Not Started  
**Estimated Duration**: 0.5 hours  
**Dependencies**: B.1 (Builder Patterns)  
**Priority**: LOW-MEDIUM  

## Context

From the [Comprehensive Review](../../../reviews/cli-refactor-optimization/comprehensive-review.md#92-medium-priority), builder `build()` methods should have debug assertions for invariants.

## Problem

Builder patterns may have implicit invariants that should be checked in debug builds:
- Mutually exclusive options
- Required combinations
- Valid ranges
- State consistency

## Solution

Add `debug_assert!` statements in builder `build()` methods:

```rust
impl ShadowcatBuilder {
    pub fn build(self) -> Result<Shadowcat> {
        // Debug-only invariant checks
        debug_assert!(
            self.session_timeout > Duration::from_secs(0),
            "Session timeout must be positive"
        );
        
        debug_assert!(
            self.rate_limit_requests > 0,
            "Rate limit requests must be positive"
        );
        
        // Production validation
        // ...
    }
}
```

## Implementation Steps

1. [ ] Review builder files:
   - [ ] `src/api.rs` - ShadowcatBuilder
   - [ ] `src/transport/builders.rs` - Transport builders
   - [ ] `src/proxy/builders.rs` - Proxy builders
   - [ ] `src/session/builder.rs` - Session builder
   - [ ] `src/interceptor/builder.rs` - Interceptor builder

2. [ ] For each builder, identify invariants:
   - [ ] Value ranges (timeouts > 0, limits > 0)
   - [ ] Mutually exclusive options
   - [ ] Required field combinations
   - [ ] State consistency

3. [ ] Add appropriate `debug_assert!` statements with descriptive messages

4. [ ] Test in debug mode to ensure assertions are correct

## Examples

```rust
// In TransportFactoryConfigBuilder
debug_assert!(
    !(self.stdio_defaults.is_some() && self.force_stdio_only),
    "Cannot set stdio defaults when forcing stdio-only mode"
);

// In ForwardProxyBuilder
debug_assert!(
    self.client_transport.is_some() || self.server_transport.is_some(),
    "At least one transport must be configured"
);

// In SessionManagerBuilder
debug_assert!(
    self.cleanup_interval <= self.session_timeout,
    "Cleanup interval should not exceed session timeout"
);
```

## Files to Modify

- `src/api.rs` - ShadowcatBuilder
- `src/transport/builders.rs` - Transport builders
- `src/proxy/builders.rs` - Proxy builders
- `src/session/builder.rs` - Session builder
- `src/interceptor/builder.rs` - Interceptor builder

## Testing

- [ ] Run tests in debug mode: `cargo test`
- [ ] Run tests in release mode: `cargo test --release`
- [ ] Verify assertions don't fire for valid configurations
- [ ] Test that invalid configurations trigger assertions in debug mode

## Success Criteria

- [ ] Key invariants have debug assertions
- [ ] Assertions have descriptive messages
- [ ] No performance impact in release builds
- [ ] All tests pass in both debug and release modes
- [ ] Debug assertions catch logic errors early