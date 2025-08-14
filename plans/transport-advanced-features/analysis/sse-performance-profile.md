# SSE Performance Profile Report

## Executive Summary
Analysis of the current SSE transport implementation reveals several optimization opportunities focused on buffer management, string allocations, and parsing efficiency.

## Current Implementation Analysis

### 1. Memory Allocation Patterns

#### Parser (sse/parser.rs)
- **Issue**: Creates new Vec<u8> buffer with 8KB initial capacity per parser instance
- **Impact**: Frequent allocations for each SSE connection
- **Optimization**: Use buffer pool for parser buffers

#### Buffer Management (sse/buffer.rs)
- **Issue**: Creates new buffer per SseStream instance (8KB default)
- **Impact**: No buffer reuse across connections
- **Optimization**: Integrate with global buffer pools

#### Event Parsing (raw/sse.rs)
- **Issue**: Multiple string allocations during event parsing
  - String::from_utf8_lossy creates new strings
  - Event formatting allocates new strings
  - Buffer.push_str causes reallocations
- **Impact**: High allocation rate during streaming
- **Optimization**: Use BytesMut and minimize string conversions

### 2. Hot Paths Identified

#### Primary Hot Path: Event Reception
```
SseRawClient::receive_stream() 
  -> event_rx.recv() 
  -> SseEvent::parse() 
  -> String allocations
```

#### Secondary Hot Path: Buffer Processing
```
SseParser::feed() 
  -> buffer.extend_from_slice() 
  -> parse_events() 
  -> process_line() 
  -> String conversions
```

### 3. Performance Bottlenecks

#### String Processing
- **Current**: Heavy use of String::from_utf8_lossy
- **Issue**: Creates owned strings unnecessarily
- **Fix**: Use str references where possible

#### Buffer Management
- **Current**: No buffer pooling for SSE
- **Issue**: Allocates new buffers per connection
- **Fix**: Implement SSE-specific buffer pool

#### Event Creation
- **Current**: Multiple intermediate allocations
- **Issue**: EventBuilder creates temporary structures
- **Fix**: Direct event construction with pooled buffers

### 4. Reconnection Overhead

#### Current State
- No automatic reconnection logic
- No last-event-id tracking
- No exponential backoff
- Connection drops cause data loss

#### Impact
- Manual reconnection required
- Potential message loss
- No graceful recovery

## Baseline Metrics

### Memory Usage (per connection)
- Parser buffer: 8KB
- Stream buffer: 8KB
- Event buffers: Variable (avg 2KB)
- Total baseline: ~18KB per connection

### Allocation Rate
- Parser: 1 allocation per connection
- Events: 2-3 allocations per event
- Strings: 4-5 allocations per event

### Performance Targets
- Reduce memory usage by >15%
- Improve throughput by >20%
- Zero message loss during reconnects

## Optimization Priorities

### High Priority
1. **Buffer Pooling**: Integrate with existing buffer pool system
2. **String Optimization**: Minimize allocations in parser
3. **Zero-Copy Parsing**: Use references where possible

### Medium Priority
1. **Lazy Parsing**: Defer parsing for large messages
2. **Event Pool**: Reuse event structures
3. **Reconnection Logic**: Add automatic recovery

### Low Priority
1. **Compression**: Add optional compression support
2. **Batching**: Batch small events
3. **Metrics**: Add detailed performance metrics

## Recommended Approach

### Phase 1: Buffer Pool Integration (Quick Win)
- Add SSE_POOL to global_pools
- Modify parser to use pooled buffers
- Update SseStream to reuse buffers

### Phase 2: String Optimization
- Replace String::from_utf8_lossy with str references
- Use BytesMut for event data
- Minimize intermediate strings

### Phase 3: Reconnection Logic
- Implement exponential backoff
- Add last-event-id tracking
- Ensure message ordering

## Expected Impact

### Memory Reduction
- Buffer pooling: -8KB per connection (reused)
- String optimization: -30% allocation rate
- Total expected: >15% memory reduction

### Performance Improvement
- Reduced allocations: +15% throughput
- Zero-copy parsing: +10% throughput
- Total expected: >20% performance gain

## Next Steps
1. Implement buffer pool integration
2. Optimize string handling in parser
3. Add reconnection logic with exponential backoff
4. Benchmark improvements
5. Update documentation