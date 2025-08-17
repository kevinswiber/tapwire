# Simplified Traffic Recording Plan

## The Real Problem

We need to get 3 SSE metadata fields (event_id, event_type, retry_ms) from the transport layer to the recording layer for faithful replay. That's it.

## Why the Original Plan is Overly Complex

The original plan proposes:
1. Creating a new `RawWireData` structure with Arc-wrapped bytes
2. Passing raw wire data alongside MessageEnvelope through multiple layers
3. Extracting SSE metadata from raw bytes in the recording layer
4. Complex memory management with Arc

This is architectural astronautics for passing 3 optional strings/numbers!

## The Pragmatic Solution

### Core Insight
Yes, SSE metadata is "wire format details" that don't belong in TransportContext in a perfect world. But we're pre-release, and pragmatism beats purity.

### Simplified Approach

#### Step 1: Add SSE metadata to TransportContext::Http (1 hour)
```rust
pub enum TransportContext {
    Stdio { /* unchanged */ },
    Http {
        method: String,
        path: String,
        headers: HashMap<String, String>,
        status_code: Option<u16>,
        remote_addr: Option<String>,
        response_mode: Option<ResponseMode>,  // Already planned
        
        // Just add this - yes, it's not pure, but it works
        sse_metadata: Option<SseMetadata>,    // NEW
    }
    Sse { /* Will be removed */ }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SseMetadata {
    pub event_id: Option<String>,
    pub event_type: Option<String>,
    pub retry_ms: Option<u64>,
}
```

#### Step 2: Update SSE transports (1 hour)
When creating Http context for SSE responses:
```rust
TransportContext::Http {
    method: "GET".to_string(),
    path: self.path.clone(),
    headers: headers.clone(),
    status_code: Some(200),
    remote_addr: self.remote_addr.clone(),
    response_mode: Some(ResponseMode::SseStream),
    sse_metadata: Some(SseMetadata {
        event_id: sse_event.id,
        event_type: Some(sse_event.event_type),
        retry_ms: sse_event.retry,
    }),
}
```

#### Step 3: Update recording layer (30 min)
```rust
match &envelope.context.transport {
    TransportContext::Http { sse_metadata, response_mode, .. } => {
        // If it's an SSE stream with metadata, record it
        if matches!(response_mode, Some(ResponseMode::SseStream)) {
            if let Some(sse) = sse_metadata {
                metadata.sse_metadata = Some(SseMetadata {
                    event_id: sse.event_id.clone(),
                    event_type: sse.event_type.clone(),
                    retry_ms: sse.retry_ms,
                    last_event_id: None,
                });
            }
        }
        // Continue with regular HTTP metadata...
    }
    TransportContext::Sse { .. } => {
        // This arm goes away
    }
}
```

#### Step 4: Remove TransportContext::Sse (30 min)
- Delete the variant
- Fix compilation errors
- All SSE contexts now use Http with sse_metadata

## Comparison with Original Plan

### Original Plan Complexity
- New `RawWireData` struct with Arc wrapping
- New `record_frame_with_raw` method
- Passing raw bytes through proxy layer
- Complex SSE parsing in recording layer
- ~17 hours of work

### Simplified Plan
- Add one field to existing struct
- Update a few context creation sites
- Simple extraction in recording layer
- ~3 hours of work

## Trade-offs

### What We Lose
- Architectural purity (SSE metadata in TransportContext)
- Ability to record exact wire format (but do we need this?)

### What We Gain  
- 14 hours of saved work
- Simpler codebase
- Less code to maintain
- Can ship faster

## Why This is OK

1. **We're pre-release** - Perfect is the enemy of good
2. **SSE metadata is tiny** - 3 optional fields, not worth complex infrastructure
3. **It's typed** - Using SseMetadata struct, not HashMap
4. **Easy to refactor later** - If we need RawWireData approach, we can add it
5. **Solves the immediate problem** - Gets metadata to recording layer

## Implementation Order

1. **Add response_mode to Http** (already partially done)
2. **Add sse_metadata to Http** 
3. **Update SSE transport creation**
4. **Update recording extraction**
5. **Remove Sse variant**
6. **Test with recordings**

Total: ~3 hours vs 17 hours

## What About the Other Issues?

### Duplicate SseEvent types
✅ Already fixed - consolidated to just the canonical one and Vec<u8> buffering

### ResponseMode 
Good idea, keep it - helps identify response format

### Type safety
We're still using typed SseMetadata, not HashMap

## Conclusion

The original plan is solving an architectural purity problem that doesn't need solving right now. We have a simple need: pass 3 fields from transport to recording. The pragmatic solution is to just add them to TransportContext::Http with a guard (ResponseMode::SseStream).

This gets us:
- Working SSE recording in 3 hours instead of 17
- Simpler code that's easier to understand
- Can always refactor later if needed

## Next Steps

1. Get buy-in on this simplified approach
2. Implement in 3 hours
3. Move on to more important work
4. Consider architectural improvements post-release if needed