# The Real Problem: We're Throwing Away SSE Metadata!

## The Revelation

After understanding that TransportContext is message-level (not session-level), we realize:
- `TransportContext::Sse` is CORRECT and should exist
- The three variants (Stdio, Http, Sse) represent the three ways a message can be delivered
- This is not a code smell at all!

## The Actual Problem

Looking at `transport/outgoing/http.rs`, we found the real issue:

```rust
// Line 157: We extract only the data, throwing away metadata!
fn extract_sse_data(event: ParsedSseEvent) -> Vec<u8> {
    event.data.into_bytes()  // Lost: event.id, event.event_type, event.retry
}

// Line 195: We buffer only the data bytes
let data = Self::extract_sse_data(event);
buf.push_back(data);  // Just Vec<u8>, no metadata

// Line 170: Later, we create context with empty metadata!
transport: TransportContext::sse(),  // All fields are None!
```

**We're throwing away the SSE event metadata when we buffer events!**

## The Simple Fix

### Option 1: Buffer Full SseEvent (Recommended)
Instead of buffering `Vec<u8>`, buffer the full `SseEvent`:

```rust
// Change buffer type
sse_event_buffer: Arc<Mutex<VecDeque<SseEvent>>>,  // Not Vec<u8>

// Keep full event when buffering
buf.push_back(event);  // Keep the whole event

// Create context with metadata
fn create_sse_envelope(&self, event: SseEvent) -> TransportResult<MessageEnvelope> {
    // Deserialize the data
    let message = self.protocol.deserialize(event.data.as_bytes())?;
    
    // Create context WITH metadata
    let context = MessageContext {
        transport: TransportContext::Sse {
            event_type: Some(event.event_type),
            event_id: event.id,
            retry_ms: event.retry,
            headers: HashMap::new(),  // Could add HTTP headers if needed
        },
        // ... rest of context
    };
    
    Ok(MessageEnvelope::new(message, context))
}
```

### Option 2: Tuple Buffer
Buffer `(Vec<u8>, Option<String>, Option<String>, Option<u64>)` but that's uglier.

## Why This is So Much Simpler

Our previous plans involved:
- Creating RawWireData structures
- Passing raw bytes through layers
- Complex Arc wrapping
- 17+ hours of work

The actual fix:
- Change buffer from `Vec<u8>` to `SseEvent` 
- Update create_sse_envelope to use the metadata
- 30 minutes of work!

## Impact on Recording

With this fix, the recording layer will automatically get the SSE metadata because:
1. TransportContext::Sse will have the actual values (not None)
2. Recording layer already extracts from TransportContext::Sse correctly
3. No changes needed to recording layer at all!

## Implementation Steps

1. **Revert the simplification** (10 min)
   - Change back from `Vec<u8>` to buffering `SseEvent`
   - We already had this before we "simplified" it!

2. **Update create_sse_envelope** (10 min)
   - Accept `SseEvent` instead of `Vec<u8>`
   - Populate TransportContext::Sse with actual metadata

3. **Test** (10 min)
   - Verify SSE metadata appears in recordings
   - Test replay with SSE events

Total: 30 minutes

## The Lessons Learned

1. **Understand the abstraction level** - TransportContext is message-level, not session-level
2. **Follow the data** - We were losing metadata at the buffering step
3. **Simple problems have simple solutions** - We don't need complex architectures
4. **TransportContext::Sse is correct** - It represents a distinct message delivery mechanism

## Conclusion

We don't need to remove TransportContext::Sse or create complex RawWireData structures. We just need to stop throwing away the SSE metadata when we buffer events. The fix is trivial - buffer the full SseEvent instead of just the data bytes.