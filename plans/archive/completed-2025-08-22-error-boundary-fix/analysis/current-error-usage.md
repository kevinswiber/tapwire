# Current Error Usage Analysis

## Summary Statistics
- **Total `crate::Error` references**: 5 (direct construction/usage)
- **Total `crate::Result` references**: 13 
- **Total violations**: 18
- **Modules affected**: 9 distinct modules

## Violations by Module

| Module | Error Refs | Result Refs | Has Own Error? | Priority | Notes |
|--------|------------|-------------|----------------|----------|-------|
| auth | 4 | 3 | ✅ Yes | HIGH | Has Error but still references crate::Error |
| pool | 0 | 4 | ✅ Yes | MEDIUM | Has Error but uses crate::Result in traits |
| proxy/reverse | 1 | 1 | ✅ Yes | MEDIUM | upstream/stdio.rs violates |
| proxy/forward | 0 | 2 | ✅ Yes | MEDIUM | single/multi session files |
| audit | 0 | 1 | ❌ No | HIGH | Needs Error type |
| session | 0 | 1 | ✅ Yes | LOW | builder.rs only |
| shutdown | 0 | 1 | ❌ No | LOW | Single file module |
| transport | 0 | 1 | ✅ Yes | LOW | factory.rs only |
| telemetry | 0 | 0 | ❌ No | MEDIUM | Uses Error in imports |
| process | 0 | 0 | ❌ No | MEDIUM | Uses Error in imports |
| mcp | 0 | 0 | ❌ No | HIGH | Multiple files use Error |
| cli | 0 | 0 | ❌ No | LOW | CLI command modules |

## Violation Patterns

### Pattern 1: Direct Error Construction
Files directly constructing `crate::Error` variants:
- `auth/gateway.rs` - Wrapping auth errors in crate::Error::Auth
- `auth/middleware.rs` - Using crate::Error in middleware
- `proxy/reverse/upstream/stdio.rs` - Mapping to crate::Error::Transport

**Fix approach**: These modules have their own Error types but are bypassing them to construct crate::Error directly.

### Pattern 2: Result Type in Traits
Files using `crate::Result` in trait definitions:
- `pool/mod.rs` - Trait methods returning crate::Result
- `pool/traits.rs` - Importing crate::Result

**Fix approach**: Traits should use associated types or generic parameters instead of crate::Result.

### Pattern 3: Result Type Aliases
Files creating aliases to `crate::Result`:
- `auth/policy.rs` - `use crate::Result as ShadowcatResult`
- `auth/rate_limit.rs` - `use crate::Result as ShadowcatResult`
- `audit/logger.rs` - `use crate::Result as ShadowcatResult`

**Fix approach**: These should use module-local Result types instead.

### Pattern 4: Missing Error Types
Modules without any Error type:
- `audit` - No Error enum, uses crate::Result
- `telemetry` - No Error enum, imports crate types
- `process` - No Error enum, imports crate types
- `mcp` - No Error enum, multiple files import crate types
- `cli` - No Error enum (might be okay for CLI)

**Fix approach**: Create module-specific Error and Result types.

## High-Risk Modules

### 1. `auth` module (7 violations)
- **Primary issue**: Has its own Error type but still constructs crate::Error directly
- **Files affected**: gateway.rs, middleware.rs, policy.rs, rate_limit.rs
- **Dependencies**: Used by proxy::reverse
- **Suggested approach**: Stop constructing crate::Error, use auth::Error consistently

### 2. `mcp` module (5+ imports)
- **Primary issue**: No Error type, multiple files import crate types
- **Files affected**: validation.rs, handshake.rs, handler.rs, encoding.rs, builder.rs
- **Dependencies**: Core protocol module used everywhere
- **Suggested approach**: Create mcp::Error and Result types

### 3. `audit` module (1 violation)
- **Primary issue**: No Error type, uses crate::Result alias
- **Files affected**: logger.rs, store.rs
- **Dependencies**: Used for compliance logging
- **Suggested approach**: Create audit::Error and Result types

### 4. `pool` module (4 violations)
- **Primary issue**: Has Error type but traits use crate::Result
- **Files affected**: mod.rs, traits.rs
- **Dependencies**: Used by proxy modules
- **Suggested approach**: Refactor traits to use associated types

## Modules Already Compliant

These modules have proper Error types and appear to use them correctly:
- ✅ `config` - Has Error, no violations found
- ✅ `interceptor` - Has Error, no violations found
- ✅ `rate_limiting` - Has Error, no violations found
- ✅ `recorder` - Has Error, no violations found
- ✅ `replay` - Has Error, no violations found
- ✅ `transport` - Has Error, only factory.rs imports (might be at boundary)

## Key Insights

1. **Fewer violations than expected**: Only 18 actual violations vs 161 shown in screenshot
   - The screenshot might be showing all references including legitimate ones in lib.rs
   - Or counting transitive dependencies

2. **Main problems**:
   - Modules with Error types still constructing crate::Error directly (auth)
   - Traits using crate::Result instead of generic types (pool)
   - Core modules lacking Error types (mcp, audit, telemetry, process)

3. **Quick wins**:
   - Create Error types for audit, telemetry, process, mcp
   - Fix auth module to use its own Error consistently
   - Refactor pool traits to use associated types

4. **CLI modules**: The cli submodules might not need their own Error types since they're just command interfaces

## Recommended Priority Order

1. **Create missing Error types** (Phase 1)
   - mcp (core protocol, no dependencies)
   - process (no dependencies)
   - telemetry (no dependencies)
   - audit (few dependencies)

2. **Fix existing modules** (Phase 2)
   - auth (stop constructing crate::Error)
   - pool (refactor traits)

3. **Clean up remaining** (Phase 3)
   - proxy modules (forward, reverse)
   - session (builder only)
   - transport (factory only)
   - shutdown (single file)