# SSE Reconnection Retry-After Header Refactor

## Current State

The SSE reconnection logic in `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/reconnect.rs` currently handles HTTP status codes for retry decisions but doesn't respect the `Retry-After` header that servers may send with 429 (Too Many Requests) or 503 (Service Unavailable) responses.

## Problem

When a server responds with 429 or 503, it often includes a `Retry-After` header indicating when the client should retry:
- As a number of seconds: `Retry-After: 120`
- As an HTTP-date: `Retry-After: Wed, 21 Oct 2025 07:28:00 GMT`

Currently, we use exponential backoff regardless of what the server suggests, which could lead to:
1. Retrying too soon and getting rejected again
2. Waiting longer than necessary when the server is ready

## Proposed Design

### 1. Enhanced Error Type

Update `SseConnectionError::Http` to include retry information:

```rust
// In src/transport/sse/connection.rs
#[derive(Debug, thiserror::Error)]
pub enum SseConnectionError {
    #[error("HTTP error{}: {message}", status.map(|s| format!(" {s}")).unwrap_or_default())]
    Http { 
        status: Option<u16>, 
        message: String,
        retry_after: Option<RetryAfter>,  // NEW
    },
    // ... other variants
}

#[derive(Debug, Clone)]
pub enum RetryAfter {
    /// Retry after this many seconds
    Delay(Duration),
    /// Retry at this specific time
    DateTime(SystemTime),
}

impl RetryAfter {
    /// Calculate how long to wait from now
    pub fn duration_from_now(&self) -> Duration {
        match self {
            RetryAfter::Delay(d) => *d,
            RetryAfter::DateTime(when) => {
                when.duration_since(SystemTime::now())
                    .unwrap_or(Duration::ZERO)
            }
        }
    }
    
    /// Parse from HTTP header value
    pub fn from_header_value(value: &str) -> Option<Self> {
        // First try parsing as seconds (more common)
        if let Ok(seconds) = value.parse::<u64>() {
            return Some(RetryAfter::Delay(Duration::from_secs(seconds)));
        }
        
        // Then try parsing as HTTP-date
        if let Ok(date) = httpdate::parse_http_date(value) {
            return Some(RetryAfter::DateTime(date));
        }
        
        None
    }
}
```

### 2. Update Error Creation Sites

In `src/transport/sse/client.rs`, extract Retry-After header when creating errors:

```rust
// Example for one location - apply pattern to all HTTP error creation sites
if !response.status().is_success() {
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(RetryAfter::from_header_value);
    
    return Err(SseConnectionError::Http {
        status: Some(response.status().as_u16()),
        message: format!("HTTP error: {}", response.status()),
        retry_after,  // NEW
    });
}
```

### 3. Update Reconnection Logic

In `src/transport/sse/reconnect.rs`, modify the retry logic to respect Retry-After:

```rust
impl ReconnectingStream {
    /// Update retry delay based on server hint or SSE retry field
    pub fn update_retry_delay(&mut self, delay: Duration) {
        self.retry_override = Some(delay);
        debug!("Updated retry delay to {}s", delay.as_secs());
    }
}

// In poll_next(), when handling connection failure:
Poll::Ready(Err(e)) => {
    error!("Reconnection attempt {} failed: {}", *attempt + 1, e);
    
    // Extract retry hint from error
    let retry_hint = match &e {
        SseConnectionError::Http { retry_after: Some(retry), .. } => {
            Some(retry.duration_from_now())
        }
        _ => None
    };
    
    if this.manager.strategy.should_retry(&e, *attempt + 1) {
        // Use server's retry hint if available, otherwise exponential backoff
        let delay = retry_hint
            .or(this.retry_override)
            .unwrap_or_else(|| this.manager.strategy.next_delay(*attempt + 1));
        
        // Cap the delay to something reasonable (e.g., 5 minutes)
        let delay = delay.min(Duration::from_secs(300));
        
        info!("Will retry in {}s (hint: {})", 
            delay.as_secs(), 
            retry_hint.is_some());
        
        this.state = StreamState::Reconnecting {
            attempt: *attempt + 1,
            next_retry: Instant::now() + delay,
        };
        // ... rest of existing code
    }
}
```

### 4. Create Reusable Retry Module

For broader reuse across the project, create a new module:

