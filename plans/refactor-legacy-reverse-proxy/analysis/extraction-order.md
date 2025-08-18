# Extraction Order Plan

## Overview
Breaking down a 3,682-line monolithic file into manageable modules (<500 lines each) while maintaining functionality and test coverage.

## Extraction Phases

### Phase 1: Foundation Extraction (Week 1)
**Goal:** Extract pure data types and simple utilities
**Risk:** Very Low
**Test Impact:** None

#### 1.1 Remove Admin UI
**Action:** Delete admin-related code
**Lines to Remove:** ~900
**Content:**
- handle_admin_request function
- Admin HTML generation
- Admin route setup
- Related tests

**Steps:**
1. Remove admin handler functions
2. Remove admin routes from router
3. Update tests to remove admin tests
4. Verify remaining tests pass

#### 1.2 Error Types Module
**File:** `src/proxy/reverse/error.rs`
**Lines to Extract:** ~30
**Content:**
- ReverseProxyError enum
- Error conversions
- IntoResponse implementation

**Steps:**
1. Create error.rs
2. Move error types
3. Update imports in legacy.rs
4. Run tests to verify

#### 1.3 Configuration Module
**File:** `src/proxy/reverse/config/`
**Lines to Extract:** ~250
**Content:**
- config/upstream.rs - ReverseUpstreamConfig, HealthCheckConfig, PoolConfig
- config/load_balancing.rs - ReverseLoadBalancingStrategy
- config/server.rs - ReverseProxyConfig
- All impl Default blocks

**Steps:**
1. Create config.rs
2. Move all config structs
3. Add necessary imports
4. Update legacy.rs imports
5. Run tests

#### 1.3 Metrics Module
**File:** `src/proxy/reverse/metrics.rs`
**Lines to Extract:** ~45
**Content:**
- ReverseProxyMetrics struct
- Implementation methods
- Default implementation

**Steps:**
1. Create metrics.rs
2. Move metrics struct and impl
3. Update imports
4. Test metrics collection

#### 1.4 Constants Module
**File:** `src/proxy/reverse/constants.rs`
**Lines to Extract:** ~20
**Content:**
- Default values
- Magic numbers
- Shared constants

### Phase 2: Builder and Validation (Week 1-2)
**Goal:** Extract construction and validation logic
**Risk:** Low
**Test Impact:** Minimal

#### 2.1 Builder Module
**File:** `src/proxy/reverse/builder.rs`
**Lines to Extract:** ~50
**Content:**
- ReverseProxyServerBuilder
- Builder methods

**Dependencies:** config.rs, error.rs

#### 2.2 Validators Module
**File:** `src/proxy/reverse/validators.rs`
**Lines to Extract:** ~150
**Content:**
- Config validation methods
- URL validation
- Transport validation

**Dependencies:** config.rs, error.rs

### Phase 3: Feature Modules (Week 2)
**Goal:** Extract standalone features
**Risk:** Medium
**Test Impact:** Some test restructuring

#### 3.1 Health/Metrics Endpoints
**File:** Keep in server/router.rs
**Lines to Extract:** ~50
**Content:**
- handle_health()
- handle_metrics()

**Dependencies:** metrics.rs, state abstraction

#### 3.2 Session Operations Module
**File:** `src/proxy/reverse/session_ops.rs`
**Lines to Extract:** ~200
**Content:**
- get_or_create_session()
- update_session_state()
- Session helper functions

**Dependencies:** SessionManager from main crate

#### 3.3 Response Module
**File:** `src/proxy/reverse/response.rs`
**Lines to Extract:** ~100
**Content:**
- Response formatting
- Header creation
- Error responses

### Phase 4: Upstream Processing (Week 2-3)
**Goal:** Extract upstream-specific logic
**Risk:** Medium-High
**Test Impact:** May need test refactoring

#### 4.1 Upstream Processors Module
**File:** `src/proxy/reverse/upstream/`
**Lines to Extract:** ~650
**Content:**
- upstream/stdio_processor.rs - process_via_stdio_pooled()
- upstream/http_processor.rs - process_via_http(), process_via_http_hyper()
- upstream/selector.rs - Upstream selection logic, routing decisions

**Dependencies:** Main transport traits, connection pools

#### 4.2 Connection Pool Management
**File:** Keep using main `proxy::pool`
**Action:** Update references to use main pool module

### Phase 5: SSE Handling (Week 3)
**Goal:** Extract SSE streaming logic
**Risk:** High
**Test Impact:** Significant

#### 5.1 SSE Streaming Module
**File:** `src/proxy/reverse/sse_streaming.rs`
**Lines to Extract:** ~450
**Content:**
- handle_mcp_sse_request()
- proxy_sse_response()
- proxy_sse_from_upstream()
- SSE utilities

**Dependencies:** Complex async streaming

### Phase 6: Core Refactoring (Week 3-4)
**Goal:** Refactor main request handler
**Risk:** Very High
**Test Impact:** Major

