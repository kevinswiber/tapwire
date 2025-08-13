# Task 009.1: Fix Critical Session Cleanup Design Flaws

## Overview
Address critical design flaws discovered in the session cleanup implementation through code review.

## Context
A comprehensive code review (see [session-cleanup-review-2025-08-07.md](../../reviews/session-cleanup-review-2025-08-07.md)) identified several critical issues in Task 009's implementation that must be fixed before the code can be used in production.

## Scope
- **Files to modify**: 
  - `src/session/manager.rs` (primary focus)
  - `src/session/store.rs` (minor updates)
  - `src/cli/session.rs` (validation fixes)
- **Priority**: CRITICAL - System stability
- **Time estimate**: 0.5-1 day

## Critical Issues to Fix

### 🔴 Issue 1: Deadlock in LRU Eviction
**Location**: `src/session/manager.rs:383-402`
**Problem**: `evict_lru_sessions()` holds write lock while calling `delete_session()` which tries to acquire the same lock
**Impact**: Guaranteed system freeze when LRU eviction triggers

### 🔴 Issue 2: Race Condition in Metrics
**Location**: `src/session/manager.rs:341-346`  
**Problem**: Metrics incorrectly assume all deleted sessions were active
**Impact**: Incorrect or negative session counts

### 🟠 Issue 3: Memory Leak in LRU Queue
**Location**: `src/session/manager.rs:134`
**Problem**: Duplicate entries for same session in VecDeque
**Impact**: Unbounded memory growth

### 🟠 Issue 4: O(n) Cleanup Performance
**Location**: `src/session/manager.rs:286-354`
**Problem**: Linear scan of all sessions for cleanup
**Impact**: Performance degradation with many sessions

### 🟠 Issue 5: Missing Backpressure
**Location**: `src/session/manager.rs:505-547`
**Problem**: No rate limiting on request tracking
**Impact**: System can be overwhelmed by rapid requests

## Implementation Plan

### Step 1: Fix Deadlock (CRITICAL)

```rust
async fn evict_lru_sessions(&self, count: usize) -> SessionResult<()> {
    let mut removed = 0;
    
    while removed < count {
        // Acquire lock, pop item, release lock immediately
        let session_to_remove = {
            let mut lru = self.lru_queue.write().await;
            if lru.is_empty() {
                break;
            }
            lru.pop_front()
        };
        
        if let Some((session_id, _)) = session_to_remove {
            if let Err(e) = self.delete_session(&session_id).await {
                error!("Failed to evict LRU session {}: {}", session_id, e);
            } else {
                removed += 1;
            }
        }
    }
    
    if removed > 0 {
        info!("Evicted {} sessions via LRU", removed);
    }
    
    Ok(())
}
```

### Step 2: Fix Race Condition in Metrics

```rust
// Track active sessions separately
let mut active_removed = 0;
for session_id in &removed_ids {
    let session = self.store.get_session(session_id).await?;
    if matches!(session.status, SessionStatus::Active) {
        active_removed += 1;
    }
    if let Err(e) = self.delete_session(session_id).await {
        error!("Failed to delete session {}: {}", session_id, e);
    } else {
        removed_count += 1;
    }
}

self.metrics.sessions_cleaned.fetch_add(removed_count as u64, Ordering::Relaxed);
self.metrics.sessions_active.fetch_sub(active_removed as u64, Ordering::Relaxed);
```

### Step 3: Replace VecDeque with LinkedHashMap for LRU

```rust
use std::collections::LinkedHashMap;

// In struct definition
pub struct SessionManager {
    // ...
    lru_queue: Arc<RwLock<LinkedHashMap<SessionId, SystemTime>>>,
}

// When adding to LRU
{
    let mut lru = self.lru_queue.write().await;
    // Remove existing entry if present
    lru.remove(&session_id);
    // Add at the end (most recently used)
    lru.insert(session_id.clone(), SystemTime::now());
}
```

### Step 4: Add Priority Queue for Efficient Cleanup

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct ExpiryEntry {
    expiry_time: SystemTime,
    session_id: SessionId,
}

// Maintain min-heap of sessions by expiry time
expiry_queue: Arc<RwLock<BinaryHeap<Reverse<ExpiryEntry>>>>,
```

### Step 5: Add Backpressure for Request Tracking

```rust
struct SessionRequestRate {
    last_request: Instant,
    request_count: u32,
    window_start: Instant,
}

// Check rate before accepting new request
const MAX_REQUESTS_PER_SECOND: u32 = 100;
if session_rate.request_count > MAX_REQUESTS_PER_SECOND {
    return Err(SessionError::RateLimited);
}
```

## Additional Improvements

### Medium Priority
- Add panic recovery in cleanup task
- Fix duration parsing validation in CLI
- Use VecDeque for frame storage

### Low Priority  
- Reduce unnecessary clones
- Add instrumentation to critical paths

## Testing Requirements

### New Tests to Add

```rust
#[tokio::test]
async fn test_concurrent_cleanup_no_deadlock() {
    // Test concurrent cleanup and session operations
}

#[tokio::test]
async fn test_lru_eviction_no_duplicates() {
    // Verify LRU queue maintains uniqueness
}

#[tokio::test] 
async fn test_metrics_accuracy_under_load() {
    // Verify metrics remain accurate with concurrent operations
}

#[tokio::test]
async fn test_cleanup_performance_with_1000_sessions() {
    // Benchmark cleanup with max sessions
}
```

## Validation

### Pre-check
```bash
# Test for deadlocks
cargo test evict_lru --release -- --test-threads=1

# Check for memory leaks
valgrind --leak-check=full ./target/debug/shadowcat session cleanup --all

# Verify metrics accuracy
cargo test metrics_accuracy
```

### Post-check
```bash
# All tests pass including new concurrent tests
cargo test session

# No deadlocks under load
./stress-test-sessions.sh

# Memory usage remains bounded
cargo bench session_cleanup
```

## Success Criteria

- [ ] No deadlocks in LRU eviction
- [ ] Metrics accurately track active/cleaned sessions
- [ ] LRU queue maintains unique entries only
- [ ] Cleanup completes in O(log n) for expired sessions
- [ ] Request tracking has proper backpressure
- [ ] All existing tests still pass
- [ ] New concurrent tests pass
- [ ] Memory usage remains bounded under load

## Performance Targets

- Cleanup of 1000 sessions: < 10ms
- LRU eviction: < 1ms per session
- Memory overhead per session: < 1KB
- No observable latency impact on normal operations

## Notes

- This is a critical fix that blocks production deployment
- The deadlock issue is a showstopper and must be fixed immediately
- Consider using `parking_lot::RwLock` for better performance
- After fixes, re-run the rust-code-reviewer to verify all issues resolved

## References

- [Comprehensive Review Document](../../reviews/session-cleanup-review-2025-08-07.md)
- [Original Task 009](./task-009-session-cleanup.md)
- [Shadowcat Refactor Tracker](./shadowcat-refactor-tracker.md)