```rust
// src/retry/mod.rs
pub mod strategy;
pub mod http;

// src/retry/http.rs
use hyper::HeaderMap;
use std::time::{Duration, SystemTime};

/// Extract retry guidance from HTTP response headers
pub struct HttpRetryInfo {
    pub retry_after: Option<RetryAfter>,
    pub rate_limit_remaining: Option<u32>,
    pub rate_limit_reset: Option<SystemTime>,
}

impl HttpRetryInfo {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let retry_after = headers
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(RetryAfter::from_header_value);
        
        let rate_limit_remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());
        
        let rate_limit_reset = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(|secs| SystemTime::UNIX_EPOCH + Duration::from_secs(secs));
        
        Self {
            retry_after,
            rate_limit_remaining,
            rate_limit_reset,
        }
    }
    
    /// Calculate optimal retry delay based on all available information
    pub fn suggested_delay(&self) -> Option<Duration> {
        // Prefer explicit Retry-After
        if let Some(retry) = &self.retry_after {
            return Some(retry.duration_from_now());
        }
        
        // Fall back to rate limit reset if we're out of requests
        if self.rate_limit_remaining == Some(0) {
            if let Some(reset) = self.rate_limit_reset {
                return Some(
                    reset.duration_since(SystemTime::now())
                        .unwrap_or(Duration::from_secs(60))
                );
            }
        }
        
        None
    }
}

// src/retry/strategy.rs
/// Trait for retry strategies that can incorporate server hints
pub trait RetryStrategy: Send + Sync {
    /// Calculate next delay, optionally using server hint
    fn next_delay_with_hint(
        &self, 
        attempt: usize, 
        hint: Option<Duration>
    ) -> Duration {
        hint.unwrap_or_else(|| self.next_delay(attempt))
    }
    
    fn next_delay(&self, attempt: usize) -> Duration;
    fn should_retry(&self, error: &dyn std::error::Error, attempt: usize) -> bool;
}
```

## Progress Update (Phase 1 Complete)

Date: 2025-08-08

We implemented Phase 1 basic Retry-After support in the SSE transport:

- Enhanced error type in `shadowcat/src/transport/sse/connection.rs`:
  - Added `retry_after: Option<RetryAfter>` to `SseConnectionError::Http`.
  - Introduced `RetryAfter` enum with `Delay(Duration)` and `DateTime(SystemTime)` variants, including `duration_from_now()` and `from_header_value()` (using `httpdate`).
- Updated HTTP client in `shadowcat/src/transport/sse/client.rs` to extract `Retry-After` for all HTTP error creation paths (POST/GET, including JSON parse errors).
- Refined reconnection behavior in `shadowcat/src/transport/sse/reconnect.rs`:
  - `ReconnectingStream::update_retry_delay` now accepts `Duration` (instead of `u64` ms).
  - On failed reconnection with `SseConnectionError::Http { retry_after: Some(..) }`, we compute a hint via `duration_from_now()` and use that delay instead of pure exponential backoff, with a safety cap of 5 minutes.
  - Existing exponential backoff remains the fallback when no hint is available.
- Dependency added/updated in `shadowcat/Cargo.toml`:
  - `httpdate = "1.0.3"`.

Notes:
- To keep builds warning-free, the `RetryAfter` import in `reconnect.rs` is gated behind `#[cfg(test)]` and only pulled into scope for tests.
- The SSE event `retry` field (if present) continues to override via `update_retry_delay(Duration::from_millis(ms))`.

### Tests Added

- `shadowcat/src/transport/sse/connection.rs`:
  - `test_retry_after_seconds` – parses numeric `Retry-After`.
  - `test_retry_after_http_date` – parses HTTP-date `Retry-After` (uses the RFC 7231 canonical date).
- `shadowcat/src/transport/sse/reconnect.rs`:
  - `test_reconnect_uses_retry_after_hint` – verifies reconnection uses a server-provided `Retry-After` hint (Delay of 100ms) rather than base exponential backoff.

All tests are green locally after the change.

## Implementation Steps

1. **Phase 1: Basic Retry-After Support**
   - Add `retry_after` field to `SseConnectionError::Http`
   - Update error creation in `client.rs` to extract header
   - Modify reconnection logic to use retry hint

2. **Phase 2: Comprehensive Rate Limit Support**
   - Add support for X-RateLimit-* headers
   - Create HttpRetryInfo struct for all retry-related headers
   - Test with real rate-limited APIs

