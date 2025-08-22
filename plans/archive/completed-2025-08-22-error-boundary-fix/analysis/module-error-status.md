# Module Error Type Status

## Modules With Complete Error Types ✅

These modules have both Error enum and Result type alias:

- ✅ **config** - Has Error and Result, no violations
- ✅ **interceptor** - Has Error and Result, no violations  
- ✅ **rate_limiting** - Has Error and Result, no violations
- ✅ **recorder** - Has Error and Result, no violations
- ✅ **replay** - Has Error and Result, no violations
- ✅ **session** - Has Error and Result, minor violation in builder.rs
- ✅ **transport** - Has Error and Result, minor violation in factory.rs

## Modules With Partial Implementation ⚠️

These modules have Error types but still violate boundaries:

- ⚠️ **auth** - Has Error and Result, but:
  - Still constructs `crate::Error::Auth` directly (gateway.rs, middleware.rs)
  - Uses `crate::Result as ShadowcatResult` aliases (policy.rs, rate_limit.rs)
  - **Action needed**: Use auth::Error consistently

- ⚠️ **pool** - Has Error and Result, but:
  - Traits use `crate::Result` in return types
  - **Action needed**: Refactor traits to use associated types

- ⚠️ **proxy** - Has Error and Result, but:
  - forward submodule uses `crate::Result` (single_session.rs, multi_session.rs)
  - **Action needed**: Use proxy::forward::Result

- ⚠️ **proxy::reverse** - Has Error and Result, but:
  - upstream/stdio.rs maps to `crate::Error::Transport`
  - **Action needed**: Use proxy::reverse::Error

## Modules Needing Error Types ❌

These modules have no Error enum or Result type:

- ❌ **audit** - No Error type
  - Files needing update: logger.rs, store.rs
  - Currently uses: `crate::Result as ShadowcatResult`
  - **Priority**: HIGH (used for compliance)

- ❌ **telemetry** - No Error type
  - Files needing update: mod.rs
  - Currently imports: crate types
  - **Priority**: MEDIUM

- ❌ **process** - No Error type
  - Files needing update: mod.rs
  - Currently imports: crate types
  - **Priority**: MEDIUM

- ❌ **mcp** - No Error type
  - Files needing update: validation.rs, handshake.rs, handler.rs, encoding.rs, builder.rs
  - Currently imports: crate types extensively
  - **Priority**: HIGH (core protocol module)

- ❌ **shutdown** - No Error type
  - Files needing update: shutdown.rs
  - Currently uses: `crate::Result`
  - **Priority**: LOW (single file, might be boundary)

## Special Cases 📝

### CLI Modules
The `cli` module and its submodules (forward, reverse, record, replay, etc.) don't have their own Error types. This might be acceptable since:
- They're command-line interface modules
- They already use module-specific errors (e.g., `replay::Error`)
- They're at the boundary between CLI and library

**Recommendation**: Leave CLI modules as-is unless they cause issues

### API Module
The `api` module is expected to use `crate::Result` as it's the public API boundary. This is correct behavior.

## Summary Statistics

| Status | Count | Modules |
|--------|-------|---------|
| ✅ Complete | 7 | config, interceptor, rate_limiting, recorder, replay, session, transport |
| ⚠️ Partial | 4 | auth, pool, proxy, proxy::reverse |
| ❌ Missing | 5 | audit, telemetry, process, mcp, shutdown |
| 📝 Special | 2 | cli (submodules), api (boundary) |

## Migration Complexity

### Easy (< 2 hours each)
- telemetry - Few files, simple module
- process - Few files, simple module  
- shutdown - Single file module

### Medium (2-4 hours each)
- audit - Need to create Error type and update 2 files
- mcp - Multiple files but straightforward
- pool - Just need to refactor traits

### Complex (4+ hours each)
- auth - Many files, complex interactions, OAuth handling
- proxy/forward - Multiple session types
- proxy/reverse - Complex upstream handling

## Next Steps

1. **Phase 1**: Create Error types for modules that lack them (mcp, process, telemetry, audit)
2. **Phase 2**: Fix modules with partial implementation (auth, pool)
3. **Phase 3**: Clean up remaining boundary violations
4. **Phase 4**: Validate and test