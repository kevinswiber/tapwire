# E.3: Optimize Memory Usage

**Task ID**: E.3  
**Phase**: Critical Fixes  
**Duration**: 2 hours  
**Dependencies**: E.1  
**Priority**: 🟡 HIGH  
**Status**: ⬜ Not Started

## Problem Statement

Current implementation has multiple memory inefficiencies:
- Event IDs stored as `String` with frequent cloning
- No string interning for common patterns
- Unbounded HashMap growth for sessions
- No LRU eviction for long-running sessions
- ~20KB per session overhead

## Objective

Optimize memory usage to:
- Reduce per-session overhead to < 5KB
- Implement string interning for event IDs
- Add LRU eviction for old sessions
- Use more efficient data structures

## Implementation Steps

### 1. Switch to Arc<str> for Event IDs (30 min)

```rust
// In src/transport/sse/reconnect.rs

pub struct EventTracker {
    // Change from String to Arc<str>
    last_event_id: Arc<RwLock<Option<Arc<str>>>>,
    seen_events: Arc<RwLock<VecDeque<Arc<str>>>>,
    max_tracked: usize,
    on_new_event: Option<Arc<dyn Fn(&str) + Send + Sync>>,
}

impl EventTracker {
    pub async fn record_event(&self, event: &SseEvent) {
        if let Some(ref id) = event.id {
            // Convert to Arc<str> once
            let arc_id: Arc<str> = Arc::from(id.as_str());
            
            // Check for duplicate
            {
                let seen = self.seen_events.read().await;
                if seen.iter().any(|s| Arc::ptr_eq(s, &arc_id) || **s == *arc_id) {
                    return; // Duplicate
                }
            }
            
            // Record event
            {
                let mut seen = self.seen_events.write().await;
                seen.push_back(arc_id.clone());
                if seen.len() > self.max_tracked {
                    seen.pop_front();
                }
            }
            
            // Update last event ID
            *self.last_event_id.write().await = Some(arc_id);
        }
    }
}
```

### 2. Implement String Interning (45 min)

```rust
// In src/transport/sse/intern.rs (new file)

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Thread-safe string interning cache
pub struct StringInterner {
    cache: Arc<RwLock<HashMap<u64, Arc<str>>>>,
    max_entries: usize,
}

impl StringInterner {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::with_capacity(max_entries))),
            max_entries,
        }
    }
    
    pub fn intern(&self, s: &str) -> Arc<str> {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        // Calculate hash
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Check cache
        {
            let cache = self.cache.read();
            if let Some(arc) = cache.get(&hash) {
                if **arc == *s {  // Verify actual string matches
                    metrics::STRING_INTERN_CACHE_HITS.inc();
                    return arc.clone();
                }
            }
        }
        
        metrics::STRING_INTERN_CACHE_MISSES.inc();
        
        // Add to cache
        let arc = Arc::from(s);
        {
            let mut cache = self.cache.write();
            
            // LRU eviction if needed
            if cache.len() >= self.max_entries {
                // Simple eviction: remove random entry
                if let Some(key) = cache.keys().next().cloned() {
                    cache.remove(&key);
                    metrics::STRING_INTERN_EVICTIONS.inc();
                }
            }
            
            cache.insert(hash, arc.clone());
        }
        
        arc
    }
}

// Update EventTracker to use interner
pub struct EventTracker {
    // ... existing fields ...
    interner: StringInterner,
}

impl EventTracker {
    pub async fn record_event(&self, event: &SseEvent) {
        if let Some(ref id) = event.id {
            // Intern the string
            let arc_id = self.interner.intern(id);
            // ... rest of logic using arc_id
        }
    }
}
```

### 3. Add LRU Session Eviction (30 min)

```rust
// In src/session/manager.rs

use lru::LruCache;

pub struct SessionManager {
    // Change from HashMap to LruCache
    sessions: Arc<RwLock<LruCache<SessionId, Arc<Session>>>>,
    event_trackers: Arc<RwLock<LruCache<SessionId, Arc<EventTracker>>>>,
    // ... other fields ...
}

impl SessionManager {
    pub async fn new(config: SessionConfig, store: Arc<dyn SessionStore>) -> Result<Self> {
        Ok(Self {
            sessions: Arc::new(RwLock::new(
                LruCache::new(config.max_sessions.try_into().unwrap())
            )),
            event_trackers: Arc::new(RwLock::new(
                LruCache::new(config.max_sessions.try_into().unwrap())
            )),
            // ... other fields ...
        })
    }
    
    pub fn create_event_tracker(&self, session_id: SessionId) -> Arc<EventTracker> {
        let mut trackers = self.event_trackers.write();
        
        // Check if eviction occurred
        let prev_len = trackers.len();
        
        let tracker = trackers.get_or_insert(session_id.clone(), || {
            Arc::new(EventTracker::new_with_interner(
                self.config.max_pending_per_session,
                self.string_interner.clone(),
            ))
        }).clone();
        
        // Track evictions
        if trackers.len() < prev_len {
            self.eviction_count.fetch_add(1, Ordering::Relaxed);
            metrics::SESSION_LRU_EVICTIONS.inc();
        }
        
        tracker
    }
}
```

