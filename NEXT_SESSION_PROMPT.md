# MCP Compliance Phase 0 - Task 0.4: Add Version State Management

## Context

You are working on the Shadowcat MCP proxy implementation in the Tapwire project. The project is currently at 10% completion (3 of 29 tasks) for MCP compliance, with Phase 0 at 60% complete (3 of 5 tasks).

### What Has Been Completed

1. **Task 0.1**: Fix Initialize Version Extraction ✅
   - Created centralized protocol module for version management
   - Removed all non-compliant "2025-11-05" references
   - Added VersionInfo struct with negotiation tracking
   - Implemented backward compatibility between versions

2. **Task 0.2**: Fix HTTP Default Version ✅
   - Changed HTTP default from "2025-11-05" to "2025-03-26"
   - Added HTTP_DEFAULT_VERSION constant in protocol module
   - Updated all HTTP header extraction to use centralized constant

3. **Task 0.3**: Implement Version Negotiation Response ✅
   - Created `protocol/negotiation.rs` with VersionNegotiator
   - Modified forward proxy to intercept initialize responses
   - Track initialize requests by ID for response matching
   - Negotiate versions when client/server mismatch
   - Fixed memory leak and added TTL-based cleanup for tracked requests
   - Added LATEST_SUPPORTED_VERSION constant for maintainability

### Recent Improvements
- Fixed critical memory leak in initialize request tracking
- Implemented TTL-based cleanup (60s) for orphaned requests
- Added bounded size protection (max 1000 tracked requests)
- Switched from Mutex to RwLock for better performance
- All tests passing with no clippy warnings

## Current Task: Task 0.4 - Add Version State Management

### Objective

Create a comprehensive version state management system that tracks the complete version lifecycle throughout a session, including requested, negotiated, and transport versions. This will ensure proper version consistency across all protocol layers and enable better debugging of version-related issues.

### Working Directory
```
/Users/kevin/src/tapwire/shadowcat
```

### Essential Context Files to Read

1. **Task Specification** (if exists):
   ```
   plans/mcp-compliance/tasks/phase-0-task-004-version-state-management.md
   ```

2. **Current Implementation**:
   - `src/session/store.rs` - Current VersionInfo struct (lines 99-108)
   - `src/session/manager.rs` - Session management and version tracking
   - `src/protocol/mod.rs` - Protocol constants and version handling
   - `src/protocol/negotiation.rs` - Version negotiation logic
   - `src/proxy/forward.rs` - Forward proxy with version negotiation
   - `src/transport/http_mcp.rs` - HTTP transport version handling

3. **Architecture Documents**:
   - `plans/mcp-compliance/005-multi-version-architecture-design.md` - Multi-version architecture
   - `plans/mcp-compliance/006-critical-version-bugs.md` - Bug #4: Version State Not Tracked Properly

### Implementation Strategy

#### Phase 1: Design VersionState Structure
1. Create comprehensive VersionState struct to replace simple VersionInfo
2. Track multiple version sources:
   - Requested version (from client initialize)
   - Negotiated version (after handshake)
   - Transport version (from HTTP headers)
   - Negotiation method (initialize-only vs dual-channel)

#### Phase 2: Implement State Transitions
1. Define valid state transitions
2. Add validation for version changes
3. Prevent renegotiation after initial handshake
4. Handle version conflicts between channels

#### Phase 3: Integrate with Session Management
1. Update Session struct to use new VersionState
2. Migrate existing VersionInfo usage
3. Update session manager to track state changes
4. Add proper error handling for invalid transitions

#### Phase 4: Add Dual-Channel Validation
1. Validate HTTP header matches negotiated version (for 2025-06-18+)
2. Detect and reject version conflicts
3. Add proper error types for version mismatches
4. Update reverse proxy to use version state

#### Phase 5: Testing and Documentation
1. Unit tests for state transitions
2. Integration tests for version tracking
3. Tests for dual-channel consistency
4. Update documentation

### Detailed Objectives

1. **Create VersionState struct** with:
   - `requested: Option<String>` - Version from initialize request
   - `negotiated: Option<String>` - Version after negotiation
   - `transport_version: Option<String>` - Version from HTTP headers
   - `negotiation_method: NegotiationMethod` enum (InitializeOnly, DualChannel)
   - `state: VersionStatePhase` enum (Uninitialized, Requested, Negotiated, Validated)

2. **Implement state machine** for version transitions:
   - Uninitialized → Requested (on initialize request)
   - Requested → Negotiated (on initialize response)
   - Negotiated → Validated (on HTTP header match for dual-channel)
   - Prevent invalid transitions and renegotiation

3. **Add validation methods**:
   - `validate_transition()` - Check if state change is allowed
   - `validate_consistency()` - Check dual-channel consistency
   - `is_finalized()` - Check if version is locked
   - `get_active_version()` - Get the current active version

4. **Update existing code**:
   - Replace VersionInfo with VersionState in Session
   - Update session manager to use new state methods
   - Modify forward/reverse proxies to track state properly
   - Add logging for state transitions

