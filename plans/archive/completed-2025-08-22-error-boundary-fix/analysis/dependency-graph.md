# Module Dependency Graph

## Dependency Hierarchy

### Level 0 - Foundation (No internal dependencies)
These modules can be migrated first as they don't depend on other internal modules:

```
telemetry (no deps)
shutdown (no deps)
```

### Level 1 - Core Infrastructure
Modules with minimal dependencies:

```
mcp (deps: transport)
process (deps: telemetry, transport)
```

### Level 2 - Transport & Config
Core infrastructure modules:

```
transport (deps: mcp, process, telemetry, + uses for others)
config (deps: mcp, session, transport)
```

### Level 3 - Mid-Level Services
Modules that depend on core infrastructure:

```
pool (deps: none internally, but uses crate::Result)
audit (deps: auth, mcp, rate_limiting)
recorder (deps: mcp, session, interceptor)
session (deps: config, mcp, transport, shutdown)
```

### Level 4 - Auth & Interceptor
Higher-level services:

```
auth (deps: mcp, transport, session, interceptor)
interceptor (deps: auth, mcp)
  ^ Circular dependency between auth <-> interceptor
```

### Level 5 - Proxy Layer
Top-level proxy implementations:

```
proxy (deps: auth, config, interceptor, mcp, pool, rate_limiting, recorder, session)
├── forward (deps: transport, session, pool)
└── reverse (deps: auth, config, transport)
```

### Level 6 - Application Layer
User-facing modules:

```
replay (deps: config, mcp, recorder)
api (deps: all modules)
cli (deps: most modules)
```

## Visual Dependency Tree

```
                          api/cli
                             │
                    ┌────────┴────────┐
                    │                  │
                  replay            proxy
                    │              /   │   \
                recorder      forward reverse
                    │             │       │
            ┌───────┴────┬────────┴───────┴───────┐
            │            │                         │
      interceptor <-> auth                      pool
            │            │                         
        session     rate_limiting                  
            │            │                         
        ┌───┴────┬───────┴────────┐               
        │        │                │               
    config   transport          audit             
        │        │                                 
        │    ┌───┴────┐                           
        │    │        │                           
       mcp process telemetry                      
                      │                           
                 (shutdown)                        
```

## Circular Dependency Risks ⚠️

### Identified Circular Dependencies

1. **auth ↔ interceptor**
   - auth uses interceptor
   - interceptor uses auth
   - **Risk**: Cannot migrate independently
   - **Mitigation**: Migrate together or break dependency

### Potential Issues

1. **transport module**
   - Many modules depend on transport
   - transport also imports from proxy, recorder, session
   - **Risk**: Complex migration
   - **Mitigation**: transport might be at API boundary

## Error Flow Paths

### Current (Problematic) Flows

```
pool::Error ──────────────> crate::Error (VIOLATION)
auth::Error ──────────────> crate::Error (VIOLATION)
transport::Error ─────────> crate::Error (VIOLATION)
```

### Target Error Flow

```
Level 0-1 Errors:
telemetry::Error ─────> process::Error ─────> transport::Error
mcp::Error ───────────────────────────────> transport::Error

Level 2-3 Errors:
transport::Error ─────> session::Error ─────> proxy::Error ─> crate::Error
config::Error ────────> proxy::Error ──────> crate::Error
pool::Error ──────────> proxy::forward::Error ─> crate::Error

Level 4-5 Errors:
auth::Error ──────────> proxy::reverse::Error ─> crate::Error
interceptor::Error ───> proxy::Error ─────────> crate::Error
```

## Migration Order Recommendation

Based on dependencies, migrate in this order:

### Phase 1: Foundation (No dependencies)
1. **telemetry** - No dependencies
2. **shutdown** - No dependencies

### Phase 2: Core Protocol
3. **mcp** - Only depends on transport (which has its own Error)
4. **process** - Depends on telemetry (done) and transport

### Phase 3: Infrastructure
5. **pool** - No internal deps, just fix trait usage
6. **audit** - Create Error type

### Phase 4: Services
7. **auth** - Fix to use own Error consistently
8. **interceptor** - Already has Error, check usage

### Phase 5: Proxy Layer
9. **proxy::forward** - Fix Result usage
10. **proxy::reverse** - Fix Error construction

### Phase 6: Cleanup
11. **transport** - Fix factory.rs
12. **session** - Fix builder.rs
13. Remaining minor violations

## Key Insights

1. **Circular dependency**: auth ↔ interceptor needs special handling
2. **Transport is central**: Many modules depend on transport
3. **Clean foundation**: telemetry and shutdown can be done first
4. **mcp is critical**: Core protocol module used everywhere
5. **Proxy is top-level**: Should be migrated last

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| auth ↔ interceptor circular | HIGH | Migrate together or use trait objects |
| transport dependencies | MEDIUM | Might be at boundary, check if needed |
| Breaking changes cascade | MEDIUM | Test after each module migration |
| Hidden dependencies | LOW | Run full test suite frequently |