3. **Phase 3: Reusable Retry Module**
   - Extract retry logic into standalone module
   - Make it generic for use with regular HTTP requests
   - Add to reverse proxy for upstream retry logic

## Testing Approach

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retry_after_seconds() {
        let retry = RetryAfter::from_header_value("120").unwrap();
        match retry {
            RetryAfter::Delay(d) => assert_eq!(d, Duration::from_secs(120)),
            _ => panic!("Expected Delay variant"),
        }
    }
    
    #[test]
    fn test_retry_after_http_date() {
        let retry = RetryAfter::from_header_value(
            "Wed, 21 Oct 2025 07:28:00 GMT"
        ).unwrap();
        match retry {
            RetryAfter::DateTime(_) => (), // Just check variant
            _ => panic!("Expected DateTime variant"),
        }
    }
    
    #[tokio::test]
    async fn test_reconnection_with_retry_after() {
        // Mock server that returns 429 with Retry-After: 5
        // Verify we wait 5 seconds, not exponential backoff
    }
}
```

## Benefits

1. **Server-Friendly**: Respects server's rate limiting and availability windows
2. **Faster Recovery**: Can resume as soon as server is ready instead of waiting for exponential backoff
3. **Reusable**: The retry module can be used throughout the codebase
4. **Standards Compliant**: Follows RFC 7231 (Retry-After) and common rate limit header conventions

## Dependencies

Add to Cargo.toml:
```toml
httpdate = "1.0.3"  # For parsing HTTP-date format
```

## Migration Notes

- This change is backward compatible - if no Retry-After header is present, we fall back to exponential backoff
- The `retry_after` field in the error is `Option<RetryAfter>` so existing code continues to work
- Consider adding metrics/logging to track how often we use server hints vs exponential backoff

## Future Enhancements

1. **Adaptive Strategy**: Learn from successful retries to optimize future retry delays
2. **Circuit Breaker**: If retry hints consistently fail, switch to circuit breaker pattern
3. **Jitter with Hints**: Add small jitter even to server-provided delays to prevent thundering herd
4. **Per-Endpoint Tracking**: Track retry patterns per endpoint for smarter retries

## Related Files

- `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/reconnect.rs` - Main reconnection logic
- `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/connection.rs` - Error types
- `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/client.rs` - HTTP client creating errors
- `/Users/kevin/src/tapwire/shadowcat/src/proxy/reverse.rs` - Could benefit from same retry logic

## Next Steps

- Phase 2: Comprehensive rate limit support using `X-RateLimit-*` and `RateLimit-*` headers. Create an `HttpRetryInfo` struct and feed its `suggested_delay()` as a hint to the reconnection strategy. Add metrics/logging to track server-hint vs backoff usage.
- Phase 3: Extract a reusable retry module usable by both SSE and regular HTTP flows (e.g., reverse proxy). Unify strategy interfaces and ensure type-safe, testable injection of hints.

## Context for Next Session

---

# Phase 2: Comprehensive Rate Limit Support (Plan)

Goal: Go beyond `Retry-After` and incorporate rate limiting headers to compute smarter, standards-aware retry delays.

## Headers to Support

- Retry hint headers (already supported):
  - `Retry-After` (seconds or HTTP-date)
- Common rate limit headers (vendor-specific):
  - `X-RateLimit-Remaining` (int)
  - `X-RateLimit-Reset` (epoch seconds, ms, or HTTP-date depending on provider)
  - `X-RateLimit-Limit` (optional, int)
- IETF standard-style rate limit headers (RFC draft):
  - `RateLimit-Limit`
  - `RateLimit-Remaining`
  - `RateLimit-Reset` (delta seconds)

Notes:
- When multiple header families are present, prefer the most explicit value (see Algorithm below).
- Normalize `Reset` values to `SystemTime` to compute a delay from now.

## Data Model

Introduce `HttpRetryInfo` to capture all retry-related signals in one place.

```rust
// src/retry/http.rs (planned)
pub struct HttpRetryInfo {
    pub retry_after: Option<RetryAfter>,
    pub rate_limit_remaining: Option<u32>,
    pub rate_limit_reset_at: Option<SystemTime>,
    pub rate_limit_limit: Option<u32>,
}

