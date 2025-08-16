# Transport Dependency Map

**Created**: 2025-08-16  
**Purpose**: Visual representation of transport-related dependencies in shadowcat

## Module Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│                      TransportType Enum                      │
│                   (src/transport/mod.rs)                     │
└─────────────────────────────┬───────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Session    │     │   Transport  │     │    Config    │
│  Management  │     │    Layer     │     │    Layer     │
├──────────────┤     ├──────────────┤     ├──────────────┤
│ - store.rs   │     │ - envelope   │     │ - schema     │
│ - manager.rs │     │ - directional│     │ - reverse    │
│ - memory.rs  │     │ - factory    │     │   _proxy     │
└──────────────┘     └──────────────┘     └──────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                ┌─────────────┴─────────────┐
                │                           │
                ▼                           ▼
        ┌──────────────┐           ┌──────────────┐
        │   Forward    │           │   Reverse    │
        │    Proxy     │           │    Proxy     │
        ├──────────────┤           ├──────────────┤
        │ Uses:        │           │ Uses:        │
        │ - Incoming   │           │ - Direct     │
        │   Transport  │           │   HTTP       │
        │ - Outgoing   │           │ - Manual SSE │
        │   Transport  │           │   detection  │
        └──────────────┘           └──────────────┘
```

## is_sse_session Field Dependencies

```
┌─────────────────────────────────────────────────────────────┐
│                    Session Struct                           │
│                 (src/session/store.rs)                      │
│  Fields:                                                    │
│  - transport_type: TransportType                           │
│  - is_sse_session: bool  ← CODE SMELL                      │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
                 ┌────────────────────────┐
                 │  mark_as_sse_session() │
                 │    (Never Called!)      │
                 └────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │    is_sse()     │
                    │  (Rarely Used)  │
                    └─────────────────┘
```

## Directional Transport Architecture (Forward Proxy)

```
┌─────────────────────────────────────────────────────────────┐
│                    Forward Proxy                            │
└─────────────────────────┬───────────────────────────────────┘
                          │
            ┌─────────────┴─────────────┐
            │                           │
            ▼                           ▼
    ┌──────────────────┐       ┌──────────────────┐
    │ IncomingTransport│       │OutgoingTransport │
    │      Trait       │       │      Trait       │
    └────────┬─────────┘       └────────┬─────────┘
             │                           │
    ┌────────┼────────┐         ┌───────┼────────┐
    │        │        │         │       │        │
    ▼        ▼        ▼         ▼       ▼        ▼
┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
│Stdio │ │ HTTP │ │ SSE  │ │Stdio │ │ HTTP │ │ SSE  │
│ In   │ │Server│ │ In   │ │ Out  │ │Client│ │ Out  │
└──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘
```

## Current Reverse Proxy Architecture (Problematic)

```
┌─────────────────────────────────────────────────────────────┐
│                    Reverse Proxy                            │
│                  (legacy.rs - 1000+ lines)                  │
└─────────────────────────┬───────────────────────────────────┘
                          │
            ┌─────────────┼─────────────┐
            │             │             │
            ▼             ▼             ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐
    │  Direct  │   │  Manual  │   │Connection│
    │   HTTP   │   │   SSE    │   │   Pool   │
    │  Client  │   │Detection │   │(Stdio only)│
    └──────────┘   └──────────┘   └──────────┘
            │             │             │
            └─────────────┼─────────────┘
                          │
                          ▼
                ┌──────────────────┐
                │ Content-Type     │
                │ Detection:       │
                │ - text/event-    │
                │   stream → SSE   │
                │ - application/   │
                │   json → JSON    │
                └──────────────────┘
```

## Cross-Module Dependencies

### Core Dependencies
```
transport/mod.rs (TransportType)
    ├── session/store.rs (Session.transport_type)
    ├── session/manager.rs (create_session)
    ├── config/schema.rs (ReverseUpstreamConfig)
    ├── transport/envelope.rs (TransportContext conversion)
    └── proxy/reverse/legacy.rs (routing logic)
```

### Test Dependencies
```
TransportType used in tests:
    ├── tests/integration_* (15+ files)
    ├── tests/performance_test.rs
    ├── tests/version_* (3 files)
    ├── src/*/tests.rs (inline tests)
    └── examples/*.rs (3 files)
```

## Proposed Architecture (After Refactor)

```
┌─────────────────────────────────────────────────────────────┐
│                     Unified Architecture                     │
└─────────────────────────┬───────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│SessionOrigin │  │ResponseMode  │  │  Directional │
│    Enum      │  │    Enum      │  │  Transports  │
├──────────────┤  ├──────────────┤  ├──────────────┤
│ - Stdio      │  │ - Json       │  │ - Incoming   │
│ - Http       │  │ - Sse        │  │ - Outgoing   │
│ - Sse        │  │ - Bidirect   │  │              │
└──────────────┘  └──────────────┘  └──────────────┘
        │                 │                 │
        └─────────────────┼─────────────────┘
                          │
            ┌─────────────┴─────────────┐
            │                           │
            ▼                           ▼
    ┌──────────────┐           ┌──────────────┐
    │   Forward    │           │   Reverse    │
    │    Proxy     │           │    Proxy     │
    │  (unified)   │           │  (unified)   │
    └──────────────┘           └──────────────┘
```

## Migration Path

### Phase B Dependencies (Quick Fix)
```
1. Add ResponseMode enum
   └── src/transport/mod.rs or new file

2. Update Session struct
   ├── Remove is_sse_session field
   └── Add response_mode field

3. Update SSE detection
   ├── proxy/reverse/hyper_client.rs
   ├── proxy/reverse/legacy.rs
   └── proxy/reverse/sse_resilience.rs
```

### Phase C Dependencies (Unification)
```
1. Refactor reverse proxy
   ├── Adopt IncomingTransport trait
   ├── Adopt OutgoingTransport trait
   └── Remove duplicate transport logic

2. Unify transport factory
   ├── Merge forward/reverse creation
   └── Single factory for all transports

3. Update connection pooling
   ├── Support all transport types
   └── Unified pool management
```

## Risk Areas

### High-Risk Dependencies
- `proxy/reverse/legacy.rs` - Core reverse proxy logic
- `session/manager.rs` - Session lifecycle management
- `transport/envelope.rs` - Protocol message handling

### Medium-Risk Dependencies
- `config/reverse_proxy.rs` - Configuration parsing
- `interceptor/*` - Message interception
- `recorder/tape.rs` - Session recording

### Low-Risk Dependencies
- Tests - Can be updated incrementally
- Examples - Reference implementations
- CLI - Command-line interface

## Summary

The dependency map reveals:
1. **TransportType is widely used** but mostly for categorization
2. **is_sse_session is isolated** with minimal dependencies
3. **Forward proxy architecture is clean** and should be adopted
4. **Reverse proxy is monolithic** and needs decomposition
5. **Test coverage is extensive** but manageable with incremental updates

The refactor can proceed safely by:
- Starting with isolated changes (ResponseMode enum)
- Gradually migrating reverse proxy to directional transports
- Maintaining backward compatibility during transition
- Leveraging extensive test coverage for validation