#### 6.1 Request Parser Module
**File:** `src/proxy/reverse/request_parser.rs`
**Lines to Extract:** ~150
**Content:**
- Header extraction
- Request validation
- Protocol detection

#### 6.2 Message Processor Module
**File:** `src/proxy/reverse/message_processor.rs`
**Lines to Extract:** ~200
**Content:**
- process_message()
- Message routing
- Interception logic

#### 6.3 MCP Handler Module
**File:** `src/proxy/reverse/mcp_handler.rs`
**Lines to Extract:** ~500
**Content:**
- Refactored handle_mcp_request() (currently 550 lines)
- Split into smaller helper functions
- Main /mcp endpoint logic

### Phase 7: Server Module (Week 4)
**Goal:** Extract server lifecycle
**Risk:** High
**Test Impact:** Integration tests affected

#### 7.1 Server Module
**File:** `src/proxy/reverse/server.rs`
**Lines to Extract:** ~500
**Content:**
- ReverseProxyServer struct
- Server methods
- Router creation
- Shutdown handling

#### 7.2 State Module
**File:** `src/proxy/reverse/state.rs`
**Lines to Extract:** ~100
**Content:**
- AppState struct
- State initialization
- State accessors

### Phase 8: Cleanup (Week 4-5)
**Goal:** Final organization
**Risk:** Low
**Test Impact:** None

#### 8.1 Module Organization
**File:** `src/proxy/reverse/mod.rs`
**Content:**
- Public exports
- Module structure
- Documentation

#### 8.2 Test Reorganization
**File:** `src/proxy/reverse/tests/`
**Content:**
- Split tests by module
- Integration tests
- Unit tests

## Execution Timeline

| Week | Phase | Modules | Risk | Lines Removed |
|------|-------|---------|------|--------------|
| 1 | Phase 1-2 | errors, config, metrics, builders | Low | ~500 |
| 2 | Phase 3-4 | admin, session, transport | Medium | ~750 |
| 3 | Phase 5-6 | SSE, request handling | High | ~1000 |
| 4 | Phase 7-8 | server, cleanup | Medium | ~600 |

## Success Criteria Per Phase

### Phase 1 Success
- [ ] Config types extracted
- [ ] Error types separated
- [ ] All tests passing
- [ ] No functionality change

### Phase 2 Success
- [ ] Builders extracted
- [ ] Validation separated
- [ ] Tests passing
- [ ] Code compiles cleanly

### Phase 3 Success
- [ ] Admin endpoints isolated
- [ ] Session logic extracted
- [ ] Feature tests passing

### Phase 4 Success
- [ ] Transport logic modular
- [ ] Pool management separated
- [ ] Integration tests passing

### Phase 5 Success
- [ ] SSE in dedicated module
- [ ] Streaming tests passing
- [ ] No performance regression

### Phase 6 Success
- [ ] Request handler < 300 lines
- [ ] Clear separation of concerns
- [ ] All tests passing

### Phase 7 Success
- [ ] Server module complete
- [ ] State abstraction working
- [ ] Integration tests passing

### Phase 8 Success
- [ ] No module > 500 lines
- [ ] Clean module structure
- [ ] Documentation complete
- [ ] All tests organized

## Risk Mitigation Strategies

### For Each Extraction
1. **Create module file**
2. **Move code with tests**
3. **Update imports**
4. **Run tests immediately**
5. **Fix compilation errors**
6. **Run full test suite**
7. **Check clippy warnings**
8. **Commit if green**

### Rollback Plan
- Git commit after each successful extraction
- Tag stable points
- Keep legacy.rs until Phase 8
- Parallel implementation if needed

### Testing Strategy
- Unit tests move with code
- Integration tests updated per phase
- New tests for interfaces
- Performance benchmarks maintained

## Module Size Targets

| Module | Target Lines | Notes |
|--------|-------------|-------|
| **Removed** | | |
| admin UI | ~900 | Deleted entirely |
| **New Modules** | | |
| error.rs | 50 | Error types |
| config/ | 250 | All config types |
| metrics.rs | 50 | Metrics collection |
| builder.rs | 100 | Server builder |
| server/ | 550 | Server lifecycle |
| mcp_handler.rs | 500 | Main /mcp handler |
| sse_streaming.rs | 450 | SSE response streaming |
| upstream/ | 650 | Upstream processing |
| session_ops.rs | 200 | Session operations |
| middleware/ | 400 | Auth, rate limit, interceptor |
| response/ | 150 | Response utilities |

**Total Target:** ~2,750 lines (from 3,682 - 900 admin)
**Test Target:** Keep at ~800 lines but organized

## Next Steps

1. **Review this plan** with stakeholders
2. **Create branch** for refactoring
3. **Start Phase 1** with error types
4. **Document progress** in tracker
5. **Update estimates** based on actual progress