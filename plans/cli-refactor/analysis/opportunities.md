# Refactoring Opportunities

## High Priority (Maximum Impact)

### 1. Extract ProxyConfig to Common Module
- **Impact**: Eliminates duplication across 4 commands
- **Location**: Used in forward, reverse, replay commands  
- **Benefit**: Single source of truth for proxy configuration
- **Implementation**: Move to `cli/common.rs` with builder pattern

### 2. Consolidate Rate Limiter Initialization
- **Impact**: Removes 3 duplicate code blocks (~40 lines each)
- **Current**: Lines 303-328, 414-439, 907-932
- **Benefit**: Consistent rate limiting setup
- **Implementation**: Create `common::create_rate_limiter(config)`

### 3. Extract Command Execution Logic
- **Impact**: Reduces main.rs from 1294 to ~200 lines
- **Commands to extract**:
  - Forward (stdio: ~110 lines, http: ~60 lines)
  - Reverse (~103 lines)
  - Record (stdio: ~90 lines, http: ~60 lines)
  - Replay (~158 lines + handler)
- **Benefit**: Testable command modules

### 4. Create Shared HTTP Handlers Module
- **Impact**: Reusable HTTP handling logic
- **Functions**: handle_recording_request, handle_replay_request
- **Benefit**: Consistent HTTP request processing

## Medium Priority (Good Value)

### 5. Extract Session Manager Factory
- **Impact**: Consistent session manager creation
- **Pattern**: Arc::new(SessionManager::with_config())
- **Benefit**: Centralized session configuration

### 6. Move JSON Conversion Utilities
- **Impact**: Better organization (~78 lines)
- **Functions**: json_to_transport_message, transport_message_to_json
- **Target**: `transport` module or `cli/common.rs`
- **Benefit**: Reusable across all HTTP handlers

### 7. Create Command Spawning Helper
- **Impact**: Shared logic for stdio commands
- **Used by**: forward-stdio, record-stdio
- **Benefit**: Consistent process management

### 8. Standardize Error Handling
- **Impact**: Consistent error reporting
- **Pattern**: Create common error handler for CLI
- **Benefit**: Better user experience

## Low Priority (Nice to Have)

### 9. Extract Logging Configuration
- **Impact**: Cleaner main function
- **Current**: init_logging function
- **Target**: `cli/common.rs` or separate module

### 10. Consolidate Test Message Creation
- **Impact**: Remove hardcoded test messages
- **Current**: Initialize request in forward command
- **Benefit**: Easier testing

### 11. Create HTTP Server Builder
- **Impact**: Consistent axum server setup
- **Used by**: forward-http, reverse, record-http, replay
- **Benefit**: Standardized server configuration

## Code Quality Improvements

### Remove Dead Code
- **Test message sending**: Lines 341-387 in run_stdio_forward
- **Placeholder implementations**: Forward commands are incomplete

### Reduce Function Complexity
- **run_replay_server**: 158 lines - split into smaller functions
- **handle_recording_request**: 96 lines - extract sub-operations
- **main function**: 131 lines - could be more concise with proper dispatch

### Improve Type Safety
- **String-based configuration**: Could use strongly-typed builders
- **Command arguments**: Could validate earlier with better types

## Migration Strategy Priorities

### Phase 1: Foundation (Week 1)
1. Create `cli/common.rs` with ProxyConfig
2. Extract rate limiter initialization
3. Move JSON utilities

### Phase 2: Command Modules (Week 1-2)
1. Create `cli/forward.rs` with both transports
2. Create `cli/reverse.rs`
3. Create `cli/record.rs` with both transports
4. Create `cli/replay.rs`

### Phase 3: Cleanup (Week 2)
1. Remove extracted code from main.rs
2. Simplify main() to pure dispatch
3. Add comprehensive tests

## Expected Outcomes

### main.rs After Refactoring
- **Target Size**: < 200 lines
- **Responsibilities**: 
  - CLI parsing (clap structures)
  - Command dispatch to modules
  - Top-level error handling

### New Module Structure
```
cli/
├── mod.rs         # Public API
├── common.rs      # Shared utilities, ProxyConfig
├── forward.rs     # Forward proxy (stdio + HTTP)
├── reverse.rs     # Reverse proxy
├── record.rs      # Recording (stdio + HTTP)
├── replay.rs      # Replay server
├── handlers.rs    # HTTP request handlers
├── tape.rs        # (existing)
├── intercept.rs   # (existing)
└── session.rs     # (existing)
```

## Risk Mitigation

### Testing Strategy
- Write tests BEFORE extracting each component
- Maintain integration tests throughout
- Use feature flags for gradual rollout

### Backward Compatibility
- Keep exact same CLI interface
- Preserve all command-line arguments
- Maintain same error messages

### Performance Considerations
- No additional allocations in hot paths
- Keep Arc patterns for shared state
- Profile after major extractions