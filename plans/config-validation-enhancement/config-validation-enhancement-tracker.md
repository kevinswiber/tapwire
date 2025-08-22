# Config Validation Enhancement Tracker

## Overview

The Shadowcat config module currently uses generic error types that provide limited context for debugging and user guidance. This plan implements rich, actionable error types with specific variants for different validation failures, workload-based defaults, and improved user experience through help text and suggestions.

**Last Updated**: 2025-08-22  
**Total Estimated Duration**: 16-24 hours  
**Status**: Planning

## Goals

1. **Rich Error Context** - Replace generic string errors with specific, actionable error variants
2. **Better Naming** - Rename `ShadowcatConfig` to `Config` to avoid redundancy
3. **Smart Defaults** - Add workload-based configuration profiles
4. **User Guidance** - Provide help text and suggestions for configuration errors
5. **Performance Warnings** - Distinguish between errors and performance warnings

## Current Problems

### 1. Generic Error Messages
```rust
// Current - loses context
Error::Invalid(format!("Invalid port in server bind address '{}': {}", addr, e))

// Desired - preserves context
Error::InvalidPort { 
    port: 8080,
    reason: PortError::Privileged,
    suggestion: "Use a port above 1024 or run with elevated privileges"
}
```

### 2. Redundant Naming
```rust
// Current
use shadowcat::config::ShadowcatConfig;  // Redundant

// Desired
use shadowcat::config::Config;  // Clean
```

### 3. No Smart Defaults
Users must configure everything manually, even when common patterns exist for different workloads.

### 4. Poor Error Guidance
Errors tell users what's wrong but not how to fix it.

## Architecture Vision

```
Current Architecture:
┌─────────────────────────────────────┐
│         config/mod.rs               │
│  - Generic Error enum               │
│  - Basic validation                 │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│      config/validator.rs            │
│  - String-based error messages      │
│  - No actionable guidance          │
└─────────────────────────────────────┘

Target Architecture:
┌─────────────────────────────────────┐
│         config/error.rs             │
│  - Rich Error variants              │
│  - PortError, ResourceType enums    │
│  - help_text() method               │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│      config/validator.rs            │
│  - Specific error construction      │
│  - Performance warnings             │
│  - Resource limit checks            │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│      config/defaults.rs             │
│  - Workload-based profiles          │
│  - Smart defaults                   │
└─────────────────────────────────────┘
```

## Work Phases

### Phase 1: Analysis & Design (2-3 hours)
| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | Audit Current Error Usage | 1h | None | ⬜ Not Started | | Find all validation error patterns |
| A.1 | Design Error Variants | 1h | A.0 | ⬜ Not Started | | Define specific error types needed |
| A.2 | Document Breaking Changes | 0.5h | A.1 | ⬜ Not Started | | List all files affected by rename |

### Phase 2: Core Error Refactor (4-6 hours)
| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.0 | Create Enhanced Error Types | 2h | A.1 | ⬜ Not Started | | New error.rs with rich variants |
| B.1 | Rename ShadowcatConfig → Config | 1h | A.2 | ⬜ Not Started | | Update all imports |
| B.2 | Update Validator Error Construction | 3h | B.0 | ⬜ Not Started | | Use specific error variants |

### Phase 3: Smart Defaults (4-6 hours)
| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.0 | Define Workload Profiles | 1h | B.1 | ⬜ Not Started | | High-throughput, low-latency, dev, prod |
| C.1 | Implement defaults.rs | 2h | C.0 | ⬜ Not Started | | Workload-based configurations |
| C.2 | Add Builder Pattern | 2h | C.1 | ⬜ Not Started | | ConfigBuilder with validation |

### Phase 4: User Experience (3-4 hours)
| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.0 | Add help_text() Method | 1.5h | B.0 | ⬜ Not Started | | Actionable error guidance |
| D.1 | Add Performance Warnings | 1.5h | B.2 | ⬜ Not Started | | Distinguish warnings from errors |
| D.2 | Add Resource Limit Checking | 1h | B.2 | ⬜ Not Started | | Check FD limits, memory, etc. |

### Phase 5: Testing & Documentation (3-5 hours)
| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| E.0 | Update Unit Tests | 2h | D.0-D.2 | ⬜ Not Started | | Test new error types |
| E.1 | Add Integration Tests | 1.5h | E.0 | ⬜ Not Started | | Test workload profiles |
| E.2 | Update Documentation | 1h | E.1 | ⬜ Not Started | | Document new features |

## Success Criteria

### Functional Requirements
- [ ] All validation errors use specific variants instead of generic strings
- [ ] `ShadowcatConfig` renamed to `Config` throughout codebase
- [ ] Workload-based defaults available (high-throughput, low-latency, development, production)
- [ ] Error help text provides actionable guidance
- [ ] Performance warnings separate from hard errors

### Code Quality Requirements
- [ ] No clippy warnings
- [ ] All tests pass
- [ ] Breaking changes documented
- [ ] Examples updated

### User Experience Requirements
- [ ] Error messages clearly indicate the problem
- [ ] Help text provides specific solutions
- [ ] Common configurations achievable with workload defaults
- [ ] Validation catches incompatible settings

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Breaking API changes | HIGH | HIGH | Document changes, update examples |
| Complex error hierarchies | MEDIUM | MEDIUM | Start with most common cases |
| Performance regression in validation | LOW | LOW | Benchmark before/after |
| Merge conflicts during rename | MEDIUM | HIGH | Complete rename quickly in one session |

## Dependencies

### External
- `thiserror` - Already in use for error derivation
- `serde` - Already in use for config serialization

### Internal
- All modules that import `ShadowcatConfig` will need updates
- API examples may need updates
- Documentation needs updates

## Implementation Notes

### Error Variant Priority
Start with the most commonly hit validation errors:
1. Port validation (privileged, out of range, in use)
2. Address parsing
3. Rate limiting configuration
4. Resource limits
5. TLS configuration

### Workload Profiles
Focus on four main profiles initially:
1. **Development** - Permissive, verbose, long timeouts
2. **Production** - Secure, optimized, short timeouts
3. **HighThroughput** - Large buffers, many connections
4. **LowLatency** - Small buffers, TCP_NODELAY

### Help Text Strategy
Each error variant should answer:
1. What went wrong?
2. Why is it a problem?
3. How can the user fix it?
4. What are the alternatives?

## Session Planning

### Session 1 (Analysis & Core Refactor)
- Complete Phase 1 (Analysis & Design)
- Start Phase 2 (Core Error Refactor)
- Goal: New error types in place

### Session 2 (Complete Refactor & Defaults)
- Complete Phase 2
- Complete Phase 3 (Smart Defaults)
- Goal: Workload-based configs working

### Session 3 (UX & Testing)
- Complete Phase 4 (User Experience)
- Complete Phase 5 (Testing & Documentation)
- Goal: Full feature complete and tested

## Key Commands

```bash
# Find all ShadowcatConfig references
grep -r "ShadowcatConfig" src/ --include="*.rs"

# Find all Error::Invalid constructions
grep -r "Error::Invalid" src/config --include="*.rs"

# Test config validation
cargo test config::validator

# Check for breaking changes
cargo check --all-targets
```

## References

- Original feedback: From Claude session on config error handling
- Current config module: `src/config/`
- Validation patterns to replace: `Error::Invalid(format!(...))`
- Example workload configs: High-throughput proxy, development server

## Related Work

- Error boundary fix (completed 2025-08-22) - Established module error patterns
- Previous config work - Basic validation already in place