### Success Criteria Checklist

- [ ] VersionState struct created with all required fields
- [ ] State machine implemented with proper transitions
- [ ] Validation methods prevent invalid state changes
- [ ] Session struct updated to use VersionState
- [ ] Forward proxy tracks version state correctly
- [ ] Reverse proxy validates dual-channel consistency
- [ ] All existing tests still pass
- [ ] New tests for version state management
- [ ] Tests for invalid transition rejection
- [ ] Tests for dual-channel validation
- [ ] No clippy warnings
- [ ] Code properly formatted with cargo fmt
- [ ] Documentation updated
- [ ] Compliance tracker updated

### Commands to Use

```bash
# Navigate to shadowcat directory
cd /Users/kevin/src/tapwire/shadowcat

# Run tests frequently
cargo test version
cargo test session

# Run specific test modules
cargo test session::store::tests
cargo test protocol::

# Check for compilation errors
cargo build

# Format code
cargo fmt

# Run clippy
cargo clippy --all-targets -- -D warnings

# Run all tests
cargo test

# Commit changes (in shadowcat repo)
git add -A
git commit -m "feat: implement comprehensive version state management"
git push

# Update parent repo
cd ..
git add shadowcat
git commit -m "feat: add version state management to shadowcat"
git push
```

### Expected Deliverables

1. **New file**: `src/protocol/version_state.rs`
   - VersionState struct
   - VersionStatePhase enum
   - NegotiationMethod enum
   - State transition logic
   - Validation methods

2. **Modified files**:
   - `src/session/store.rs` - Replace VersionInfo with VersionState
   - `src/session/manager.rs` - Update to use new state management
   - `src/proxy/forward.rs` - Track state transitions
   - `src/proxy/reverse.rs` - Add dual-channel validation
   - `src/protocol/mod.rs` - Export new version_state module

3. **Test files**:
   - Add tests in version_state.rs module
   - Update existing session tests
   - Add integration tests for state tracking

## Important Notes

- **Always use TodoWrite tool** to track your progress through the task
- **Start with examining existing code** to understand current architecture
- **Follow established patterns** from previous implementations
- **Test incrementally** as you build each component
- **Run `cargo fmt`** after implementing new functionality
- **Run `cargo clippy --all-targets -- -D warnings`** before any commit
- **Update the compliance tracker** when the task is complete
- **Focus on the current phase objectives**

## Model Usage Guidelines

- **IMPORTANT** Be mindful of model capabilities. Assess whether Claude Opus or Claude Sonnet would be best for each step. When there's a benefit to a model change, pause and recommend it. Be mindful of the context window. When the context window has less than 15% availability, suggest creating a new Claude session and output a good prompt, referencing all available plans, tasks, and completion files that are relevant. Save the prompt into NEXT_SESSION_PROMPT.md.

## Development Workflow

1. Create todo list with TodoWrite tool to track progress
2. Examine existing codebase architecture and established patterns
3. Study current implementations related to the task
4. Design the solution approach and identify key components
5. Implement functionality incrementally with frequent testing
6. Add comprehensive error handling following project patterns
7. Create tests demonstrating functionality works correctly
8. Run tests after each significant change to catch issues early
9. Run `cargo fmt` to ensure consistent code formatting
10. Run `cargo clippy -- -D warnings` to catch potential issues
11. Update project documentation and tracker as needed
12. Commit changes with clear, descriptive messages

## Current Phase Status

**Phase 0: Critical Version Bug Fixes**
- Task 0.1: Fix Initialize Version Extraction ✅
- Task 0.2: Fix HTTP Default Version ✅
- Task 0.3: Implement Version Negotiation Response ✅
- **Task 0.4: Add Version State Management** 🎯 CURRENT
- Task 0.5: Handle Dual-Channel Version Conflicts ⏳

After completing Task 0.4, Task 0.5 will be ready to start, which will complete Phase 0.

## Key Technical Context

### Current Version Support
- **Minimum supported**: 2025-03-26 (initialize-only negotiation)
- **Current target**: 2025-06-18 (dual-channel negotiation)
- **Latest supported**: 2025-06-18 (defined in LATEST_SUPPORTED_VERSION constant)

### Version Negotiation Flow
1. Client sends initialize with protocolVersion
2. Server responds with same or alternative version
3. For 2025-06-18+: HTTP headers must match negotiated version
4. Proxy must track and validate consistency

### Known Issues to Address
- Version state is not properly tracked through session lifecycle
- No validation of state transitions
- Dual-channel consistency not enforced (only warned)
- No clear separation between negotiation methods

## Notes from Previous Session

- Version negotiation is working but needs better state management
- Memory leak in request tracking has been fixed
- TTL-based cleanup is implemented (60 seconds)
- Bounded size protection prevents DoS attacks
- RwLock improves concurrent read performance
- All constants are centralized for maintainability

Start by reading the existing VersionInfo implementation and understanding how it's currently used throughout the codebase, then design the enhanced VersionState structure to properly track the complete version lifecycle.