impl HttpRetryInfo {
    pub fn from_headers(headers: &HeaderMap) -> Self { /* parse */ }

    pub fn suggested_delay(&self, now: SystemTime) -> Option<Duration> {
        // 1) Prefer explicit Retry-After
        // 2) If Remaining == 0 and Reset is available, wait until Reset
        // 3) Otherwise None
    }
}
```

Integration options (SSE Phase 2):
- Extend `SseConnectionError::Http` with `retry_info: Option<HttpRetryInfo>`, OR
- Keep `HttpRetryInfo` internal to the client and surface only a computed hint delay (keep `retry_after` as is).

Decision: Extend the error with `retry_info` for debuggability and future reuse. The reconnection logic will then prefer `retry_info.suggested_delay()` if present; otherwise fallback to `retry_after` and finally backoff.

## Algorithm

Given a response with headers H and time now N:

1. If `Retry-After` is present, compute delay D = `RetryAfter::duration_from_now()`.
2. Else, if `RateLimit-Remaining == 0` and `RateLimit-Reset` present, compute D = `Reset - N` (min 0).
3. Else, None.

Then:
- Apply small jitter (e.g., ±10%) to D.
- Cap at a sane maximum (e.g., 5 minutes, configurable via `ReconnectionConfig`).
- Record metrics and logs indicating whether a server hint was used and which header produced it.

## Implementation Tasks

1) Create `src/retry/http.rs` with `HttpRetryInfo` and parsing helpers (including robust parsers for epoch seconds, ms, and HTTP-date).
2) Parse headers in `shadowcat/src/transport/sse/client.rs` for non-success responses:
   - Build `HttpRetryInfo` and attach it to `SseConnectionError::Http { retry_info: Some(..) }`.
   - Keep existing `retry_after` field for compatibility; `retry_info.suggested_delay()` gets precedence if available.
3) Update `shadowcat/src/transport/sse/reconnect.rs` to:
   - Extract `retry_info.suggested_delay()` first; fallback to `retry_after.duration_from_now()`; fallback to backoff.
   - Preserve current cap (5 minutes) and jitter behavior.
4) Add structured logging:
   - When a hint is applied, log the source (Retry-After, X-RateLimit, RateLimit-*), the computed delay, and the attempt number.
5) Add metrics (names illustrative):
   - `mcp_sse_retry_hint_total{source=retry_after|x_rate_limit|rate_limit}`
   - `mcp_sse_retry_delay_seconds` (histogram)
   - `mcp_sse_retry_fallback_total` (no hint → backoff)

## Testing Plan

Unit tests:
- `HttpRetryInfo::from_headers` parses:
  - `Retry-After: 120`
  - `Retry-After: Wed, 21 Oct 2015 07:28:00 GMT`
  - `X-RateLimit-Remaining: 0` with `X-RateLimit-Reset: <epoch-seconds>`
  - `RateLimit-Remaining: 0` with `RateLimit-Reset: <delta-seconds>`

Integration tests (SSE):
- Mock server returns 429 with `Retry-After: 2` → reconnect waits ~2s.
- Mock server returns 429 with `X-RateLimit-Remaining: 0` and `X-RateLimit-Reset` in the near future → reconnect waits until reset.
- Priority check: when both `Retry-After` and rate limit headers are present, `Retry-After` wins.

Resilience:
- If headers are malformed, parsing returns None and we fall back to backoff.

## Rollout & Compatibility

- Backward compatible: if no rate-limit headers present, behavior remains unchanged from Phase 1.
- Additional error field (`retry_info`) is optional; existing code paths remain functional.
- Consider feature flag to disable hint usage for troubleshooting.

## Timeline

- Day 1: Implement `HttpRetryInfo`, parsing, and unit tests.
- Day 2: Wire into SSE client and reconnection logic; add metrics/logs; write integration tests.
- Day 3: Validate in reverse proxy flows (prep for Phase 3 extraction).

When implementing this refactor:

1. Start by reading this design document
2. Check current state of the mentioned files
3. Implement Phase 1 first (basic Retry-After support)
4. Test with a mock server that sends Retry-After headers
5. Then proceed to Phase 2 and 3 for broader support

The key insight is that retry logic should be data-driven (using headers) rather than purely algorithmic (exponential backoff), while maintaining the fallback to exponential backoff when no guidance is available.