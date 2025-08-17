# C.2: Remove Redundant Event Tracking

**Task ID**: C.2  
**Phase**: Cleanup  
**Duration**: 1 hour  
**Dependencies**: C.1  
**Status**: ⬜ Not Started

## Objective

Remove duplicate event ID tracking from ConnectionInfo and other redundant locations now that EventTracker is the single source of truth.

## Current Redundant Tracking Locations

Based on analysis, these locations have redundant tracking:

1. **`src/session/sse_integration.rs`** - ConnectionInfo struct
   - Has `last_event_id` field
   - Has `set_last_event_id()` method
   - Tracks per-connection instead of per-session

2. **`src/session/store.rs`** - Session struct
   - Has `last_event_id` field (keep for now, used by store)
   - Has getter/setter methods

## Implementation Steps

### 1. Remove from ConnectionInfo (30 min)

```rust
// In src/session/sse_integration.rs

// REMOVE these fields from ConnectionInfo:
pub struct ConnectionInfo {
    // pub last_event_id: Option<String>,  // DELETE THIS
    // ... other fields remain
}

// REMOVE these methods:
// pub fn set_last_event_id(&mut self, event_id: String) { ... }
// pub fn get_last_event_id(&self) -> Option<&str> { ... }
```

### 2. Update SseSessionState (20 min)

```rust
// Update to use shared tracker instead of local tracking
pub struct SseSessionState {
    // Remove any local event tracking
    // Use session_manager.create_event_tracker() instead
}
```

### 3. Update Call Sites (10 min)

Find and update all places that called the removed methods:

```bash
# Find all references to update
grep -r "set_last_event_id" src/
grep -r "get_last_event_id" src/
grep -r "last_event_id" src/session/sse_integration.rs
```

Replace with calls to EventTracker methods or SessionManager methods.

## What to Keep

Keep these as they're part of the unified design:

1. **Session.last_event_id** - Used by SessionStore trait for persistence
2. **EventTracker** - The single source of truth
3. **SessionStore methods** - store_last_event_id(), get_last_event_id()

## Success Criteria

- [ ] ConnectionInfo has no event tracking fields
- [ ] No duplicate set_last_event_id() methods
- [ ] All call sites updated to use EventTracker
- [ ] No compilation errors
- [ ] All tests pass

## Testing

```bash
# Compile check
cargo check

# Run session tests
cargo test session::

# Run SSE integration tests  
cargo test sse_integration

# Full test suite
cargo test
```

## Benefits

- Single source of truth (EventTracker)
- No synchronization issues between trackers
- Cleaner code with clear responsibilities
- Automatic persistence via callbacks

## Notes

- Be careful not to remove Session.last_event_id as it's used by the store
- The SessionStore trait methods are the persistence layer, not tracking
- EventTracker owns deduplication logic