### 4. Optimize VecDeque Size (15 min)

```rust
impl EventTracker {
    pub fn new_optimized(max_tracked: usize) -> Self {
        // Pre-allocate with reasonable capacity
        let initial_capacity = max_tracked.min(100);
        
        Self {
            last_event_id: Arc::new(RwLock::new(None)),
            seen_events: Arc::new(RwLock::new(
                VecDeque::with_capacity(initial_capacity)
            )),
            max_tracked,
            interner: StringInterner::new(10000), // Share across events
            on_new_event: None,
        }
    }
}
```

### 5. Add Comprehensive Memory Metrics and Monitoring (10 min)

```rust
pub struct MemoryMetrics {
    event_tracker_bytes: Gauge,
    interned_strings: Gauge,
    interned_cache_hits: Counter,
    interned_cache_misses: Counter,
    session_count: Gauge,
    evicted_sessions: Counter,
    bytes_per_session: Histogram,
    string_intern_ratio: Gauge,
}

impl SessionManager {
    pub async fn report_memory_usage(&self) {
        let sessions = self.sessions.read().len();
        let trackers = self.event_trackers.read().len();
        let interner_stats = self.string_interner.stats();
        
        // Calculate actual memory usage
        let session_memory = sessions * size_of::<Session>();
        let tracker_memory = trackers * size_of::<EventTracker>();
        let interned_memory = interner_stats.cache_size * size_of::<Arc<str>>();
        let total_memory = session_memory + tracker_memory + interned_memory;
        
        // Set metrics
        metrics::MEMORY_TOTAL_BYTES.set(total_memory as f64);
        metrics::MEMORY_SESSION_COUNT.set(sessions as f64);
        metrics::MEMORY_TRACKER_COUNT.set(trackers as f64);
        metrics::MEMORY_INTERNED_STRINGS.set(interner_stats.cache_size as f64);
        metrics::MEMORY_BYTES_PER_SESSION.observe((total_memory / sessions.max(1)) as f64);
        
        // Interning efficiency
        let hit_ratio = interner_stats.hits as f64 / 
                       (interner_stats.hits + interner_stats.misses).max(1) as f64;
        metrics::STRING_INTERN_HIT_RATIO.set(hit_ratio);
        
        // LRU eviction metrics
        metrics::SESSION_EVICTIONS.inc_by(self.eviction_count.swap(0, Ordering::Relaxed));
    }
}
```

## Success Criteria

- [ ] Event IDs use Arc<str> instead of String
- [ ] String interning reduces memory for common patterns
- [ ] LRU eviction prevents unbounded growth
- [ ] Memory per session < 5KB (from ~20KB)
- [ ] No performance regression from optimizations
- [ ] Metrics track memory usage
- [ ] Monitoring metrics exposed:
  - `memory_total_bytes` gauge
  - `memory_session_count` gauge
  - `memory_tracker_count` gauge
  - `memory_interned_strings` gauge
  - `memory_bytes_per_session` histogram
  - `string_intern_hit_ratio` gauge
  - `string_intern_cache_hits` counter
  - `string_intern_cache_misses` counter
  - `session_evictions_total` counter

## Testing

```bash
# Memory usage tests
cargo test session::memory_usage

# Benchmark memory before/after
cargo bench memory_overhead

# Test string interning
cargo test test_string_interning

# LRU eviction test
cargo test test_lru_session_eviction
```

## Performance Targets

- Memory per session: < 5KB (from ~20KB)
- String interning hit rate: > 50% for common patterns
- LRU eviction overhead: < 1ms per eviction
- No increase in lock contention

## Memory Savings Calculation

### Before Optimization
- Event ID: 24 bytes (String) + 8 bytes (pointer) = 32 bytes
- 1000 events: 32KB per session
- 1000 sessions: 32MB total

### After Optimization
- Event ID: 8 bytes (Arc<str> pointer)
- Interned strings: ~10KB shared cache
- 1000 events: 8KB per session
- 1000 sessions: 8MB + 10KB = ~8MB total (75% reduction)

## Notes

- Consider using `parking_lot::RwLock` for better performance
- May need to tune interner cache size based on patterns
- LRU implementation should be non-blocking
- Monitor for memory fragmentation in long-running processes
- All metrics should integrate with existing telemetry
- Consider adding alerts on:
  - `memory_bytes_per_session` > 10KB (regression)
  - `string_intern_hit_ratio` < 0.5 (poor interning)
  - `session_evictions_total` increasing rapidly (memory pressure)
  - `memory_total_bytes` > configured limit
- Use heap profiling tools (heaptrack, valgrind) for deep analysis
- Consider memory allocator tuning (jemalloc, mimalloc)